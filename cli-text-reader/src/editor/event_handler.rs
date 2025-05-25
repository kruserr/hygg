use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers};
use std::io;

use super::core::{Editor, EditorMode};

impl Editor {
  // Handle key events in normal mode
  pub fn handle_normal_mode_event(
    &mut self,
    key_event: event::KeyEvent,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    match key_event.code {
      KeyCode::Char(':') => {
        self.editor_state.mode = EditorMode::Command;
        self.editor_state.command_buffer.clear();
      }
      KeyCode::Char('/') => {
        self.editor_state.mode = EditorMode::Search;
        self.editor_state.command_buffer.clear();
        self.editor_state.search_direction = true;
      }
      KeyCode::Char('?') => {
        self.editor_state.mode = EditorMode::ReverseSearch;
        self.editor_state.command_buffer.clear();
        self.editor_state.search_direction = false;
      }
      KeyCode::Char('n') => {
        if !self.editor_state.search_query.is_empty() {
          // Use the original search direction
          self.find_next_match(self.editor_state.search_direction);
          self.center_on_match();
        }
      }
      KeyCode::Char('N') => {
        if !self.editor_state.search_query.is_empty() {
          // Use opposite of original search direction
          self.find_next_match(!self.editor_state.search_direction);
          self.center_on_match();
        }
      }
      KeyCode::Char('v') => {
        // Enter visual character mode
        self.editor_state.mode = EditorMode::VisualChar;
        self.start_selection();
      }
      KeyCode::Char('V') => {
        // Enter visual line mode
        self.editor_state.mode = EditorMode::VisualLine;
        self.start_selection();
      }
      KeyCode::Char('y') => {
        match key_event.modifiers {
          event::KeyModifiers::SHIFT => {
            // 'Y' - Same as 'yy'
            self.yank_line();
          }
          _ => {
            // Set operator pending for 'y'
            self.editor_state.operator_pending = Some('y');
          }
        }
      }
      KeyCode::Char('j') | KeyCode::Down => {
        self.move_cursor_down();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        self.move_cursor_up();
      }
      KeyCode::Char('h') | KeyCode::Left => {
        self.move_cursor_left();
      }
      KeyCode::Char('l') | KeyCode::Right => {
        self.move_cursor_right();
      }
      KeyCode::Char('w') => {
        // Word forward motion
        self.move_to_word_boundary(true, false);
      }
      KeyCode::Char('W') => {
        // Big word forward motion
        self.move_to_word_boundary(true, true);
      }
      KeyCode::Char('b') => {
        // Word backward motion
        self.move_to_word_boundary(false, false);
      }
      KeyCode::Char('B') => {
        // Big word backward motion
        self.move_to_word_boundary(false, true);
      }
      KeyCode::Char('i') => {
        // Text object selection
        if let Some('v') = self.editor_state.operator_pending {
          // For visual text object selection operations like 'viw'
          match event::read()? {
            CEvent::Key(inner_key) => match inner_key.code {
              KeyCode::Char('w') => {
                // Inner word
                self.select_inner_word(false);
                self.editor_state.operator_pending = None;
              }
              KeyCode::Char('W') => {
                // Inner WORD
                self.select_inner_word(true);
                self.editor_state.operator_pending = None;
              }
              KeyCode::Char('"')
              | KeyCode::Char('\'')
              | KeyCode::Char('(')
              | KeyCode::Char(')')
              | KeyCode::Char('{')
              | KeyCode::Char('}')
              | KeyCode::Char('[')
              | KeyCode::Char(']') => {
                // Inner quotes, parentheses, braces, brackets
                if let KeyCode::Char(c) = inner_key.code {
                  if let Some((start, end)) = self.find_text_object(c) {
                    let line_idx = self.offset + self.cursor_y;
                    self.editor_state.selection_start = Some((line_idx, start));
                    self.editor_state.selection_end = Some((line_idx, end));
                  }
                }
                self.editor_state.operator_pending = None;
              }
              _ => {
                self.editor_state.operator_pending = None;
              }
            },
            _ => {
              self.editor_state.operator_pending = None;
            }
          }
        }
      }
      KeyCode::PageDown => {
        // Move cursor down by page height to get overscroll behavior
        let current_line = self.offset + self.cursor_y;
        let content_height = self.height.saturating_sub(1);
        let page_size = content_height.saturating_sub(3); // Leave some overlap
        let target_line =
          (current_line + page_size).min(self.total_lines.saturating_sub(1));

        if target_line != current_line {
          self.goto_line_with_overscroll(target_line);
        }
      }
      KeyCode::PageUp => {
        // Move cursor up by page height to get overscroll behavior
        let current_line = self.offset + self.cursor_y;
        let content_height = self.height.saturating_sub(1);
        let page_size = content_height.saturating_sub(3); // Leave some overlap
        let target_line = current_line.saturating_sub(page_size);

        if target_line != current_line {
          self.goto_line_with_overscroll(target_line);
        }
      }
      KeyCode::Char('g') => {
        // Handle 'gg' motion - wait for another 'g'
        if let CEvent::Key(inner_key) = event::read()? {
          match inner_key.code {
            KeyCode::Char('g') => {
              // 'gg' - go to first line with overscroll
              self.goto_line_with_overscroll(0);
            }
            _ => {
              // Invalid sequence, ignore
            }
          }
        }
      }
      KeyCode::Char('G') => {
        // 'G' - go to last line with overscroll
        let last_line = self.total_lines.saturating_sub(1);
        self.goto_line_with_overscroll(last_line);
      }
      _ => {
        // Handle operator pending operations
        if let Some(op) = self.editor_state.operator_pending {
          match op {
            'y' => {
              match key_event.code {
                KeyCode::Char('y') => {
                  // 'yy' - Yank current line
                  self.yank_line();
                }
                KeyCode::Char('w') => {
                  // 'yw' - Yank word
                  self.yank_word();
                }
                _ => {}
              }
              self.editor_state.operator_pending = None;
            }
            _ => {
              self.editor_state.operator_pending = None;
            }
          }
        }
      }
    }
    Ok(false)
  }

