use crossterm::terminal;
use std::io;

use super::core::{Editor, ViewMode};
use crate::config::{save_config, AppConfig};

impl Editor {
  pub fn execute_command(
    &mut self,
    _stdout: &mut io::Stdout,
  ) -> Result<bool, Box<dyn std::error::Error>> {
    let cmd = self.get_active_command_buffer().trim().to_string();
    self.debug_log_event("command", "execute_command", &format!("cmd='{cmd}'"));
    
    // Track command for tutorial will be done in specific command handlers
    self.debug_log_state(
      "command",
      "buffers_count",
      &self.buffers.len().to_string(),
    );
    self.debug_log_state(
      "command",
      "active_buffer",
      &self.active_buffer.to_string(),
    );
    self.debug_log_state(
      "command",
      "view_mode",
      &format!("{:?}", self.view_mode),
    );

    // Handle :q, :q!, :quit, :exit commands
    if cmd == "q" || cmd == "q!" || cmd == "quit" || cmd == "exit" {
      // Check if we're in horizontal split view
      if self.view_mode == ViewMode::HorizontalSplit
      {
        // In split view, :q closes the split from either pane
        self.debug_log_event(
          "command",
          "quit_split",
          &format!("closing horizontal split from buffer {}", self.active_buffer),
        );
        
        // Check if we're in tutorial mode - if so, return to tutorial overlay
        if self.tutorial_active {
          self.close_split();
          // Restore tutorial overlay
          self.update_tutorial_step();
        } else {
          self.close_split();
        }
        
        self.set_active_mode(super::core::EditorMode::Normal);
        self.editor_state.command_buffer.clear();
        self.editor_state.command_cursor_pos = 0;
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
          buffer.command_buffer.clear();
          buffer.command_cursor_pos = 0;
        }
        return Ok(false);
      } else if self.can_close_buffer() {
        // In overlay view, :q closes the overlay
        self.debug_log_event(
          "command",
          "quit_overlay",
          "closing overlay buffer",
        );
        
        // Check if we're closing the tutorial overlay
        if self.tutorial_active && self.active_buffer == 1 {
          self.debug_log_event(
            "command",
            "quit_tutorial",
            "properly completing tutorial on :q",
          );
          self.complete_tutorial_interactive();
        } else {
          self.close_overlay();
        }
        
        self.set_active_mode(super::core::EditorMode::Normal);
        self.editor_state.command_buffer.clear();
        self.editor_state.command_cursor_pos = 0;
        if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
          buffer.command_buffer.clear();
          buffer.command_cursor_pos = 0;
        }
        return Ok(false);
      } else {
        // In main buffer, :q exits the editor
        self.debug_log_event(
          "command",
          "quit_editor",
          "exiting from main buffer",
        );
        return Ok(true);
      }
    }

    // Handle command execution
    if let Some(shell_cmd) = cmd.strip_prefix('!') {
      // Execute shell command
      let shell_cmd = shell_cmd.to_string();
      self.debug_log_event(
        "command",
        "shell_command",
        &format!("cmd='{}', from_buffer={}", shell_cmd, self.active_buffer),
      );
      self.debug_log_state(
        "command",
        "mode_before_exec",
        &format!("{:?}", self.editor_state.mode),
      );

      // Check if we're in tutorial mode - if so, handle shell commands differently
      if self.tutorial_active {
        // For tutorial, show command output in overlay instead of split
        self.execute_shell_command_in_tutorial(&shell_cmd)?;
      } else {
        self.execute_shell_command(&shell_cmd)?;
      }

      self.debug_log(&format!(
        "After execute_shell_command - buffers: {}, active: {}, mode: {:?}",
        self.buffers.len(),
        self.active_buffer,
        self.view_mode
      ));
      self
        .debug_log(&format!("  Lines in active buffer: {}", self.lines.len()));

      // Ensure cursor is within bounds after command execution
      let viewport_height = self.height.saturating_sub(1);
      if self.cursor_y >= viewport_height {
        let old_y = self.cursor_y;
        self.cursor_y = viewport_height.saturating_sub(1);
        self.debug_log(&format!(
          "Adjusted cursor_y from {} to {} (viewport_height={})",
          old_y, self.cursor_y, viewport_height
        ));
      }

      self.set_active_mode(super::core::EditorMode::Normal);
      self.editor_state.command_buffer.clear();
      self.editor_state.command_cursor_pos = 0;
      if let Some(buffer) = self.buffers.get_mut(self.active_buffer) {
        buffer.command_buffer.clear();
        buffer.command_cursor_pos = 0;
      }
      self.debug_log("Command execution complete, mode set to Normal");
      return Ok(false);
    }

    match cmd.as_str() {
      "p" => self.handle_progress_command(),
      "cursor" | "c" => self.handle_cursor_command(),
      "help" | "commands" => self.handle_help_command(),
      "notutorial" => self.handle_notutorial_command(),
      cmd if cmd.starts_with("tutorial") => {
        // Check if there's a parameter after "tutorial"
        if let Some(param_str) = cmd.strip_prefix("tutorial").map(|s| s.trim()) {
          if param_str == "on" {
            self.handle_tutorial_toggle_command(true)
          } else if param_str == "off" {
            self.handle_tutorial_toggle_command(false)
          } else if !param_str.is_empty() {
            // Try to parse as step number
            if let Ok(step_num) = param_str.parse::<usize>() {
              self.handle_tutorial_command_with_step(step_num)
            } else {
              self.handle_tutorial_command()
            }
          } else {
            self.handle_tutorial_command()
          }
        } else {
          self.handle_tutorial_command()
        }
      }
      "next" | "continue" => self.handle_next_command(),
      "back" | "prev" | "previous" => self.handle_back_command(),
      "h" => self.handle_highlight_command(),
      "nohl" | "nohlsearch" => self.handle_nohl_command(),
      "credits" | "author" => self.handle_credits_command(),
      "about" => self.handle_about_command(),
      _ => {
        let result = handle_command(&cmd, &mut self.show_highlighter);
        if cmd == "z" {
          self.save_current_config();
        }
        Ok(result)
      }
    }
  }


}

// Handle Vim-style commands
pub fn handle_command(command: &str, show_highlighter: &mut bool) -> bool {
  match command.trim() {
    "q" => true,
    "z" => {
      *show_highlighter = !*show_highlighter;
      false
    }
    "p" | "help" | "tutorial" => false,
    _ => false,
  }
}
