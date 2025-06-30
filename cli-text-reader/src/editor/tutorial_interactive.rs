use super::core::{Editor, EditorMode};
use crate::interactive_tutorial::{
    get_interactive_tutorial_steps, create_tutorial_buffer
};
use crate::interactive_tutorial_buffer::TutorialSuccessCondition;
use crate::config::{save_config, AppConfig};
use crossterm::event::KeyCode;

impl Editor {
  // Show the interactive tutorial
  pub fn show_interactive_tutorial(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    self.debug_log("Starting simple interactive tutorial");
    
    // Save current cursor position
    self.previous_position = Some((self.offset + self.cursor_y, self.cursor_x));
    
    // Clear all highlights for the tutorial
    self.highlights.clear_all_highlights();
    
    // Set tutorial mode
    self.tutorial_active = true;
    self.tutorial_step = 0;
    
    // Show the first step (with is_new_step=true to clear any existing state)
    self.update_tutorial_step_internal(true);
    
    Ok(())
  }
  
  // Update tutorial display for current step
  pub fn update_tutorial_step(&mut self) {
    self.update_tutorial_step_internal(false);
  }
  
  // Internal method that can optionally preserve state
  pub fn update_tutorial_step_internal(&mut self, is_new_step: bool) {
    let steps = get_interactive_tutorial_steps();
    if self.tutorial_step >= steps.len() {
      self.complete_tutorial_interactive();
      return;
    }
    
    let step = &steps[self.tutorial_step];
    // Leave space for command line
    let available_height = self.height.saturating_sub(1);
    let buffer_lines = create_tutorial_buffer(
      step,
      self.tutorial_step,
      steps.len(),
      self.width,
      self.tutorial_step_completed
    );
    
    // Ensure buffer doesn't exceed available height
    // Reserve extra space to ensure command line is always visible
    let safe_height = available_height.saturating_sub(2);
    let mut truncated_lines = buffer_lines;
    if truncated_lines.len() > safe_height {
      truncated_lines.truncate(safe_height);
    }
    
    // Create overlay with the tutorial content
    self.create_overlay("tutorial", truncated_lines);
    
    // Store current step's success condition
    self.current_tutorial_condition = Some(step.success_check.clone());
    
    // Only reset state if we're advancing to a new step
    if is_new_step {
      // Reset success flags to prevent bleed-through from previous steps
      self.tutorial_highlight_created = false;
      self.tutorial_yank_performed = false;
      self.tutorial_paste_performed = false;
      self.tutorial_search_navigated = false;
      self.tutorial_bookmark_jumped = false;
      self.tutorial_forward_search_used = false;
      self.tutorial_backward_search_used = false;
      self.last_executed_command = None;
      
      // Clear search highlights to prevent bleed-through
      self.editor_state.search_query.clear();
      self.editor_state.current_match = None;
      
      // Sync search state with active buffer
      if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
        buffer.search_query.clear();
        buffer.current_match = None;
      }
    }
    
    self.mark_dirty();
  }
  
  
  // Handle tutorial progression
  pub fn advance_tutorial(&mut self) {
    let steps = get_interactive_tutorial_steps();
    self.debug_log(&format!("Advancing tutorial from step {} (total steps: {})", self.tutorial_step, steps.len()));
    
    // Safety check to prevent out of bounds
    if self.tutorial_step >= steps.len() {
      self.debug_log("Tutorial step out of bounds, completing tutorial");
      self.complete_tutorial_interactive();
      return;
    }
    
    if self.tutorial_step < steps.len() - 1 {
      // Clear highlights when advancing FROM the highlighting step (step 3)
      // This ensures highlights don't carry over to subsequent steps
      if self.tutorial_step == 3 {
        self.highlights.clear_all_highlights();
      }
      
      self.tutorial_step += 1;
      
      // Reset completion flag for the new step
      self.tutorial_step_completed = false;
      // Update with is_new_step=true to clear state from previous step
      self.update_tutorial_step_internal(true);
    } else {
      self.complete_tutorial_interactive();
    }
  }
  
  // Go back to previous tutorial step
  pub fn back_tutorial(&mut self) {
    self.debug_log(&format!("Going back from tutorial step {}", self.tutorial_step));
    
    if self.tutorial_step > 0 {
      self.tutorial_step -= 1;
      
      // Reset completion flag for the step we're going back to
      self.tutorial_step_completed = false;
      // Clear any highlights when going back
      self.highlights.clear_all_highlights();
      // Update with is_new_step=true to reset state
      self.update_tutorial_step_internal(true);
    }
  }
  
  // Complete the tutorial
  pub fn complete_tutorial_interactive(&mut self) {
    self.debug_log("Completing interactive tutorial");
    self.debug_log(&format!("Current buffers: {}, active: {}", self.buffers.len(), self.active_buffer));
    
    // Reset flags
    self.tutorial_active = false;
    self.current_tutorial_condition = None;
    self.tutorial_step_completed = false;
    
    // Save config
    let config = AppConfig {
      enable_tutorial: None,
      enable_line_highlighter: None,
      show_cursor: None,
      show_progress: None,
      tutorial_shown: Some(true),
    };
    
    if let Err(e) = save_config(&config) {
      self.debug_log_error(&format!("Failed to save tutorial state: {e}"));
    }
    
    // Close overlay and return to normal mode with original document
    self.debug_log("Closing tutorial overlay, returning to original document");
    self.close_overlay();
    self.set_active_mode(EditorMode::Normal);
    
    // Restore cursor position
    if let Some((line, col)) = self.previous_position {
      self.debug_log(&format!("Restoring cursor position to line {line}, col {col}"));
      if line < self.lines.len() {
        self.offset = line.saturating_sub(self.height / 2);
        self.cursor_y = line.saturating_sub(self.offset).min(self.height - 2);
        self.cursor_x = col;
      }
    }
    
    self.debug_log(&format!("After close: buffers: {}, active: {}", self.buffers.len(), self.active_buffer));
  }
  
  // Process keys during tutorial - let normal editor handle most things
  pub fn process_tutorial_key(&mut self, key: KeyCode) -> bool {
    self.debug_log(&format!("Tutorial processing key: {:?}, step_completed: {}", key, self.tutorial_step_completed));
    
    // Tutorial can only be exited with :q command, not just 'q'
    // This ensures users learn the proper command mode
    
    // If step is completed, allow all normal movement but show the :next hint
    if self.tutorial_step_completed {
      // Just allow normal key processing - don't restrict movement
      return false;
    }
    
    // Check for specific key presses if that's what we're waiting for
    if let Some(TutorialSuccessCondition::KeyPress(expected)) = &self.current_tutorial_condition {
      if !self.tutorial_step_completed && match key {
        KeyCode::Char(c) => c.to_string() == *expected,
        KeyCode::Down => expected == "j" || expected == "Down",
        KeyCode::Up => expected == "k" || expected == "Up",
        _ => false,
      } {
        // Mark step as completed but don't advance
        self.tutorial_step_completed = true;
        // Update the display to show the ":next" hint
        self.update_tutorial_step();
        // Don't return early - let the key be processed for movement
      }
    }
    
    // For final step with NoCondition, allow :next to return to document
    if let Some(TutorialSuccessCondition::NoCondition) = &self.current_tutorial_condition {
      // Don't handle Enter here, let them use :next command
      return false;
    }
    
    // After any other action, check if the success condition is met
    if !self.tutorial_step_completed && self.check_tutorial_completion() {
      // Mark as completed and update display to show ":next" hint
      self.tutorial_step_completed = true;
      self.update_tutorial_step();
    }
    
    // Always return false to allow normal key processing for movement
    false
  }
}