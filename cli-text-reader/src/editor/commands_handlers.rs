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
      self.debug_log(&format!("handle_next_command: tutorial_step={}, buffers={}, step_completed={}", 
        self.tutorial_step, self.buffers.len(), self.tutorial_step_completed));
      
      // Enhanced buffer validation with detailed logging
      if self.buffers.is_empty() {
        self.debug_log("ERROR: No buffers available during tutorial next command");
        self.complete_tutorial_interactive();
        return Ok(false);
      }
      
      // Validate we have at least 2 buffers (main + overlay) for tutorial
      if self.buffers.len() < 2 {
        self.debug_log(&format!("WARNING: Only {} buffers, expected at least 2 for tutorial", self.buffers.len()));
      }
      
      // Log buffer states for debugging
      for (i, buffer) in self.buffers.iter().enumerate() {
        self.debug_log(&format!("  Buffer {}: lines={}, command={:?}, overlay_level={}", 
          i, buffer.lines.len(), buffer.command, buffer.overlay_level));
      }
      
      let steps = crate::interactive_tutorial::get_interactive_tutorial_steps();
      
      // Validate tutorial step is within bounds
      if self.tutorial_step >= steps.len() {
        self.debug_log(&format!("ERROR: Tutorial step {} out of bounds (max: {})", self.tutorial_step, steps.len() - 1));
        self.complete_tutorial_interactive();
        return Ok(false);
      }
      
      // Special handling for specific steps with enhanced logging
      let is_welcome = self.tutorial_step == 0;
      let is_congratulations = self.tutorial_step == steps.len() - 2; // Step before credits
      let is_credits = self.tutorial_step == steps.len() - 1;
      
      self.debug_log(&format!("  Step type: welcome={}, congratulations={}, credits={}", 
        is_welcome, is_congratulations, is_credits));
      
      // Special handling for step 3 (Text Objects - Paragraph Selection)
      if self.tutorial_step == 3 {
        self.debug_log("  Special handling for step 3 - ensuring state is saved");
        // Force save state before advancing from step 3
        self.save_current_buffer_state();
      }
      
      if is_credits {
        // From credits screen, complete tutorial
        self.debug_log("  Completing tutorial from credits screen");
        self.complete_tutorial_interactive();
      } else if is_welcome || is_congratulations || self.tutorial_step_completed {
        // Allow advancement from welcome, congratulations (always), or any completed step
        self.debug_log(&format!("  Advancing tutorial (welcome: {}, congrats: {}, completed: {})", 
          is_welcome, is_congratulations, self.tutorial_step_completed));
        
        // Extra validation before advancing
        if self.active_buffer >= self.buffers.len() {
          self.debug_log(&format!("WARNING: Active buffer {} out of range, resetting", self.active_buffer));
          self.active_buffer = self.buffers.len().saturating_sub(1);
        }
        
        self.advance_tutorial();
      } else {
        // Step not completed yet
        self.debug_log("  Tutorial step not completed yet, cannot advance");
      }
    } else {
      self.debug_log("  Not in tutorial mode, ignoring :next command");
    }
    
    // Clear command state consistently
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    
    // Clear command buffer in active buffer if it exists
    if self.active_buffer < self.buffers.len() {
      if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
        buffer.command_buffer.clear();
        buffer.command_cursor_pos = 0;
      }
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

  // Handle :notutorial command - permanently disable tutorial
  pub fn handle_notutorial_command(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
    self.debug_log("Handling :notutorial command");
    
    // Disable tutorial in config
    let config = AppConfig {
      enable_tutorial: Some(false),
      enable_line_highlighter: None, // Keep existing value
      show_cursor: None, // Keep existing value
      show_progress: None, // Keep existing value
      tutorial_shown: None, // Keep existing value
    };
    
    if let Err(e) = save_config(&config) {
      self.debug_log_error(&format!("Failed to save config: {e}"));
    }
    
    // If we're currently in tutorial, exit it
    if self.tutorial_active {
      self.debug_log("Exiting tutorial due to :notutorial command");
      self.complete_tutorial_interactive();
    }
    
    // Show confirmation message
    let message = vec![
      "".to_string(),
      "Tutorial disabled permanently.".to_string(),
      "".to_string(),
      "Use :tutorial on to re-enable.".to_string(),
      "".to_string(),
      "Press :q to close this message.".to_string(),
    ];
    self.create_overlay("notification", message);
    
    self.set_active_mode(EditorMode::Normal);
    self.editor_state.command_buffer.clear();
    self.editor_state.command_cursor_pos = 0;
    if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
      buffer.command_buffer.clear();
      buffer.command_cursor_pos = 0;
    }
    Ok(false)
  }

  // Handle :tutorial on/off command - toggle tutorial
  pub fn handle_tutorial_toggle_command(&mut self, enable: bool) -> Result<bool, Box<dyn std::error::Error>> {
    self.debug_log(&format!("Handling :tutorial {} command", if enable { "on" } else { "off" }));
    
    // Update tutorial setting in config
    let config = AppConfig {
      enable_tutorial: Some(enable),
      enable_line_highlighter: None, // Keep existing value
      show_cursor: None, // Keep existing value
      show_progress: None, // Keep existing value
      tutorial_shown: None, // Keep existing value
    };
    
    if let Err(e) = save_config(&config) {
      self.debug_log_error(&format!("Failed to save config: {e}"));
    }
    
    // If disabling and currently in tutorial, exit it
    if !enable && self.tutorial_active {
      self.debug_log("Exiting tutorial due to :tutorial off command");
      self.complete_tutorial_interactive();
    }
    
    // Show confirmation message
    let message = vec![
      "".to_string(),
      format!("Tutorial {} for next launch.", if enable { "enabled" } else { "disabled" }),
      "".to_string(),
      if enable {
        "The tutorial will show on next startup if not already completed.".to_string()
      } else {
        "Use :tutorial on to re-enable.".to_string()
      },
      "".to_string(),
      "Press :q to close this message.".to_string(),
    ];
    self.create_overlay("notification", message);
    
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