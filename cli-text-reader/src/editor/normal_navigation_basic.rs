use crossterm::event::{self, KeyCode, KeyModifiers};
use super::core::Editor;

impl Editor {
  // Handle basic cursor movement keys (hjkl, arrows)
  pub fn handle_basic_movement_keys(
    &mut self,
    key_code: KeyCode,
  ) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    match key_code {
      KeyCode::Char('j') | KeyCode::Down => {
        self.move_cursor_down();
        Ok(Some(false))
      }
      KeyCode::Char('k') | KeyCode::Up => {
        self.move_cursor_up();
        Ok(Some(false))
      }
      KeyCode::Char('h') | KeyCode::Left => {
        self.move_cursor_left();
        Ok(Some(false))
      }
      KeyCode::Char('l') | KeyCode::Right => {
        self.move_cursor_right();
        Ok(Some(false))
      }
      _ => Ok(None),
    }
  }

  // Handle word navigation keys (w/W/b/B/e/E)
  pub fn handle_word_movement_keys(
    &mut self,
    key_code: KeyCode,
  ) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    match key_code {
      KeyCode::Char('w') => {
        // Word forward motion
        self.move_to_word_boundary(true, false);
        Ok(Some(false))
      }
      KeyCode::Char('W') => {
        // Big word forward motion
        self.move_to_word_boundary(true, true);
        Ok(Some(false))
      }
      KeyCode::Char('b') => {
        // Word backward motion
        self.move_to_word_boundary(false, false);
        Ok(Some(false))
      }
      KeyCode::Char('B') => {
        // Big word backward motion
        self.move_to_word_boundary(false, true);
        Ok(Some(false))
      }
      KeyCode::Char('e') => {
        // End of word motion
        let (new_line, new_col) = self.find_word_end(false);
        self.move_to_position(new_line, new_col);
        Ok(Some(false))
      }
      KeyCode::Char('E') => {
        // End of big word motion
        let (new_line, new_col) = self.find_word_end(true);
        self.move_to_position(new_line, new_col);
        Ok(Some(false))
      }
      _ => Ok(None),
    }
  }

  // Handle page navigation keys (Page Up/Down, Ctrl+u/d)
  pub fn handle_page_movement_keys(
    &mut self,
    key_code: KeyCode,
    modifiers: KeyModifiers,
  ) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    match key_code {
      KeyCode::PageDown | KeyCode::PageUp => {
        Ok(self.handle_page_navigation(key_code, modifiers))
      }
      KeyCode::Char('u') | KeyCode::Char('d')
        if modifiers.contains(KeyModifiers::CONTROL) =>
      {
        Ok(self.handle_page_navigation(key_code, modifiers))
      }
      _ => Ok(None),
    }
  }
}