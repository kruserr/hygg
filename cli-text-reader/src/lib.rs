mod bookmarks;
mod config;
mod core_state;
mod core_types;
mod debug;
mod demo_script;
mod demo_tutorial_test;
mod editor;
mod help;
mod highlights;
mod highlights_core;
mod highlights_persistence;
mod interactive_tutorial;
mod interactive_tutorial_utils;
mod interactive_tutorial_steps;
mod interactive_tutorial_buffer;
mod interactive_tutorial_tests;
mod progress;
mod tutorial;
mod utils;

use editor::Editor;

pub fn run_cli_text_reader(
  lines: Vec<String>,
  col: usize,
) -> Result<(), Box<dyn std::error::Error>> {
  run_cli_text_reader_with_demo(lines, col, false)
}


pub fn run_cli_text_reader_with_demo(
  lines: Vec<String>,
  col: usize,
  demo_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
  run_cli_text_reader_with_content(lines, col, None, demo_mode)
}

pub fn run_cli_text_reader_with_content(
  lines: Vec<String>,
  col: usize,
  raw_content: Option<String>,
  demo_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
  // Initialize debug logging
  debug::init_debug_logging()?;
  debug::debug_log("main", "Starting cli-text-reader");
  debug::debug_log_state("main", "lines_count", &lines.len().to_string());
  debug::debug_log_state("main", "col", &col.to_string());
  debug::debug_log_state("main", "demo_mode", &demo_mode.to_string());
  if raw_content.is_some() {
    debug::debug_log("main", "Raw content provided for consistent hashing");
  }

  let mut editor = if let Some(content) = raw_content {
    Editor::new_with_content(lines, col, content)
  } else {
    Editor::new(lines, col)
  };
  editor.tutorial_demo_mode = demo_mode;
  let result = editor.run();

  debug::debug_log("main", "Editor run completed");
  debug::flush_debug_log();
  result
}
