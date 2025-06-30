use crossterm::event;

use super::core::Editor;

impl Editor {
  // Handle key events in normal mode - dispatcher to specialized handlers
  pub fn handle_normal_mode_event(
    &mut self,
    key_event: event::KeyEvent,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    self.debug_log_event("normal_mode", "key_press", &format!("{key_event:?}"));

    // Handle number prefix clearing first
    self.handle_number_prefix_clearing(key_event);

    // Handle tmux prefix mode if active
    if self.tmux_prefix_active {
      if let Some(result) = self.handle_tmux_prefix(key_event)? {
        return Ok(result);
      }
    }

    // Try control keys first (mode switching, etc.)
    if let Some(result) = self.handle_control_keys(key_event)? {
      return Ok(result);
    }

    // Try operator pending operations
    if let Some(result) = self.handle_operator_pending(key_event)? {
      return Ok(result);
    }

    // Try search and visual mode operations
    if let Some(result) = self.handle_search_visual_keys(key_event)? {
      return Ok(result);
    }

    // Try navigation operations
    if let Some(result) = self.handle_navigation_keys(key_event)? {
      return Ok(result);
    }

    // If no handler claimed the event, it's unhandled
    Ok(false)
  }
}
