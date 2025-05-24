use crossterm::{
  cursor::{Hide, Show},
  execute,
  style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
  terminal::{self, Clear, ClearType},
};
use std::io::{self, IsTerminal, Result as IoResult, Write};

use super::core::{Editor, EditorMode};
use crate::config::load_config;
use crate::progress::{load_progress, save_progress};

impl Editor {
  pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    let config = load_config();

    self.show_highlighter = config.enable_line_highlighter.unwrap_or(true);

    let show_tutorial = match config.enable_tutorial {
      Some(false) => false,
      _ => self.lines.is_empty() || load_progress(self.document_hash).is_err(),
    };

    if show_tutorial {
      self.show_tutorial(&mut stdout)?;
    }

    // If the file is empty, exit after tutorial
    if self.lines.is_empty() {
      self.cleanup(&mut stdout)?;
      return Ok(());
    }

    self.offset = match load_progress(self.document_hash) {
      Ok(progress) => {
        (progress.percentage / 100.0 * self.total_lines as f64).round() as usize
      }
      Err(_) => 0,
    };

    if std::io::stdout().is_terminal() {
      execute!(stdout, terminal::EnterAlternateScreen, Hide)?;
      terminal::enable_raw_mode()?;
    }

    self.main_loop(&mut stdout)?;

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

  pub fn main_loop(
    &mut self,
    stdout: &mut io::Stdout,
  ) -> Result<(), Box<dyn std::error::Error>> {
    use crate::progress::save_progress;
    use crossterm::event::{self, Event as CEvent};

    loop {
      if std::io::stdout().is_terminal() {
        execute!(stdout, Clear(ClearType::All))?;
      }

      // Center the cursor consistently - this will ensure the
      // cursor stays in the middle with equal lines above and below
      self.center_cursor();

      // Calculate layout parameters
      let term_width = terminal::size()?.0 as u16;
      let center_offset =
        if self.width > self.col { (self.width / 2) - self.col / 2 } else { 0 };
      let center_offset_string = " ".repeat(center_offset);

      // Draw text content with highlighting
      self.draw_content(stdout, term_width, &center_offset_string)?;

      // Show status line and position info
      self.draw_status_line(stdout)?;

      // Flush stdout to ensure all content is displayed
      stdout.flush()?;

      // Position and style the cursor
      self.position_cursor(stdout, center_offset)?;

      // Handle keyboard input
      if std::io::stdout().is_terminal() {
        match event::read()? {
          CEvent::Key(key_event) => {
            let exit = match self.editor_state.mode {
              EditorMode::Normal => self.handle_normal_mode_event(key_event)?,
              EditorMode::VisualChar | EditorMode::VisualLine => {
                self.handle_visual_mode_event(key_event)?
              }
              EditorMode::Search | EditorMode::ReverseSearch => {
                self.handle_search_mode_event(key_event)?
              }
              EditorMode::Command => {
                self.handle_command_mode_event(key_event, stdout)?
              }
            };

            if exit {
              break;
            }
          }
          CEvent::Resize(w, h) => {
            self.width = w as usize;
            self.height = h as usize;
            // Recenter after resize
            self.center_cursor();
          }
          _ => {}
        }
      } else {
        break;
      }

      save_progress(self.document_hash, self.offset, self.total_lines)?;
    }

    Ok(())
  }

  // Draw content with proper highlighting
  fn draw_content(
    &self,
    stdout: &mut io::Stdout,
    term_width: u16,
    center_offset_string: &str,
  ) -> IoResult<()> {
    let content_height = self.height.saturating_sub(1);
    
    for i in 0..content_height {
      execute!(stdout, crossterm::cursor::MoveTo(0, i as u16))?;
      
      // Calculate the actual line index in the document
      let line_idx = self.offset + i;
      
      if line_idx < self.lines.len() {
        // We have a real line to display
        let line = self.lines[line_idx].clone();

        // Highlight the current line first
        let is_current_line =
          self.highlight_current_line(stdout, i, term_width)?;

        // Try highlighting selection
        if self.highlight_selection(stdout, i, &line, center_offset_string)? {
          continue;
        }

        // Try highlighting search match
        if self.highlight_search_match(stdout, i, &line, center_offset_string)? {
          continue;
        }

        // Normal line rendering - if current line was highlighted,
        // we need to use appropriate text color
        if is_current_line {
          // For the highlighted line, use a color that contrasts with the background
          execute!(
            stdout,
            crossterm::style::SetForegroundColor(crossterm::style::Color::White)
          )?;
          println!("{}{}", center_offset_string, line);
          execute!(stdout, crossterm::style::ResetColor)?;
        } else {
          println!("{}{}", center_offset_string, line);
        }
      } else {
        // This is beyond the document - show blank line for overscroll
        // But still check if we need to highlight the cursor line
        let is_current_line =
          self.highlight_current_line(stdout, i, term_width)?;
          
        if is_current_line {
          // Show highlighted empty line for cursor position
          execute!(
            stdout,
            crossterm::style::SetForegroundColor(crossterm::style::Color::White)
          )?;
          println!("{}", center_offset_string);
          execute!(stdout, crossterm::style::ResetColor)?;
        } else {
          // Just show blank line
          println!("{}", center_offset_string);
        }
      }
    }

    // Reset highlighting at the end of each frame
    if self.show_highlighter {
      execute!(stdout, SetBackgroundColor(crossterm::style::Color::Reset))?;
    }

    Ok(())
  }
}