  // Handle key events in visual mode (char and line)
  pub fn handle_visual_mode_event(
    &mut self,
    key_event: event::KeyEvent,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    match key_event.code {
      KeyCode::Esc => {
        // Exit visual mode
        self.editor_state.mode = EditorMode::Normal;
        self.clear_selection();
      }
      KeyCode::Char('y') => {
        // Yank selection and exit visual mode
        self.yank_selection();
        self.editor_state.mode = EditorMode::Normal;
        self.clear_selection();
      }
      KeyCode::Char('j') | KeyCode::Down => {
        self.move_cursor_down();
        self.update_selection();
      }
      KeyCode::Char('k') | KeyCode::Up => {
        self.move_cursor_up();
        self.update_selection();
      }
      KeyCode::Char('h') | KeyCode::Left => {
        self.move_cursor_left();
        self.update_selection();
      }
      KeyCode::Char('l') | KeyCode::Right => {
        self.move_cursor_right();
        self.update_selection();
      }
      KeyCode::Char('w') => {
        // Word forward motion
        self.move_to_word_boundary(true, false);
        self.update_selection();
      }
      KeyCode::Char('W') => {
        // Big word forward motion
        self.move_to_word_boundary(true, true);
        self.update_selection();
      }
      KeyCode::Char('b') => {
        // Word backward motion
        self.move_to_word_boundary(false, false);
        self.update_selection();
      }
      KeyCode::Char('B') => {
        // Big word backward motion
        self.move_to_word_boundary(false, true);
        self.update_selection();
      }
      _ => {}
    }
    Ok(false)
  }

  // Handle key events in search modes (forward and reverse)
  pub fn handle_search_mode_event(
    &mut self,
    key_event: event::KeyEvent,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    match key_event.code {
      KeyCode::Esc => {
        self.editor_state.mode = EditorMode::Normal;
        self.editor_state.command_buffer.clear();
      }
      KeyCode::Enter => {
        self.editor_state.search_query =
          self.editor_state.command_buffer.clone();
        // Start from current position
        self.find_next_match(self.editor_state.mode == EditorMode::Search);
        self.center_on_match();
        self.editor_state.mode = EditorMode::Normal;
        self.editor_state.command_buffer.clear();
      }
      KeyCode::Backspace => {
        self.editor_state.command_buffer.pop();
      }
      KeyCode::Char(c) => {
        self.editor_state.command_buffer.push(c);
      }
      _ => {}
    }
    Ok(false)
  }

