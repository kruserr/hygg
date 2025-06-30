use crossterm::event::{KeyCode, KeyModifiers};

use super::core::{Editor, EditorMode, ViewMode};

impl Editor {
  // Handle control and mode switching key events in normal mode
  pub fn handle_control_keys(
    &mut self,
    key_event: crossterm::event::KeyEvent,
  ) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    match key_event.code {
      KeyCode::Char('b')
        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
      {
        // Handle Ctrl+B for tmux-style prefix
        if self.view_mode == ViewMode::HorizontalSplit {
          self.debug_log("Ctrl+B pressed - entering tmux prefix mode");
          self.tmux_prefix_active = true;
          Ok(Some(false))
        } else {
          self.debug_log("Ctrl+B pressed but not in split view - ignoring");
          Ok(Some(false))
        }
      }
      KeyCode::Char('c')
        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
      {
        // Don't allow Ctrl+C to close tutorial overlay
        if self.tutorial_active {
          self.debug_log(
            "Ctrl+C pressed in tutorial mode - ignoring (use :q to exit)",
          );
          Ok(Some(false))
        } else if self.can_close_buffer() {
          self.debug_log(
            "Ctrl+C pressed in Normal mode with overlay, closing overlay",
          );
          self.close_overlay();
          Ok(Some(false))
        } else {
          self.debug_log(
            "Ctrl+C pressed in Normal mode, but no overlay - ignoring",
          );
          Ok(Some(false))
        }
      }
      KeyCode::Char(':') => {
        self.debug_log_event("normal_mode", "enter_command_mode", "");
        self.set_active_mode(EditorMode::Command);
        self.editor_state.command_buffer.clear();
        self.editor_state.command_cursor_pos = 0;
        // Also clear active buffer's command buffer
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
          buffer.command_buffer.clear();
          buffer.command_cursor_pos = 0;
        }
        Ok(Some(false))
      }
      KeyCode::Char(c) if c.is_ascii_digit() => {
        // Number prefix for commands like 50G
        self.number_prefix.push(c);
        Ok(Some(false))
      }
      _ => Ok(None), // Not handled by control
    }
  }

  // Handle number prefix clearing logic
  pub fn handle_number_prefix_clearing(
    &mut self,
    key_event: crossterm::event::KeyEvent,
  ) {
    // Clear number prefix for non-digit, non-G commands
    let should_clear_prefix = match key_event.code {
      KeyCode::Char(c) if c.is_ascii_digit() => false,
      KeyCode::Char('G') => false,
      _ => true,
    };

    if should_clear_prefix {
      self.number_prefix.clear();
    }
  }

  // Handle tmux-style prefix commands
  pub fn handle_tmux_prefix(
    &mut self,
    key_event: crossterm::event::KeyEvent,
  ) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    self.debug_log(&format!("Handling tmux prefix key: {key_event:?}"));

    // Clear prefix mode after handling
    self.tmux_prefix_active = false;

    match key_event.code {
      KeyCode::Char('j') | KeyCode::Down => {
        // Switch to bottom pane
        if self.view_mode == ViewMode::HorizontalSplit && self.active_pane != 1
        {
          self.debug_log("Switching to bottom pane");
          self.switch_split_pane(1);
        }
        Ok(Some(false))
      }
      KeyCode::Char('k') | KeyCode::Up => {
        // Switch to top pane
        if self.view_mode == ViewMode::HorizontalSplit && self.active_pane != 0
        {
          self.debug_log("Switching to top pane");
          self.switch_split_pane(0);
        }
        Ok(Some(false))
      }
      KeyCode::Char('x') => {
        // Close current pane
        if self.view_mode == ViewMode::HorizontalSplit
          && self.active_buffer == 1
        {
          self.debug_log("Closing split pane");
          self.close_split();
        }
        Ok(Some(false))
      }
      KeyCode::Char('q') => {
        // Show pane numbers (for now just log)
        self.debug_log(&format!("Active pane: {}", self.active_pane));
        Ok(Some(false))
      }
      _ => {
        // Unknown tmux command
        self.debug_log("Unknown tmux prefix command");
        Ok(None)
      }
    }
  }
}
