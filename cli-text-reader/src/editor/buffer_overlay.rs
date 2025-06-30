use super::core::{BufferState, Editor, EditorMode, ViewMode};

impl Editor {
  // Create or update the single overlay buffer
  pub fn create_overlay(&mut self, cmd: &str, lines: Vec<String>) {
    self.debug_log("=== create_overlay ===");
    self.debug_log(&format!("  Command: '{}', Lines: {}", cmd, lines.len()));

    // Save current buffer state before switching
    if self.active_buffer < self.buffers.len() {
      self.save_current_buffer_state();
    }

    // Check if we have an overlay buffer (index 1)
    if self.buffers.len() > 1 {
      // Update existing overlay buffer
      self.debug_log("  Updating existing overlay buffer");
      self.buffers[1].lines = lines;
      self.buffers[1].command = Some(cmd.to_string());
      self.buffers[1].offset = 0;
      // Don't reset cursor position - let it remain where user positioned it
      self.total_lines = self.buffers[1].lines.len();
    } else {
      // Create new overlay buffer
      self.debug_log("  Creating new overlay buffer");
      let mut buffer = BufferState::new(lines);
      buffer.command = Some(cmd.to_string());
      buffer.viewport_height = self.height.saturating_sub(1);
      buffer.viewport_start = 0;
      buffer.overlay_level = 1;
      self.buffers.push(buffer);
    }

    // Switch to overlay buffer
    self.active_buffer = 1;
    self.view_mode = ViewMode::Overlay;

    // Load the overlay buffer's state
    self.load_buffer_state(1);

    // Reset to normal mode
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;

    self.debug_log(&format!("  Now showing overlay with command: {cmd}"));
    self.debug_log("=== create_or_update_overlay complete ===");
  }

  // Close overlay and return to main buffer
  pub fn close_overlay(&mut self) -> bool {
    self.debug_log("=== close_overlay ===");

    if self.view_mode != ViewMode::Overlay {
      self.debug_log("  Not in overlay mode, nothing to close");
      return false;
    }

    // Remove overlay buffer and switch back to main buffer
    if self.buffers.len() > 1 {
      self.buffers.pop(); // Remove overlay buffer
    }
    self.active_buffer = 0;
    self.view_mode = ViewMode::Normal;

    // Load main buffer state
    self.load_buffer_state(0);

    self.debug_log("  Returned to main buffer");
    self.debug_log("=== close_overlay complete ===");
    true
  }

  // Check if we can close the current buffer
  pub fn can_close_buffer(&self) -> bool {
    self.buffers.len() > 1
  }
}