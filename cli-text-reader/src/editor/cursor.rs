use crossterm::{
  cursor::{MoveTo, SetCursorStyle, Show},
  execute,
};
use std::io::{self, IsTerminal};

use super::core::{Editor, EditorMode};

impl Editor {
  // Position and style the cursor based on editor mode
  pub fn position_cursor(
    &self,
    stdout: &mut io::Stdout,
    center_offset: usize,
  ) -> io::Result<()> {
    if std::io::stdout().is_terminal() {
      // Position the cursor at current position in text
      if self.editor_state.mode == EditorMode::Normal
        || self.editor_state.mode == EditorMode::VisualChar
        || self.editor_state.mode == EditorMode::VisualLine
      {
        // Position cursor exactly on the highlighted line
        execute!(
          stdout,
          Show,
          MoveTo((center_offset + self.cursor_x) as u16, self.cursor_y as u16),
          // Ensure cursor is clearly visible regardless of terminal settings
          SetCursorStyle::BlinkingBlock
        )?;

        // Set appropriate cursor style for the mode
        match self.editor_state.mode {
          EditorMode::Normal => {
            execute!(stdout, SetCursorStyle::BlinkingBlock)?;
          }
          EditorMode::VisualChar | EditorMode::VisualLine => {
            execute!(stdout, SetCursorStyle::SteadyBlock)?;
          }
          _ => {}
        }
      } else if self.editor_state.mode == EditorMode::Command
        || self.editor_state.mode == EditorMode::Search
        || self.editor_state.mode == EditorMode::ReverseSearch
      {
        // In command/search mode, position cursor after the buffer content
        let cmd_len = match self.editor_state.mode {
          EditorMode::Command => 1 + self.editor_state.command_buffer.len(), /* ":" + buffer */
          EditorMode::Search => 1 + self.editor_state.command_buffer.len(), /* "/" + buffer */
          EditorMode::ReverseSearch => {
            1 + self.editor_state.command_buffer.len()
          } /* "?" + buffer */
          _ => 0,
        };

        execute!(
          stdout,
          Show,
          MoveTo(cmd_len as u16, (self.height - 1) as u16),
          SetCursorStyle::BlinkingBar
        )?;
      }
    }
    Ok(())
  }

  // Calculate center position for cursor - called from main_loop
  pub fn center_cursor(&mut self) {
    self.center_cursor_with_overscroll(true);
  }

  // Calculate center position for cursor with optional overscroll
  pub fn center_cursor_with_overscroll(&mut self, allow_overscroll: bool) {
    // Get the actual line we're focusing on (absolute document position)
    let current_line = self.offset + self.cursor_y;

    // Ensure we don't go beyond document boundaries
    let current_line = current_line.min(self.total_lines.saturating_sub(1));

    // Calculate center position for cursor - place in middle of content area
    // (excluding status line)
    let content_height = self.height.saturating_sub(1);
    let center_y = content_height / 2;

    if allow_overscroll {
      // With overscroll, always try to center the current line on screen
      // This allows first and last lines to be centered with blank lines
      // above/below

      // Calculate the offset needed to center the current line
      let desired_offset = if current_line >= center_y {
        current_line - center_y
      } else {
        // For lines near the beginning, use negative offset (handled as 0)
        0
      };

      // Allow negative offset conceptually by using the offset as signed
      if current_line < center_y {
        // We're in the overscroll region at the top
        self.offset = 0;
        self.cursor_y = current_line;
      } else {
        // Normal case or overscroll at bottom
        self.offset = desired_offset;
        self.cursor_y = center_y;
      }

      // No maximum offset limit - allow overscroll at the bottom too
      // This means we can have the last line centered with blank lines below
    } else {
      // Original logic without overscroll
      // First handle boundary cases
      if current_line < center_y {
        // Too close to the top to center properly
        self.offset = 0;
        self.cursor_y = current_line;
      } else if current_line >= self.total_lines.saturating_sub(center_y) {
        // Too close to the bottom to center properly
        if self.total_lines > content_height {
          self.offset = self.total_lines - content_height;
          self.cursor_y = current_line - self.offset;
        } else {
          // Document is shorter than screen height
          self.offset = 0;
          self.cursor_y = current_line;
        }
      } else {
        // Standard case - we can properly center
        self.offset = current_line - center_y;
        self.cursor_y = center_y;
      }

      // Final boundary check to ensure we don't go beyond document limits
      if self.offset + content_height > self.total_lines {
        self.offset = self.total_lines.saturating_sub(content_height);
      }
    }
  }
}