  // Handle key events in command mode
  pub fn handle_command_mode_event(
    &mut self,
    key_event: event::KeyEvent,
    stdout: &mut io::Stdout,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    match key_event.code {
      KeyCode::Esc => {
        self.editor_state.mode = EditorMode::Normal;
        self.editor_state.command_buffer.clear();
      }
      KeyCode::Enter => {
        if self.execute_command(stdout)? {
          return Ok(true);
        }
        self.editor_state.mode = EditorMode::Normal;
        self.editor_state.command_buffer.clear();
      }
      KeyCode::Backspace => {
        self.editor_state.command_buffer.pop();
      }
      KeyCode::Char('r')
        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
      {
        // Ctrl+R in command mode - paste from register
        if let CEvent::Key(register_key) = event::read()? {
          match register_key.code {
            KeyCode::Char('0') => {
              // Paste from yank buffer (register 0)
              self
                .editor_state
                .command_buffer
                .push_str(&self.editor_state.yank_buffer);
            }
            _ => {} // Other registers not implemented yet
          }
        }
      }
      KeyCode::Char('v')
        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
      {
        // Ctrl+V in command mode - paste from system clipboard
        if let Some(clipboard) = &mut self.clipboard {
          if let Ok(clipboard_text) = clipboard.get_text() {
            self.editor_state.command_buffer.push_str(&clipboard_text);
          }
        }
      }
      KeyCode::Char(c) => {
        self.editor_state.command_buffer.push(c);
      }
      _ => {}
    }
    Ok(false)
  }

  // Helper methods for cursor movement

  // Move cursor down one line, handling boundary conditions
  pub fn move_cursor_down(&mut self) {
    // Get the current absolute line position
    let current_line = self.offset + self.cursor_y;

    // Strict boundary check - cannot move beyond last line
    if current_line >= self.total_lines.saturating_sub(1) {
      return; // Already at last line, cannot move down
    }

    let new_line = current_line + 1;

    // Update the line position and recenter with overscroll
    let content_height = self.height.saturating_sub(1);
    let center_y = content_height / 2;

    // Always try to center the new line (overscroll behavior)
    if new_line < center_y {
      // Near the beginning - cursor follows line position
      self.offset = 0;
      self.cursor_y = new_line;
    } else {
      // Center the line on screen
      self.offset = new_line - center_y;
      self.cursor_y = center_y;
    }

    // Keep cursor position on the current line
    if let Some(line) = self.lines.get(new_line) {
      self.cursor_x = self.cursor_x.min(line.len().saturating_sub(1));
    }
  }

  // Move cursor up one line, handling boundary conditions
  pub fn move_cursor_up(&mut self) {
    // Get the current absolute line position
    let current_line = self.offset + self.cursor_y;

    // Strict boundary check - cannot move beyond first line
    if current_line == 0 {
      return; // Already at first line, cannot move up
    }

    let new_line = current_line - 1;

    // Update the line position and recenter with overscroll
    let content_height = self.height.saturating_sub(1);
    let center_y = content_height / 2;

    // Always try to center the new line (overscroll behavior)
    if new_line < center_y {
      // Near the beginning - cursor follows line position
      self.offset = 0;
      self.cursor_y = new_line;
    } else {
      // Center the line on screen
      self.offset = new_line - center_y;
      self.cursor_y = center_y;
    }

    // Keep cursor position on the current line
    if let Some(line) = self.lines.get(new_line) {
      self.cursor_x = self.cursor_x.min(line.len().saturating_sub(1));
    }
  }

  // Move cursor left one character
  pub fn move_cursor_left(&mut self) {
    if self.cursor_x > 0 {
      self.cursor_x -= 1;
    }
  }

  // Move cursor right one character
  pub fn move_cursor_right(&mut self) {
    let current_line_idx = self.offset + self.cursor_y;
    if current_line_idx < self.lines.len() {
      let line_length = self.lines[current_line_idx].len();
      if self.cursor_x < line_length.saturating_sub(1) {
        self.cursor_x += 1;
      }
    }
  }

  // Move cursor to word boundary
  pub fn move_to_word_boundary(&mut self, forward: bool, big_word: bool) {
    let (new_line, new_col) = self.find_word_boundary(forward, big_word);

    if new_line != self.offset + self.cursor_y {
      // Line changed - need to update cursor position properly
      let content_height = self.height.saturating_sub(1);
      let center_y = content_height / 2;

      if new_line >= self.offset && new_line < self.offset + content_height {
        // New line is visible on screen
        self.cursor_y = new_line - self.offset;
      } else {
        // New line is off-screen, center it with overscroll capability
        if new_line < center_y {
          // Near beginning of document
          self.offset = 0;
          self.cursor_y = new_line;
        } else {
          // Use overscroll-style centering
          self.offset = new_line.saturating_sub(center_y);
          self.cursor_y = center_y;
        }
      }
    }

    self.cursor_x = new_col;

    // Use overscroll centering for better navigation experience
    self.center_cursor_with_overscroll(true);
  }
}
