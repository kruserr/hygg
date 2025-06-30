use crossterm::event;
use super::core::Editor;

impl Editor {
  // Handle navigation-related key events in normal mode
  pub fn handle_navigation_keys(
    &mut self,
    key_event: event::KeyEvent,
  ) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    // Try basic movement keys first
    if let Ok(Some(result)) = self.handle_basic_movement_keys(key_event.code) {
      return Ok(Some(result));
    }

    // Try word movement keys
    if let Ok(Some(result)) = self.handle_word_movement_keys(key_event.code) {
      return Ok(Some(result));
    }

    // Try character finding keys
    if let Ok(Some(result)) = self.handle_char_find_keys(key_event.code) {
      return Ok(Some(result));
    }

    // Try page movement keys
    if let Ok(Some(result)) = self.handle_page_movement_keys(key_event.code, key_event.modifiers) {
      return Ok(Some(result));
    }

    // Try jump keys
    if let Ok(Some(result)) = self.handle_jump_keys(key_event.code) {
      return Ok(Some(result));
    }

    // Try text object keys
    if let Ok(Some(result)) = self.handle_text_object_keys(key_event.code) {
      return Ok(Some(result));
    }

    // Try mark keys
    if let Ok(Some(result)) = self.handle_mark_keys(key_event.code) {
      return Ok(Some(result));
    }

    // Not handled by navigation
    Ok(None)
  }
}
