use crossterm::terminal;
use std::io;

use super::core::Editor;

impl Editor {
  pub fn execute_command(
    &mut self,
    stdout: &mut io::Stdout,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    match self.editor_state.command_buffer.trim() {
      "p" => {
        self.show_progress = !self.show_progress;
        self.editor_state.mode = super::core::EditorMode::Normal;
        self.editor_state.command_buffer.clear();
        Ok(false)
      }
      "help" | "tutorial" => {
        self.show_tutorial(stdout)?;
        self.editor_state.mode = super::core::EditorMode::Normal;
        self.editor_state.command_buffer.clear();
        Ok(false)
      }
      cmd => Ok(handle_command(cmd, &mut self.show_highlighter)),
    }
  }

  // Find next text match for search functionality
  pub fn find_next_match(&mut self, forward: bool) {
    if self.editor_state.search_query.is_empty() {
      return;
    }

    let query = self.editor_state.search_query.to_lowercase();
    let start_idx = if let Some((idx, _, _)) = self.editor_state.current_match {
      idx
    } else {
      self.offset
    };

    let find_in_line = |line: &str, query: &str| -> Option<(usize, usize)> {
      line.to_lowercase().find(query).map(|start| (start, start + query.len()))
    };

    if forward {
      // Forward search
      for i in start_idx + 1..self.lines.len() {
        if let Some((start, end)) = find_in_line(&self.lines[i], &query) {
          self.editor_state.current_match = Some((i, start, end));
          return;
        }
      }
      // Wrap around to beginning
      for i in 0..=start_idx {
        if let Some((start, end)) = find_in_line(&self.lines[i], &query) {
          self.editor_state.current_match = Some((i, start, end));
          return;
        }
      }
    } else {
      // Backward search
      for i in (0..start_idx).rev() {
        if let Some((start, end)) = find_in_line(&self.lines[i], &query) {
          self.editor_state.current_match = Some((i, start, end));
          return;
        }
      }
      // Wrap around to end
      for i in (start_idx..self.lines.len()).rev() {
        if let Some((start, end)) = find_in_line(&self.lines[i], &query) {
          self.editor_state.current_match = Some((i, start, end));
          return;
        }
      }
    }
  }

  // Center the view on the current search match
  pub fn center_on_match(&mut self) {
    if let Some((line_idx, _, _)) = self.editor_state.current_match {
      let content_height = self.height.saturating_sub(1);
      let half_height = (content_height / 2) as i32;
      let new_offset = line_idx as i32 - half_height;
      self.offset = if new_offset < 0 {
        0
      } else if new_offset + content_height as i32 > self.total_lines as i32 {
        self.total_lines - content_height
      } else {
        new_offset as usize
      };
    }
  }
}

// Handle Vim-style commands
pub fn handle_command(command: &str, show_highlighter: &mut bool) -> bool {
  match command.trim() {
    "q" => true,
    "z" => {
      *show_highlighter = !*show_highlighter;
      false
    }
    "p" | "help" | "tutorial" => false,
    _ => false,
  }
}
