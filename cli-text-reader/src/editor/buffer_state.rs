use super::core::{BufferState, Editor};

impl Editor {
  // Save current editor state to the active buffer
  pub fn save_current_buffer_state(&mut self) {
    let active_idx = self.active_buffer;
    if let Some(buffer) = self.buffers.get_mut(active_idx) {
      // Save position and display state
      buffer.offset = self.offset;
      buffer.cursor_x = self.cursor_x;
      buffer.cursor_y = self.cursor_y;

      // Save search state
      buffer.search_query = self.editor_state.search_query.clone();
      buffer.current_match = self.editor_state.current_match;

      // Save selection state
      buffer.selection_start = self.editor_state.selection_start;
      buffer.selection_end = self.editor_state.selection_end;

      // Save mode and command state
      buffer.mode = self.editor_state.mode.clone();
      buffer.command_buffer = self.editor_state.command_buffer.clone();
      buffer.command_cursor_pos = self.editor_state.command_cursor_pos;

      // Extract values for logging to avoid borrow checker issues
      let offset = buffer.offset;
      let cursor_x = buffer.cursor_x;
      let cursor_y = buffer.cursor_y;
      let mode = buffer.mode.clone();
      let command_buffer = buffer.command_buffer.clone();

      self.debug_log(&format!(
        "Saved buffer {active_idx} state: offset={offset}, cursor=({cursor_x},{cursor_y}), mode={mode:?}, cmd_buf='{command_buffer}'"
      ));
    } else {
      self.debug_log(&format!(
        "ERROR: Cannot save state for buffer {active_idx}"
      ));
    }
  }

  // Load buffer state into the editor
  pub fn load_buffer_state(&mut self, buffer_idx: usize) {
    self.debug_log(&format!("Loading buffer {buffer_idx} state"));

    if let Some(buffer) = self.buffers.get(buffer_idx) {
      // Load document content
      self.lines = buffer.lines.clone();
      self.total_lines = buffer.lines.len();

      // Load position and display state
      self.offset = buffer.offset;
      self.cursor_x = buffer.cursor_x;
      self.cursor_y = buffer.cursor_y;

      // Load search state
      self.editor_state.search_query = buffer.search_query.clone();
      self.editor_state.current_match = buffer.current_match;

      // Load selection state
      self.editor_state.selection_start = buffer.selection_start;
      self.editor_state.selection_end = buffer.selection_end;

      // Load mode and command state
      self.editor_state.mode = buffer.mode.clone();
      self.editor_state.command_buffer = buffer.command_buffer.clone();
      self.editor_state.command_cursor_pos = buffer.command_cursor_pos;

      self.debug_log(&format!(
        "Loaded buffer {} state: lines={}, offset={}, cursor=({},{}), mode={:?}, cmd_buf='{}'",
        buffer_idx, self.lines.len(), self.offset, self.cursor_x, self.cursor_y,
        self.editor_state.mode, self.editor_state.command_buffer
      ));
    } else {
      self.debug_log(&format!("ERROR: Buffer {buffer_idx} not found!"));
    }
  }

  // Get the active buffer
  #[allow(dead_code)]
  pub fn get_active_buffer(&self) -> Option<&BufferState> {
    self.buffers.get(self.active_buffer)
  }

  // Get the active buffer mutably
  #[allow(dead_code)]  
  pub fn get_active_buffer_mut(&mut self) -> Option<&mut BufferState> {
    self.buffers.get_mut(self.active_buffer)
  }

  // Center cursor within the active buffer's viewport
  #[allow(dead_code)]
  pub fn center_cursor_in_buffer(&mut self) {
    if let Some(buffer) = self.get_active_buffer() {
      let viewport_height = buffer.viewport_height;
      let center = viewport_height / 2;
      let current_line = self.offset + self.cursor_y;

      // Calculate new offset for centering within viewport
      let new_offset = current_line.saturating_sub(center);
      let max_offset = self.total_lines.saturating_sub(viewport_height);
      self.offset = new_offset.min(max_offset);

      // Update cursor_y relative to new offset, ensuring it stays within
      // viewport
      self.cursor_y = if current_line >= self.offset {
        let relative_pos = current_line - self.offset;
        // Ensure cursor_y doesn't exceed viewport bounds
        relative_pos.min(viewport_height.saturating_sub(1))
      } else {
        0
      };

      self.debug_log(&format!(
        "Centered cursor in buffer: line={}, offset={}, cursor_y={}, viewport_height={}",
        current_line, self.offset, self.cursor_y, viewport_height
      ));
    }
  }
}