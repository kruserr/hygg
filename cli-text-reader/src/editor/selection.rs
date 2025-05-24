use super::core::Editor;

impl Editor {
  // Start selection at current cursor position
  pub fn start_selection(&mut self) {
    let pos = self.get_cursor_position();
    self.editor_state.selection_start = Some(pos);
    self.editor_state.selection_end = Some(pos);
  }

  // Update selection end to current cursor position
  pub fn update_selection(&mut self) {
    let pos = self.get_cursor_position();
    self.editor_state.selection_end = Some(pos);
  }

  // Clear current selection
  pub fn clear_selection(&mut self) {
    self.editor_state.selection_start = None;
    self.editor_state.selection_end = None;
  }

  // Extract selected text based on start and end positions
  pub fn get_selected_text(&self) -> String {
    if let (Some(start), Some(end)) =
      (self.editor_state.selection_start, self.editor_state.selection_end)
    {
      let (start_line, start_col) = if start.0 <= end.0 { start } else { end };

      let (end_line, end_col) = if start.0 <= end.0 { end } else { start };

      // Handle visual line mode vs character mode differently
      if self.editor_state.mode == super::core::EditorMode::VisualLine {
        // In line mode, select entire lines
        if start_line < self.lines.len() && end_line < self.lines.len() {
          return self.lines[start_line..=end_line].join("\n");
        }
      } else {
        // In character mode, select parts of lines
        if start_line == end_line {
          // Single line selection
          if start_line < self.lines.len() {
            let line = &self.lines[start_line];
            let start_col = start_col.min(line.len());
            let end_col = end_col.min(line.len());
            if start_col < end_col {
              return line[start_col..end_col].to_string();
            } else {
              return line[end_col..start_col].to_string();
            }
          }
        } else {
          // Multi-line selection
          let mut result = String::new();

          // First line (partial)
          if start_line < self.lines.len() {
            let line = &self.lines[start_line];
            let start_col = start_col.min(line.len());
            result.push_str(&line[start_col..]);
            result.push('\n');
          }

          // Middle lines (full)
          for i in start_line + 1..end_line {
            if i < self.lines.len() {
              result.push_str(&self.lines[i]);
              result.push('\n');
            }
          }

          // Last line (partial)
          if end_line < self.lines.len() {
            let line = &self.lines[end_line];
            let end_col = end_col.min(line.len());
            result.push_str(&line[..end_col]);
          }

          return result;
        }
      }
    }

    String::new()
  }

  // Select inner word
  pub fn select_inner_word(&mut self, big_word: bool) {
    let (line_idx, col_idx) = self.get_cursor_position();

    if line_idx >= self.lines.len() {
      return;
    }

    let line = &self.lines[line_idx];

    if line.is_empty() {
      return;
    }

    // Clamp cursor position to valid range
    let col_idx = col_idx.min(line.len().saturating_sub(1));

    if big_word {
      // Big word (space separated) - non-whitespace sequences
      let is_whitespace = |idx: usize| {
        line.chars().nth(idx).map_or(true, |c| c.is_whitespace())
      };

      // If cursor is on whitespace, no word to select
      if is_whitespace(col_idx) {
        return;
      }

      // Find word boundaries
      let mut start = col_idx;
      let mut end = col_idx + 1; // Start with next character

      // Move backward to find start
      while start > 0 && !is_whitespace(start - 1) {
        start -= 1;
      }

      // Move forward to find end
      while end < line.len() && !is_whitespace(end) {
        end += 1;
      }

      // Set selection
      self.editor_state.selection_start = Some((line_idx, start));
      self.editor_state.selection_end = Some((line_idx, end));
    } else {
      // Regular word (alphanumeric + underscore) - vim's 'w' behavior
      let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
      let char_at = |idx: usize| line.chars().nth(idx).unwrap_or(' ');

      let current_char = char_at(col_idx);

      // If cursor is on whitespace, no word to select  
      if current_char.is_whitespace() {
        return;
      }

      let mut start = col_idx;
      let mut end = col_idx + 1; // Start with next character

      if is_word_char(current_char) {
        // We're on a word character - find word boundaries
        
        // Move backward to find start of word
        while start > 0 && is_word_char(char_at(start - 1)) {
          start -= 1;
        }

        // Move forward to find end of word
        while end < line.len() && is_word_char(char_at(end)) {
          end += 1;
        }
      } else {
        // We're on punctuation - find punctuation sequence boundaries
        
        // Move backward to find start of punctuation sequence
        while start > 0 {
          let prev_char = char_at(start - 1);
          if prev_char.is_whitespace() || is_word_char(prev_char) {
            break;
          }
          start -= 1;
        }

        // Move forward to find end of punctuation sequence
        while end < line.len() {
          let next_char = char_at(end);
          if next_char.is_whitespace() || is_word_char(next_char) {
            break;
          }
          end += 1;
        }
      }

      // Set selection
      self.editor_state.selection_start = Some((line_idx, start));
      self.editor_state.selection_end = Some((line_idx, end));
    }
  }
}
