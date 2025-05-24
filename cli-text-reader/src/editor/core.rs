use arboard::Clipboard;
use crossterm::terminal;

use crate::progress::generate_hash;

// Editor modes
#[derive(PartialEq)]
pub enum EditorMode {
  Normal,
  Command,
  Search,
  ReverseSearch,
  VisualChar,
  VisualLine,
}

// Editor state - tracks current mode, command buffer, search state, and selection
pub struct EditorState {
  pub mode: EditorMode,
  pub command_buffer: String,
  pub search_query: String,
  pub search_direction: bool, // true for forward, false for backward
  #[allow(dead_code)]
  pub last_search_index: Option<usize>,
  pub current_match: Option<(usize, usize, usize)>, // (line_index, start, end)
  pub selection_start: Option<(usize, usize)>, // (line_index, column_index)
  pub selection_end: Option<(usize, usize)>,   // (line_index, column_index)
  pub yank_buffer: String,                     // Buffer for yanked text
  pub operator_pending: Option<char>,          // For vim operations like 'y'
}

impl EditorState {
  pub fn new() -> Self {
    Self {
      mode: EditorMode::Normal,
      command_buffer: String::new(),
      search_query: String::new(),
      search_direction: true,
      last_search_index: None,
      current_match: None,
      selection_start: None,
      selection_end: None,
      yank_buffer: String::new(),
      operator_pending: None,
    }
  }
}

// Main editor struct that holds document and UI state
pub struct Editor {
  pub lines: Vec<String>,
  pub col: usize,
  pub offset: usize,
  pub width: usize,
  pub height: usize,
  pub show_highlighter: bool,
  pub editor_state: EditorState,
  pub document_hash: u64,
  pub total_lines: usize,
  #[allow(dead_code)]
  pub progress_display_until: Option<std::time::Instant>,
  pub show_progress: bool,
  pub cursor_x: usize,              // Cursor column position
  pub cursor_y: usize,              // Cursor line position relative to offset
  pub clipboard: Option<Clipboard>, // System clipboard
}

impl Editor {
  pub fn new(lines: Vec<String>, col: usize) -> Self {
    let document_hash = generate_hash(&lines);
    let total_lines = lines.len();
    let (width, height) = terminal::size()
      .map(|(w, h)| (w as usize, h as usize))
      .unwrap_or((80, 24));

    // Initialize clipboard - may fail on headless systems
    let clipboard = Clipboard::new().ok();

    Self {
      lines,
      col,
      offset: 0,
      width,
      height,
      show_highlighter: true,
      editor_state: EditorState::new(),
      document_hash,
      total_lines,
      progress_display_until: None,
      show_progress: false,
      cursor_x: 0,
      cursor_y: height / 2,
      clipboard,
    }
  }

  // Get the actual cursor position in the document (line_index, column)
  pub fn get_cursor_position(&self) -> (usize, usize) {
    // Calculate the correct line index based on the cursor's position
    // This ensures we get the line currently being displayed under the cursor
    let line_idx = self.offset + self.cursor_y;

    // Make sure we don't exceed the document boundaries
    let line_idx = line_idx.min(self.lines.len().saturating_sub(1));

    (line_idx, self.cursor_x)
  }
}
