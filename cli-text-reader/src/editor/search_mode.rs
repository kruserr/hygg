use crossterm::event::{self, KeyCode, KeyModifiers};

use super::core::{Editor, EditorMode};

impl Editor {
  // Handle key events in search modes (forward and reverse)
  pub fn handle_search_mode_event(
    &mut self,
    key_event: event::KeyEvent,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    // Handle Ctrl+C to exit to normal mode
    if key_event.code == KeyCode::Char('c')
      && key_event.modifiers.contains(KeyModifiers::CONTROL)
    {
      self.set_active_mode(EditorMode::Normal);
      self.editor_state.command_buffer.clear();
      if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
        buffer.command_buffer.clear();
        buffer.command_cursor_pos = 0;
      }
      self.mark_dirty();
      return Ok(false);
    }

    match key_event.code {
      KeyCode::Esc => {
        self.set_active_mode(EditorMode::Normal);
        self.editor_state.command_buffer.clear();
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
          buffer.command_buffer.clear();
          buffer.command_cursor_pos = 0;
        }
        self.mark_dirty();
      }
      KeyCode::Enter => {
        self.editor_state.search_query =
          self.editor_state.command_buffer.clone();
        // Start from current position using find_first_match
        // Use stored search_direction which was set when entering search mode
        self.find_first_match(self.editor_state.search_direction);
        self.center_on_match();
        self.set_active_mode(EditorMode::Normal);
        self.editor_state.command_buffer.clear();
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
          buffer.command_buffer.clear();
          buffer.command_cursor_pos = 0;
        }
        self.mark_dirty();
      }
      KeyCode::Backspace => {
        if !self.editor_state.command_buffer.is_empty() {
          self.editor_state.command_buffer.pop();
          self.editor_state.command_cursor_pos = self.editor_state.command_buffer.len();
          
          // Sync with active buffer
          if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
            buffer.command_buffer = self.editor_state.command_buffer.clone();
            buffer.command_cursor_pos = self.editor_state.command_buffer.len();
          }
        } else {
          // If search buffer is already empty, exit search mode
          self.set_active_mode(EditorMode::Normal);
          if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
            buffer.command_buffer.clear();
            buffer.command_cursor_pos = 0;
          }
        }
        self.mark_dirty();
      }
      KeyCode::Char(c) => {
        self.editor_state.command_buffer.push(c);
        // Sync with active buffer
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
          buffer.command_buffer = self.editor_state.command_buffer.clone();
          buffer.command_cursor_pos = self.editor_state.command_buffer.len();
        }
        self.mark_dirty();
      }
      _ => {}
    }
    Ok(false)
  }
}
