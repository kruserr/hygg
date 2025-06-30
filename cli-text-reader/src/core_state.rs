use arboard::Clipboard;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use crossterm::event::KeyEvent;

use crate::highlights::HighlightData;
use crate::demo_script::DemoScript;
use crate::interactive_tutorial_buffer::TutorialSuccessCondition;
use super::core_types::{BufferState, EditorState, ViewMode};

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
  pub progress_display_until: Option<Instant>,
  pub show_progress: bool,
  pub cursor_x: usize,              
  pub cursor_y: usize,              
  pub clipboard: Option<Clipboard>, 
  pub buffers: Vec<BufferState>,    
  pub active_buffer: usize,         
  pub view_mode: ViewMode,          
  pub show_cursor: bool,            
  pub last_find_char: Option<char>, 
  pub last_find_forward: bool,      
  pub last_find_till: bool,         
  pub marks: HashMap<char, (usize, usize)>, 
  pub previous_position: Option<(usize, usize)>, 
  pub number_prefix: String, 
  pub highlights: HighlightData, 
  // Split view management
  pub active_pane: usize, // 0 = top pane, 1 = bottom pane
  pub split_ratio: f32,   // Percentage for top pane (0.0-1.0)
  pub tmux_prefix_active: bool, // Tracks if Ctrl+B was pressed
  // Rendering optimization
  pub needs_redraw: bool,
  pub last_offset: usize,
  pub force_clear: bool,
  pub cursor_moved: bool,
  // Tutorial state
  pub tutorial_step: usize,
  pub tutorial_active: bool,
  pub tutorial_demo_mode: bool,
  pub tutorial_start_time: Option<Instant>,
  // Demo script execution
  pub demo_script: Option<DemoScript>,
  pub demo_action_index: usize,
  pub demo_last_action_time: Option<Instant>,
  pub demo_hint_text: Option<String>,
  pub demo_hint_until: Option<Instant>,
  pub demo_typing_char_index: usize,
  pub demo_pending_keys: Vec<KeyEvent>,
  pub tutorial_practice_start: usize,
  pub tutorial_practice_lines: usize,
  pub current_tutorial_condition: Option<TutorialSuccessCondition>,
  pub tutorial_highlight_created: bool,
  pub tutorial_yank_performed: bool,
  pub tutorial_paste_performed: bool,
  pub tutorial_search_navigated: bool,
  pub tutorial_bookmark_jumped: bool,
  pub tutorial_forward_search_used: bool,
  pub tutorial_backward_search_used: bool,
  pub last_executed_command: Option<String>,
  pub tutorial_step_completed: bool,
  // Key event debouncing for remote desktop/VM issues
  pub last_key_event: Option<(KeyEvent, Instant)>,
  pub key_debounce_duration: Duration,
}