use super::core::Editor;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Attribute, Color, SetAttribute, SetBackgroundColor, SetForegroundColor, ResetColor, Print},
};
use std::io::Write;

impl Editor {
    // Render demo hint if active - minimalistic box with text
    pub fn render_demo_hint(&self, stdout: &mut std::io::Stdout, width: usize, height: usize) -> std::io::Result<()> {
        // Only render if we have hint text
        let hint_text = match &self.demo_hint_text {
            Some(text) if !text.is_empty() => text,
            _ => return Ok(()),
        };
        
        // Split hint text into lines
        let hint_lines: Vec<&str> = hint_text.split('\n').collect();
        let max_line_len = hint_lines.iter().map(|s| s.chars().count()).max().unwrap_or(0);
        
        // Box dimensions with padding
        let padding = 2;
        let box_width = max_line_len + (padding * 2) + 2; // +2 for borders
        let box_height = hint_lines.len() + 2; // +2 for top/bottom borders
        
        // Calculate position - centered horizontally, near bottom
        let box_x = (width.saturating_sub(box_width)) / 2;
        
        // Position the box near the bottom but ensure it's visible
        // Use a large fixed offset that should work for most terminals
        let bottom_offset = 8; // 8 lines from bottom
        let box_y = if height > 20 {
            // For normal terminals, position from bottom
            height.saturating_sub(bottom_offset + box_height)
        } else {
            // For very small terminals, center it
            (height.saturating_sub(box_height)) / 2
        };
        
        // Dark background color for the box
        let bg_color = Color::Rgb { r: 20, g: 20, b: 20 };
        let border_color = Color::Rgb { r: 255, g: 191, b: 0 }; // Amber
        let text_color = Color::Rgb { r: 255, g: 191, b: 0 }; // Amber
        
        // Draw top border
        execute!(
            stdout,
            MoveTo(box_x as u16, box_y as u16),
            SetBackgroundColor(bg_color),
            SetForegroundColor(border_color),
            Print(format!("╭{}╮", "─".repeat(box_width - 2))),
            ResetColor,
        )?;
        
        // Draw content lines
        for (i, line) in hint_lines.iter().enumerate() {
            let y = box_y + i + 1;
            let line_chars = line.chars().count();
            let left_padding = (max_line_len - line_chars) / 2 + padding;
            let right_padding = box_width - line_chars - left_padding - 2;
            
            execute!(
                stdout,
                MoveTo(box_x as u16, y as u16),
                SetBackgroundColor(bg_color),
                SetForegroundColor(border_color),
                Print("│"),
                SetForegroundColor(text_color),
                Print(format!("{}{}{}", 
                    " ".repeat(left_padding),
                    line,
                    " ".repeat(right_padding)
                )),
                SetForegroundColor(border_color),
                Print("│"),
                ResetColor,
            )?;
        }
        
        // Draw bottom border
        execute!(
            stdout,
            MoveTo(box_x as u16, (box_y + box_height - 1) as u16),
            SetBackgroundColor(bg_color),
            SetForegroundColor(border_color),
            Print(format!("╰{}╯", "─".repeat(box_width - 2))),
            ResetColor,
        )?;
        
        Ok(())
    }
}