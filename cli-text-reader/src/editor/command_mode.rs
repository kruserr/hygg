use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers};
use std::io;

use super::core::{Editor, EditorMode};

impl Editor {
  // Handle key events in command mode
  pub fn handle_command_mode_event(
    &mut self,
    key_event: event::KeyEvent,
    stdout: &mut io::Stdout,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    self.debug_log(&format!("handle_command_mode_event: key={key_event:?}"));
    self.debug_log(&format!(
      "  Command buffer: '{}', cursor_pos: {}",
      self.editor_state.command_buffer, self.editor_state.command_cursor_pos
    ));

    // Handle Ctrl+C to exit to normal mode
    if key_event.code == KeyCode::Char('c')
      && key_event.modifiers.contains(KeyModifiers::CONTROL)
    {
      self.debug_log("  Ctrl+C pressed, exiting to Normal mode");
      self.set_active_mode(EditorMode::Normal);
      self.editor_state.command_buffer.clear();
      self.editor_state.command_cursor_pos = 0;
      self.editor_state.visual_selection_active = false;
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
        self.editor_state.command_cursor_pos = 0;
        self.editor_state.visual_selection_active = false;
        self.editor_state.previous_visual_mode = None;
        // Clear all buffers' command state
        for buffer in &mut self.buffers {
          buffer.command_buffer.clear();
          buffer.command_cursor_pos = 0;
          buffer.mode = EditorMode::Normal;
        }
        self.mark_dirty(); // Force redraw to clear command line
      }
      KeyCode::Enter => {
        self.debug_log("  Enter pressed, executing command");
        let should_exit = self.execute_command(stdout)?;
        self.debug_log(&format!("  execute_command returned: {should_exit}"));
        if should_exit {
          return Ok(true);
        }
        // Ensure we're not leaving any command state behind
        self.debug_log("  Setting mode to Normal after command execution");
        self.set_active_mode(EditorMode::Normal);
        self.editor_state.command_buffer.clear();
        self.editor_state.command_cursor_pos = 0;
        // Clear all buffers' command state to ensure no trailing colon
        for buffer in &mut self.buffers {
          buffer.command_buffer.clear();
          buffer.command_cursor_pos = 0;
          buffer.mode = EditorMode::Normal;
        }
        self.mark_dirty(); // Force redraw to clear command line
      }
      KeyCode::Backspace => {
        if self.editor_state.command_cursor_pos > 0 {
          let pos = self.editor_state.command_cursor_pos;
          
          // Just remove the character
          self.editor_state.command_buffer.remove(pos - 1);
          self.editor_state.command_cursor_pos -= 1;

          // Also update active buffer's command buffer
          if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
            buffer.command_buffer.remove(pos - 1);
            buffer.command_cursor_pos = self.editor_state.command_cursor_pos;
          }
        } else if self.editor_state.command_buffer.is_empty() {
          // If we're at position 0 and buffer is empty, we're trying to delete the ':'
          // Return to normal mode
          self.set_active_mode(EditorMode::Normal);
          self.editor_state.command_cursor_pos = 0;
          self.editor_state.visual_selection_active = false;
          if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
            buffer.command_buffer.clear();
            buffer.command_cursor_pos = 0;
          }
          self.mark_dirty();
        }
      }
      KeyCode::Delete => {
        if self.editor_state.command_cursor_pos
          < self.editor_state.command_buffer.len()
        {
          let pos = self.editor_state.command_cursor_pos;
          self.editor_state.command_buffer.remove(pos);
        }
      }
      KeyCode::Left => {
        if self.editor_state.command_cursor_pos > 0 {
          self.editor_state.command_cursor_pos -= 1;
          // Also update active buffer's command cursor position
          if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
            buffer.command_cursor_pos = self.editor_state.command_cursor_pos;
          }
        }
      }
      KeyCode::Right => {
        if self.editor_state.command_cursor_pos
          < self.editor_state.command_buffer.len()
        {
          self.editor_state.command_cursor_pos += 1;
          // Also update active buffer's command cursor position
          if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
            buffer.command_cursor_pos = self.editor_state.command_cursor_pos;
          }
        }
      }
      KeyCode::Home => {
        self.editor_state.command_cursor_pos = 0;
        // Also update active buffer's command cursor position
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
          buffer.command_cursor_pos = 0;
        }
      }
      KeyCode::End => {
        self.editor_state.command_cursor_pos =
          self.editor_state.command_buffer.len();
        // Also update active buffer's command cursor position
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
          buffer.command_cursor_pos = self.editor_state.command_cursor_pos;
        }
      }
      KeyCode::Char('r')
        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
      {
        // Ctrl+R in command mode - paste from register
        if let CEvent::Key(register_key) = event::read()? {
          if let KeyCode::Char('0') = register_key.code {
            // Paste from yank buffer (register 0) at cursor position
            let pos = self.editor_state.command_cursor_pos;
            let yank_text = self.editor_state.yank_buffer.clone();
            self.debug_log_event(
              "command_mode",
              "paste_register_0",
              &format!("yank_buffer='{yank_text}', pos={pos}"),
            );

            // Remove newlines from yanked text to prevent command execution
            let clean_text = yank_text.replace('\n', " ").replace('\r', "");
            self.debug_log_state("command_mode", "clean_text", &clean_text);

            self.editor_state.command_buffer.insert_str(pos, &clean_text);
            self.editor_state.command_cursor_pos += clean_text.len();

            // Also update active buffer's command buffer
            if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
              buffer.command_buffer.insert_str(pos, &clean_text);
              buffer.command_cursor_pos = self.editor_state.command_cursor_pos;
            }
            
            // Track paste for tutorial
            if self.tutorial_active {
              self.tutorial_paste_performed = true;
              self.debug_log("Tutorial: paste performed via Ctrl+R 0");
            }
            
            self.debug_log_state(
              "command_mode",
              "new_command_buffer",
              &self.editor_state.command_buffer,
            );
          }
        }
        // Other registers not implemented yet
      }
      KeyCode::Char('v')
        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
      {
        // Ctrl+V in command mode - paste from system clipboard
        if let Some(clipboard) = &mut self.clipboard {
          if let Ok(clipboard_text) = clipboard.get_text() {
            let pos = self.editor_state.command_cursor_pos;
            // Remove newlines from clipboard text to prevent command execution
            let clean_text =
              clipboard_text.replace('\n', " ").replace('\r', "");
            self.editor_state.command_buffer.insert_str(pos, &clean_text);
            self.editor_state.command_cursor_pos += clean_text.len();

            // Also update active buffer's command buffer
            if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
              buffer.command_buffer.insert_str(pos, &clean_text);
              buffer.command_cursor_pos = self.editor_state.command_cursor_pos;
            }
            
            // Track paste for tutorial
            if self.tutorial_active {
              self.tutorial_paste_performed = true;
              self.debug_log("Tutorial: paste performed via Ctrl+V");
            }
            
            self
              .debug_log(&format!("Pasted from clipboard: '{clipboard_text}'"));
          }
        }
      }
      KeyCode::Char(c) => {
        // Check for '!' at start of command to enter CommandExecution mode
        if c == '!'
          && self.editor_state.command_buffer.is_empty()
          && self.get_active_mode() == EditorMode::Command
        {
          self.set_active_mode(EditorMode::CommandExecution);
          self.debug_log("Entering CommandExecution mode");
        }
        let pos = self.editor_state.command_cursor_pos;
        self.editor_state.command_buffer.insert(pos, c);
        self.editor_state.command_cursor_pos += 1;

        // Also update active buffer's command buffer
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
          buffer.command_buffer.insert(pos, c);
          buffer.command_cursor_pos = self.editor_state.command_cursor_pos;
        }

        self.debug_log(&format!(
          "  Added '{}' at position {}, buffer='{}'",
          c, pos, self.editor_state.command_buffer
        ));
      }
      _ => {
        self.debug_log(&format!(
          "  Unhandled key in command mode: {:?}",
          key_event.code
        ));
      }
    }
    self.debug_log(&format!(
      "  Command mode event complete, mode={:?}",
      self.editor_state.mode
    ));
    Ok(false)
  }
}
