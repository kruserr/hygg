use crossterm::{
  cursor::MoveTo,
  execute,
  style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{Result as IoResult, Write};

use super::core::{Editor, EditorMode};

impl Editor {
  // Highlight current line
  pub fn highlight_current_line(
    &self,
    stdout: &mut impl Write,
    line_index: usize,
    term_width: u16,
  ) -> IoResult<bool> {
    if self.show_highlighter && line_index == self.cursor_y {
      // First, draw the background for the entire line
      execute!(
        stdout,
        MoveTo(0, line_index as u16),
        SetBackgroundColor(Color::Rgb { r: 40, g: 40, b: 40 })
      )?;

      // Fill the entire width with background color
      print!("{}", " ".repeat(term_width as usize));

      // Reset cursor position to beginning of line
      execute!(stdout, MoveTo(0, line_index as u16))?;

      return Ok(true);
    } else {
      return Ok(false);
    }
  }

  // Highlight selected text in visual modes
  pub fn highlight_selection(
    &self,
    stdout: &mut impl Write,
    line_index: usize,
    line: &str,
    center_offset_string: &str,
  ) -> IoResult<bool> {
    if (self.editor_state.mode == EditorMode::VisualChar
      || self.editor_state.mode == EditorMode::VisualLine)
      && self.editor_state.selection_start.is_some()
      && self.editor_state.selection_end.is_some()
    {
      let start = self.editor_state.selection_start.unwrap();
      let end = self.editor_state.selection_end.unwrap();
      let current_line_idx = self.offset + line_index;

      // Determine if this line is within the selection range
      let is_in_selection_range =
        if self.editor_state.mode == EditorMode::VisualLine {
          // In line mode, entire lines are selected
          let (min_line, _) = if start.0 <= end.0 { start } else { end };
          let (max_line, _) = if start.0 > end.0 { start } else { end };
          current_line_idx >= min_line && current_line_idx <= max_line
        } else {
          // In char mode, parts of lines are selected
          if start.0 == end.0 {
            // Single line selection
            current_line_idx == start.0
          } else {
            // Multi-line selection
            let (min_line, _) = if start.0 <= end.0 { start } else { end };
            let (max_line, _) = if start.0 > end.0 { start } else { end };
            current_line_idx >= min_line && current_line_idx <= max_line
          }
        };

      if is_in_selection_range {
        // This line is part of the selection, highlight it
        print!("{}", center_offset_string);

        if self.editor_state.mode == EditorMode::VisualLine {
          // In line mode, highlight the entire line
          execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White)
          )?;
          println!("{}", line);
          execute!(stdout, ResetColor)?;
          return Ok(true);
        } else {
          // In char mode, highlight only the selected portion
          let (start_col, end_col) = if start.0 == end.0 {
            // Selection is within the same line
            if start.1 <= end.1 { (start.1, end.1) } else { (end.1, start.1) }
          } else if current_line_idx == start.0 {
            // First line of multi-line selection
            let (min_line, min_col) =
              if start.0 <= end.0 { start } else { end };
            if current_line_idx == min_line {
              (min_col, line.len())
            } else {
              (0, line.len())
            }
          } else if current_line_idx == end.0 {
            // Last line of multi-line selection
            let (max_line, max_col) = if start.0 > end.0 { start } else { end };
            if current_line_idx == max_line {
              (0, max_col)
            } else {
              (0, line.len())
            }
          } else {
            // Middle line of multi-line selection
            (0, line.len())
          };

          // Ensure column indices are valid
          let start_col = start_col.min(line.len());
          let end_col = end_col.min(line.len());

          // Print parts of the line with appropriate highlighting
          print!("{}", &line[..start_col]);
          execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White)
          )?;
          print!("{}", &line[start_col..end_col]);
          execute!(stdout, ResetColor)?;
          println!("{}", &line[end_col..]);

          return Ok(true);
        }
      }
    }

    Ok(false)
  }

  // Highlight search matches
  pub fn highlight_search_match(
    &self,
    stdout: &mut impl Write,
    line_index: usize,
    line: &str,
    center_offset_string: &str,
  ) -> IoResult<bool> {
    if let Some((line_idx, start, end)) = self.editor_state.current_match {
      if line_idx == self.offset + line_index {
        print!("{}", center_offset_string);
        print!("{}", &line[..start]);
        execute!(
          stdout,
          SetBackgroundColor(Color::Yellow),
          SetForegroundColor(Color::Black)
        )?;
        print!("{}", &line[start..end]);
        execute!(stdout, ResetColor)?;
        println!("{}", &line[end..]);
        return Ok(true);
      }
    }

    Ok(false)
  }
}
