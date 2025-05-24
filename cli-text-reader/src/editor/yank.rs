use super::core::Editor;

impl Editor {
  // Yank selected text to buffer and system clipboard
  pub fn yank_selection(&mut self) {
    let selected_text = self.get_selected_text();
    if !selected_text.is_empty() {
      self.editor_state.yank_buffer = selected_text.clone();

      // Copy to system clipboard if available
      if let Some(clipboard) = &mut self.clipboard {
        let _ = clipboard.set_text(&selected_text);
      }
    }
  }

  // Yank current line
  pub fn yank_line(&mut self) {
    let cursor_line = self.offset + self.cursor_y;
    if cursor_line < self.lines.len() {
      self.editor_state.yank_buffer = self.lines[cursor_line].clone();

      // Copy to system clipboard if available
      if let Some(clipboard) = &mut self.clipboard {
        let _ = clipboard.set_text(&self.editor_state.yank_buffer);
      }
    }
  }

  // Yank word under cursor
  pub fn yank_word(&mut self) {
    let (line_idx, col_idx) = self.get_cursor_position();
    if line_idx < self.lines.len() {
      let line = &self.lines[line_idx];

      // Find word boundaries
      if col_idx < line.len() {
        let mut start = col_idx;
        while start > 0
          && line
            .chars()
            .nth(start - 1)
            .is_some_and(|c| !c.is_whitespace() && c.is_alphanumeric())
        {
          start -= 1;
        }

        let mut end = col_idx;
        while end < line.len()
          && line
            .chars()
            .nth(end)
            .is_some_and(|c| !c.is_whitespace() && c.is_alphanumeric())
        {
          end += 1;
        }

        if start < end {
          self.editor_state.yank_buffer = line[start..end].to_string();

          // Copy to system clipboard if available
          if let Some(clipboard) = &mut self.clipboard {
            let _ = clipboard.set_text(&self.editor_state.yank_buffer);
          }
        }
      }
    }
  }
}
