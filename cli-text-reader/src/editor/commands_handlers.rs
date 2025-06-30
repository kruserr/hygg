use super::core::{Editor, EditorMode};
use crate::config::{save_config, AppConfig};

impl Editor {
  // Handle :p command - toggle progress display
  pub fn handle_progress_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    self.show_progress = !self.show_progress;
    self.save_current_config();
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Handle :cursor command - toggle cursor visibility
  pub fn handle_cursor_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    self.show_cursor = !self.show_cursor;
    self.save_current_config();
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Handle :help command - show help overlay
  pub fn handle_help_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    let help_lines = crate::help::get_help_text();
    self.create_overlay("help", help_lines);
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Handle :tutorial command - show interactive tutorial
  pub fn handle_tutorial_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    self.show_interactive_tutorial()?;
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Handle :tutorial N command - jump to specific tutorial step
  pub fn handle_tutorial_command_with_step(&mut self, step: usize) -> Result<bool, Box<dyn std::error::Error>> {
    self.debug_log(&format!("Starting tutorial at step {step}"));
    
    // Get the number of tutorial steps
    let steps = crate::interactive_tutorial::get_interactive_tutorial_steps();
    let max_step = steps.len();
    
    // Clamp the step to valid range (0-indexed internally, but 1-indexed for users)
    let target_step = if step == 0 {
      0  // Allow :tutorial 0 to go to the first step
    } else if step > max_step {
      max_step - 1  // Go to last step if requested step is too high
    } else {
      step - 1  // Convert from 1-indexed to 0-indexed
    };
    
    // Start the tutorial
    self.show_interactive_tutorial()?;
    
    // Jump to the specified step
    self.tutorial_step = target_step;
    self.tutorial_step_completed = false;
    self.update_tutorial_step_internal(true);
    
    self.debug_log(&format!("Jumped to tutorial step {step} (internal: {target_step})"));
    
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Handle :h command - highlight selected text
  pub fn handle_highlight_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    self.debug_log_event("command", "highlight", "toggling highlight on selection");
    self.toggle_highlight();
    // Track highlight creation for tutorial
    self.tutorial_highlight_created = true;
    self.set_active_mode(EditorMode::Normal);
    self.clear_selection(); // Clear selection after highlighting
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Handle :nohl command - clear search highlights
  pub fn handle_nohl_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    self.debug_log_event("command", "nohlsearch", "clearing search highlights");
    self.editor_state.search_query.clear();
    self.editor_state.current_match = None;
    // Sync with active buffer
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.search_query.clear();
      buffer.current_match = None;
    }
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    self.mark_dirty();
    Ok(false)
  }

  // Handle :next/:continue command for tutorial
  pub fn handle_next_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    if self.tutorial_active {
      let steps = crate::interactive_tutorial::get_interactive_tutorial_steps();
      
      // Special handling for specific steps
      let is_welcome = self.tutorial_step == 0;
      let is_congratulations = self.tutorial_step == steps.len() - 2; // Step before credits
      let is_credits = self.tutorial_step == steps.len() - 1;
      
      if is_credits {
        // From credits screen, complete tutorial
        self.complete_tutorial_interactive();
      } else if is_welcome || is_congratulations || self.tutorial_step_completed {
        // Allow advancement from welcome, congratulations (always), or any completed step
        self.advance_tutorial();
      } else {
        // Step not completed yet
        self.debug_log("Tutorial step not completed yet");
      }
    }
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Handle :back/:prev command for tutorial
  pub fn handle_back_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    if self.tutorial_active {
      self.back_tutorial();
    }
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Save current config settings to file
  pub fn save_current_config(&self) {
    let config = AppConfig {
      enable_tutorial: None, // Keep existing value
      enable_line_highlighter: Some(self.show_highlighter),
      show_cursor: Some(self.show_cursor),
      show_progress: Some(self.show_progress),
      tutorial_shown: None, // Keep existing value
    };

    if let Err(e) = save_config(&config) {
      self.debug_log_error(&format!("Failed to save config: {e}"));
    }
  }

  // Handle :credits/:author command - show credits screen
  pub fn handle_credits_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    // Get the last step (credits) from tutorial
    let steps = crate::interactive_tutorial::get_interactive_tutorial_steps();
    if let Some(credits_step) = steps.last() {
      // Create a simple overlay with just the credits content
      let mut lines = Vec::new();
      
      // Header
      lines.push(format!("━━━ {} ━━━", credits_step.title));
      lines.push("".to_string());
      
      // Filter out any lines that mention ":next"
      for line in &credits_step.instructions {
        if !line.contains(":next") {
          lines.push(line.clone());
        }
      }
      lines.push("".to_string());
      
      // Practice text (additional info)
      lines.extend(credits_step.practice_text.clone());
      lines.push("".to_string());
      
      // Footer
      lines.push("Type :q to close this screen".to_string());
      
      self.create_overlay("credits", lines);
    }
    
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Handle :about command - show welcome screen
  pub fn handle_about_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    // Get the first step (welcome) from tutorial
    let steps = crate::interactive_tutorial::get_interactive_tutorial_steps();
    if let Some(welcome_step) = steps.first() {
      // Create a simple overlay with just the welcome content
      let mut lines = Vec::new();
      
      // Header
      lines.push(format!("━━━ {} ━━━", welcome_step.title));
      lines.push("".to_string());
      
      // Filter out any lines that mention ":next"
      for line in &welcome_step.instructions {
        if !line.contains(":next") {
          lines.push(line.clone());
        }
      }
      lines.push("".to_string());
      
      // Practice text (logo)
      lines.extend(welcome_step.practice_text.clone());
      lines.push("".to_string());
      
      // Footer
      lines.push("Type :q to close this screen".to_string());
      
      self.create_overlay("about", lines);
    }
    
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }
}