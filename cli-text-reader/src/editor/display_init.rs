use crossterm::{
  cursor::{Hide, Show},
  execute,
  terminal::{self, Clear, ClearType},
};
use std::io::{self, IsTerminal, Result as IoResult};

use super::core::{Editor, EditorMode, ViewMode};
use crate::bookmarks::load_bookmarks;
use crate::config::load_config;
use crate::highlights::load_highlights;
use crate::progress::{load_progress, save_progress};

impl Editor {
  pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    let config = load_config();

    self.show_highlighter = config.enable_line_highlighter.unwrap_or(true);
    self.show_cursor = config.show_cursor.unwrap_or(true);
    self.show_progress = config.show_progress.unwrap_or(false);
    
    // Check if tutorial should be shown
    let tutorial_enabled = config.enable_tutorial.unwrap_or(true);
    let tutorial_shown = config.tutorial_shown.unwrap_or(false);

    // Load bookmarks
    if let Ok(bookmark_data) = load_bookmarks(self.document_hash) {
      self.marks = bookmark_data.marks;
    }

    // Load highlights
    match load_highlights(&self.document_hash.to_string()) {
      Ok(highlight_data) => {
        self.highlights = highlight_data;
        self.debug_log(&format!(
          "Loaded {} highlights",
          self.highlights.highlights.len()
        ));
      }
      Err(e) => {
        self.debug_log_error(&format!("Failed to load highlights: {e}"));
      }
    }

    // Tutorial will be shown automatically on first launch if enabled
    
    // Note: Even with empty lines, we should allow the editor to run
    // so users can access the tutorial with :tutorial command

    let mut skip_first_center = false;
    self.offset = match load_progress(self.document_hash) {
      Ok(progress) => {
        // When loading saved progress, skip the first center_cursor call
        // to preserve the exact saved position
        skip_first_center = true;
        progress.offset
      }
      Err(_) => 0,
    };

    if std::io::stdout().is_terminal() {
      execute!(stdout, terminal::EnterAlternateScreen, Hide)?;
      terminal::enable_raw_mode()?;
    }
    
    // Show tutorial on first launch or start demo mode
    if self.tutorial_demo_mode {
      self.debug_log("Starting marketing demo mode");
      self.start_demo_mode();
    } else if tutorial_enabled && !tutorial_shown && !self.tutorial_demo_mode {
      self.debug_log("Showing interactive tutorial for first-time user");
      self.show_interactive_tutorial()?;
    }

    self.main_loop(&mut stdout, skip_first_center)?;

    self.cleanup(&mut stdout)?;
    Ok(())
  }

  pub fn cleanup(
    &self,
    stdout: &mut io::Stdout,
  ) -> Result<(), Box<dyn std::error::Error>> {
    if std::io::stdout().is_terminal() {
      execute!(stdout, Show, terminal::LeaveAlternateScreen)?;
      terminal::disable_raw_mode()?;
    }
    Ok(())
  }
}