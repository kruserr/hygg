use crossterm::{
  execute,
  style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
  terminal::{Clear, ClearType},
};
use std::io::{self, Result as IoResult, Write};

use super::core::{Editor, ViewMode};

impl Editor {
  // Draw split view with two panes
  pub(super) fn draw_split_view(
    &self,
    stdout: &mut io::Stdout,
    term_width: u16,
    center_offset_string: &str,
  ) -> IoResult<()> {
    self.debug_log("=== draw_split_view ===");
    self.debug_log(&format!(
      "Active pane: {}, Active buffer: {}",
      self.active_pane, self.active_buffer
    ));

    // Calculate pane heights
    let terminal_height = self.height.saturating_sub(1); // Subtract status line
    let top_height = (terminal_height as f32 * self.split_ratio) as usize;
    let bottom_height =
      terminal_height.saturating_sub(top_height).saturating_sub(1); // -1 for separator

    self.debug_log(&format!(
      "Terminal height: {terminal_height}, Top pane: {top_height}, Bottom pane: {bottom_height}"
    ));

    // Determine buffer indices based on tutorial mode
    let (top_buffer_idx, bottom_buffer_idx) = if self.tutorial_active && self.buffers.len() > 2 {
      // In tutorial mode: show tutorial (1) in top, command (2) in bottom
      self.debug_log("  Tutorial mode split: tutorial in top, command in bottom");
      (1, 2)
    } else {
      // Normal mode: show main (0) in top, command (1) in bottom
      self.debug_log("  Normal mode split: main in top, command in bottom");
      (0, 1)
    };

    // Draw top pane
    self.draw_pane(
      stdout,
      top_buffer_idx, // buffer index
      0, // start row
      top_height,
      term_width,
      center_offset_string,
      self.active_pane == 0,
    )?;

    // Draw separator
    execute!(
      stdout,
      crossterm::cursor::MoveTo(0, top_height as u16),
      Clear(ClearType::CurrentLine),
      SetForegroundColor(Color::DarkGrey)
    )?;
    write!(stdout, "{}", "─".repeat(term_width as usize))?;
    execute!(stdout, ResetColor)?;

    // Draw bottom pane
    self.draw_pane(
      stdout,
      bottom_buffer_idx, // buffer index
      top_height + 1, // start row (after separator)
      bottom_height,
      term_width,
      center_offset_string,
      self.active_pane == 1,
    )?;

    Ok(())
  }

  // Draw a single pane
  #[allow(clippy::too_many_arguments)]
  fn draw_pane(
    &self,
    stdout: &mut io::Stdout,
    buffer_idx: usize,
    start_row: usize,
    height: usize,
    term_width: u16,
    center_offset_string: &str,
    is_active: bool,
  ) -> IoResult<()> {
    self.debug_log(&format!(
      "Drawing pane - buffer: {buffer_idx}, start_row: {start_row}, height: {height}, active: {is_active}"
    ));

    if let Some(buffer) = self.buffers.get(buffer_idx) {
      let offset = buffer.offset;
      let cursor_y = buffer.cursor_y;
      self.debug_log(&format!(
        "  Buffer {buffer_idx}: offset={offset}, cursor_y={cursor_y}, lines={}",
        buffer.lines.len()
      ));

      for i in 0..height {
        let display_row = start_row + i;
        execute!(stdout, crossterm::cursor::MoveTo(0, display_row as u16))?;

        let line_idx = offset + i;
        if line_idx < buffer.lines.len() {
          let line = buffer.lines[line_idx].clone();

          // Highlight current line if this is the active pane
          let is_current_line = is_active && i == cursor_y;
          if self.show_highlighter && is_current_line {
            execute!(
              stdout,
              SetBackgroundColor(Color::Rgb { r: 40, g: 40, b: 40 })
            )?;
            write!(stdout, "{}", " ".repeat(term_width as usize))?;
            execute!(stdout, crossterm::cursor::MoveTo(0, display_row as u16))?;
          }

          // Render the line content
          self.render_pane_line(
            stdout,
            &line,
            buffer_idx,
            i,
            center_offset_string,
            is_current_line,
          )?;

          // Clear to end of line to prevent bleeding
          execute!(stdout, Clear(ClearType::UntilNewLine))?;

          // Reset highlighting
          if is_current_line {
            execute!(stdout, ResetColor)?;
          }
        } else {
          // Empty line
          execute!(stdout, Clear(ClearType::CurrentLine))?;
        }
      }
    } else {
      self.debug_log(&format!("Warning: Buffer {buffer_idx} not found"));
      // Clear the entire pane if buffer not found
      for i in 0..height {
        let display_row = start_row + i;
        execute!(stdout, crossterm::cursor::MoveTo(0, display_row as u16))?;
        execute!(stdout, Clear(ClearType::CurrentLine))?;
      }
    }

    Ok(())
  }

  // Render a single line in a pane
  fn render_pane_line(
    &self,
    stdout: &mut io::Stdout,
    line: &str,
    buffer_idx: usize,
    _display_row: usize,
    center_offset_string: &str,
    is_current_line: bool,
  ) -> IoResult<()> {
    // Apply centering if needed
    if let Some(_buffer) = self.buffers.get(buffer_idx) {
      // Always apply centering offset for consistency with main display
      let line_to_render = format!("{center_offset_string}{line}");

      // Check for search matches
      if !self.editor_state.search_query.is_empty() && !is_current_line {
        if line.contains(&self.editor_state.search_query) {
          self.render_line_with_search_highlight(
            stdout,
            &line_to_render,
            &self.editor_state.search_query,
          )?;
        } else {
          write!(stdout, "{line_to_render}")?;
        }
      } else {
        write!(stdout, "{line_to_render}")?;
      }
    }

    Ok(())
  }

  // Render line with search term highlighted
  fn render_line_with_search_highlight(
    &self,
    stdout: &mut io::Stdout,
    line: &str,
    search_term: &str,
  ) -> IoResult<()> {
    let mut last_end = 0;
    for (start, part) in line.match_indices(search_term) {
      // Write text before match
      write!(stdout, "{}", &line[last_end..start])?;
      // Write match with highlight
      execute!(
        stdout,
        SetBackgroundColor(Color::Yellow),
        SetForegroundColor(Color::Black)
      )?;
      write!(stdout, "{part}")?;
      execute!(stdout, ResetColor)?;
      last_end = start + part.len();
    }
    // Write remaining text
    write!(stdout, "{}", &line[last_end..])?;
    Ok(())
  }
}