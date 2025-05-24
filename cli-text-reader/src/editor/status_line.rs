use crossterm::{cursor::MoveTo, execute};
use std::io::{self, Write};

use super::core::{Editor, EditorMode};

impl Editor {
  // Draw the status line with mode indicators and position info
  pub fn draw_status_line(
    &mut self,
    stdout: &mut io::Stdout,
  ) -> io::Result<()> {
    // Draw mode indicators in the status line
    self.draw_mode_indicator(stdout)?;

    // Show position info
    self.draw_position_info(stdout)?;

    // Show progress indicator if enabled
    if self.show_progress {
      self.draw_progress_indicator(stdout)?;
    }

    Ok(())
  }

  // Draw mode indicator in the status line
  fn draw_mode_indicator(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
    match self.editor_state.mode {
      EditorMode::Command => {
        execute!(stdout, MoveTo(0, (self.height - 1) as u16))?;
        print!(":{}", self.editor_state.command_buffer);
      }
      EditorMode::Search => {
        execute!(stdout, MoveTo(0, (self.height - 1) as u16))?;
        print!("/{}", self.editor_state.command_buffer);
      }
      EditorMode::ReverseSearch => {
        execute!(stdout, MoveTo(0, (self.height - 1) as u16))?;
        print!("?{}", self.editor_state.command_buffer);
      }
      EditorMode::VisualChar => {
        execute!(stdout, MoveTo(0, (self.height - 1) as u16))?;
        print!("-- VISUAL --");
      }
      EditorMode::VisualLine => {
        execute!(stdout, MoveTo(0, (self.height - 1) as u16))?;
        print!("-- VISUAL LINE --");
      }
      _ => {}
    }
    Ok(())
  }

  // Draw position information in the status line
  fn draw_position_info(&self, stdout: &mut io::Stdout) -> io::Result<()> {
    let current_line = self.offset + self.cursor_y;
    let position_info = format!(
      "{}:{} ({}/{})",
      current_line + 1,
      self.cursor_x + 1,
      current_line + 1,
      self.total_lines
    );

    let x = self.width as u16 - position_info.len() as u16 - 1;
    let y = self.height as u16 - 1;
    execute!(stdout, MoveTo(x, y))?;
    print!("{}", position_info);

    Ok(())
  }

  // Draw progress indicator in the status line area
  fn draw_progress_indicator(&self, stdout: &mut io::Stdout) -> io::Result<()> {
    let progress =
      (self.offset as f64 / self.total_lines as f64 * 100.0).round();
    let message = format!("{}%", progress);
    let x = self.width as u16 - message.len() as u16 - 2;
    let y = self.height as u16 - 2;
    execute!(stdout, MoveTo(x, y))?;
    print!("{}", message);

    Ok(())
  }
}
