// Editor module - main exports

mod commands;
mod core;
mod cursor;
mod display;
mod event_handler;
mod highlighting;
mod navigation;
mod selection;
mod status_line;
mod tutorial_display;
mod utils;
mod yank;

// Re-export main structures and functions
pub use commands::handle_command;
pub use core::{Editor, EditorMode, EditorState};
