use super::core::Editor;

impl Editor {
  // Helper to check if character is a word character (alphanumeric +
  // underscore)
  fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
  }

  // Helper to find word boundaries for motions
  pub fn find_word_boundary(
    &self,
    forward: bool,
    big_word: bool,
  ) -> (usize, usize) {
    let (line_idx, col_idx) = self.get_cursor_position();

    if line_idx >= self.lines.len() {
      return (line_idx, col_idx);
    }

    let line = &self.lines[line_idx];

    if forward {
      // Moving forward to next word (vim 'w' behavior)
      let mut new_col = col_idx;

      if new_col >= line.len() {
        // At end of line, move to next line
        if line_idx + 1 < self.lines.len() {
          return self.find_next_line_start(line_idx + 1);
        }
        return (line_idx, col_idx);
      }

      let current_char = line.chars().nth(new_col).unwrap_or(' ');

      if big_word {
        // For big words (W), only whitespace separates words
        // Skip current non-whitespace sequence
        while new_col < line.len() {
          let c = line.chars().nth(new_col).unwrap_or(' ');
          if c.is_whitespace() {
            break;
          }
          new_col += 1;
        }
      } else {
        // For regular words (w), punctuation and alphanumeric are separate
        if Self::is_word_char(current_char) {
          // Skip current word characters
          while new_col < line.len() {
            let c = line.chars().nth(new_col).unwrap_or(' ');
            if !Self::is_word_char(c) {
              break;
            }
            new_col += 1;
          }
        } else if !current_char.is_whitespace() {
          // Skip current punctuation sequence
          while new_col < line.len() {
            let c = line.chars().nth(new_col).unwrap_or(' ');
            if c.is_whitespace() || Self::is_word_char(c) {
              break;
            }
            new_col += 1;
          }
        }
      }

      // Skip whitespace to find start of next word
      while new_col < line.len() {
        let c = line.chars().nth(new_col).unwrap_or(' ');
        if !c.is_whitespace() {
          break;
        }
        new_col += 1;
      }

      if new_col >= line.len() {
        // Reached end of line, move to next line
        if line_idx + 1 < self.lines.len() {
          return self.find_next_line_start(line_idx + 1);
        }
      }

      return (line_idx, new_col);
    } else {
      // Moving backward to previous word (vim 'b' behavior)
      if col_idx == 0 {
        // At beginning of line, go to end of previous line
        if line_idx > 0 {
          return self.find_prev_line_end(line_idx - 1);
        }
        return (line_idx, col_idx);
      }

      let mut new_col = col_idx.saturating_sub(1);

      // Skip whitespace backwards
      while new_col > 0 {
        let c = line.chars().nth(new_col).unwrap_or(' ');
        if !c.is_whitespace() {
          break;
        }
        new_col = new_col.saturating_sub(1);
      }

      // Now find the start of the current word
      let char_at_pos = line.chars().nth(new_col).unwrap_or(' ');

      if big_word {
        // For big words, find start of non-whitespace sequence
        while new_col > 0 {
          let prev_char = line.chars().nth(new_col - 1).unwrap_or(' ');
          if prev_char.is_whitespace() {
            break;
          }
          new_col -= 1;
        }
      } else {
        // For regular words, separate by character type
        if Self::is_word_char(char_at_pos) {
          // Find start of word character sequence
          while new_col > 0 {
            let prev_char = line.chars().nth(new_col - 1).unwrap_or(' ');
            if !Self::is_word_char(prev_char) {
              break;
            }
            new_col -= 1;
          }
        } else {
          // Find start of punctuation sequence
          while new_col > 0 {
            let prev_char = line.chars().nth(new_col - 1).unwrap_or(' ');
            if prev_char.is_whitespace() || Self::is_word_char(prev_char) {
              break;
            }
            new_col -= 1;
          }
        }
      }

      return (line_idx, new_col);
    }
  }

  // Helper to find the start of the next non-empty line
  fn find_next_line_start(&self, start_line: usize) -> (usize, usize) {
    for line_idx in start_line..self.lines.len() {
      let line = &self.lines[line_idx];
      for (col_idx, c) in line.char_indices() {
        if !c.is_whitespace() {
          return (line_idx, col_idx);
        }
      }
    }
    // If no non-whitespace found, go to start of the target line
    if start_line < self.lines.len() {
      (start_line, 0)
    } else {
      (self.lines.len().saturating_sub(1), 0)
    }
  }

  // Helper to find the end of the previous line (last word)
  fn find_prev_line_end(&self, target_line: usize) -> (usize, usize) {
    if target_line >= self.lines.len() {
      return (0, 0);
    }

    let line = &self.lines[target_line];

    // Find the last non-whitespace character
    let mut end_pos = line.len();
    while end_pos > 0 {
      let c = line.chars().nth(end_pos - 1).unwrap_or(' ');
      if !c.is_whitespace() {
        break;
      }
      end_pos -= 1;
    }

    // Now find the start of the last word
    let mut start_pos = end_pos;
    if start_pos > 0 {
      let char_at_end = line.chars().nth(start_pos - 1).unwrap_or(' ');

      if Self::is_word_char(char_at_end) {
        // Find start of word characters
        while start_pos > 0 {
          let prev_char = line.chars().nth(start_pos - 1).unwrap_or(' ');
          if !Self::is_word_char(prev_char) {
            break;
          }
          start_pos -= 1;
        }
      } else {
        // Find start of punctuation
        while start_pos > 0 {
          let prev_char = line.chars().nth(start_pos - 1).unwrap_or(' ');
          if prev_char.is_whitespace() || Self::is_word_char(prev_char) {
            break;
          }
          start_pos -= 1;
        }
      }
    }

    (target_line, start_pos)
  }

  // Navigate to a specific line with overscroll capability
  pub fn goto_line_with_overscroll(&mut self, target_line: usize) {
    // Clamp target line to valid range
    let target_line = target_line.min(self.total_lines.saturating_sub(1));

    // Calculate new offset and cursor position to center the target line
    let content_height = self.height.saturating_sub(1);
    let center_y = content_height / 2;

    // Always try to center the target line (overscroll behavior)
    self.offset = target_line.saturating_sub(center_y);

    // Calculate cursor_y position
    if target_line < center_y {
      // Target line is near the beginning - cursor follows the line
      self.cursor_y = target_line;
      self.offset = 0;
    } else {
      // Center the cursor
      self.cursor_y = center_y;
    }

    // Set cursor to beginning of line
    self.cursor_x = 0;

    // Find first non-whitespace character on the line if it exists
    if target_line < self.lines.len() {
      let line = &self.lines[target_line];
      for (idx, c) in line.char_indices() {
        if !c.is_whitespace() {
          self.cursor_x = idx;
          break;
        }
      }
    }
  }

  // Find the next delimiter in the current line
  pub fn find_text_object(&self, delimiter: char) -> Option<(usize, usize)> {
    let (line_idx, col_idx) = self.get_cursor_position();

    if line_idx >= self.lines.len() {
      return None;
    }

    let line = &self.lines[line_idx];

    // Define opening and closing pairs
    let (opening, closing) = match delimiter {
      '"' => ('"', '"'),
      '\'' => ('\'', '\''),
      '(' | ')' => ('(', ')'),
      '{' | '}' => ('{', '}'),
      '[' | ']' => ('[', ']'),
      _ => return None,
    };

    // Search left for opening delimiter
    let mut start = col_idx;
    let mut depth = 0;

    while start > 0 {
      start -= 1;
      if let Some(c) = line.chars().nth(start) {
        if c == closing {
          depth += 1;
        } else if c == opening {
          if depth == 0 {
            // Found the opening delimiter
            break;
          }
          depth -= 1;
        }
      }
    }

    // Search right for closing delimiter
    let mut end = col_idx;
    depth = 0;

    while end < line.len() {
      if let Some(c) = line.chars().nth(end) {
        if c == opening {
          depth += 1;
        } else if c == closing {
          if depth == 0 {
            // Found the closing delimiter
            break;
          }
          depth -= 1;
        }
      }
      end += 1;
    }

    if end < line.len() && start < end {
      // Return the inner positions (exclude the delimiters)
      return Some((start + 1, end));
    }

    None
  }
}
