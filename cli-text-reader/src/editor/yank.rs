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

      // Track yank for tutorial
      if self.tutorial_active {
        self.tutorial_yank_performed = true;
      }
    }
  }

  // Yank current line
  pub fn yank_line(&mut self) {
    let cursor_line = self.offset + self.cursor_y;
    self.debug_log_event(
      "yank",
      "yank_line_start",
      &format!("cursor_line={}, total_lines={}", cursor_line, self.lines.len()),
    );

    if cursor_line < self.lines.len() {
      let line_text = self.lines[cursor_line].clone();
      self.editor_state.yank_buffer = line_text.clone();
      self.debug_log_state("yank", "yanked_line", &line_text);
      self.debug_log_state(
        "yank",
        "yank_buffer_updated",
        &self.editor_state.yank_buffer,
      );

      // Copy to system clipboard if available
      if let Some(clipboard) = &mut self.clipboard {
        match clipboard.set_text(&self.editor_state.yank_buffer) {
          Ok(_) => self.debug_log_event(
            "yank",
            "clipboard_success",
            "copied to system clipboard",
          ),
          Err(e) => self.debug_log_error(&format!("clipboard_failed: {e}")),
        }
      } else {
        self.debug_log_event(
          "yank",
          "clipboard_unavailable",
          "no system clipboard",
        );
      }

      // Track yank for tutorial
      if self.tutorial_active {
        self.tutorial_yank_performed = true;
      }
    } else {
      self.debug_log_error(&format!(
        "yank_line_bounds_error: cursor_line={}, total_lines={}",
        cursor_line,
        self.lines.len()
      ));
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

          // Track yank for tutorial
          if self.tutorial_active {
            self.tutorial_yank_performed = true;
          }
        }
      }
    }
  }
}
