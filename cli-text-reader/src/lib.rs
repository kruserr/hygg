use chrono::{DateTime, Utc};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use crossterm::{
  cursor::{Hide, MoveTo, Show},
  event::{self, Event as CEvent, KeyCode, KeyEvent},
  execute,
  terminal::{self, Clear, ClearType},
};

#[allow(dead_code)]
fn handle_command(command: String) {
  if command == "q" {
    std::process::exit(0);
  }
  if command == ":q" {
    std::process::exit(0);
  }
}

use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
struct Progress {
  document_hash: u64,
  offset: usize,
  total_lines: usize,
  percentage: f64,
}

#[derive(Serialize, Deserialize)]
enum Event {
  UpdateProgress {
    timestamp: DateTime<Utc>,
    document_hash: u64,
    offset: usize,
    total_lines: usize,
    percentage: f64,
  },
}

fn generate_hash<T: Hash>(t: &T) -> u64 {
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

fn get_progress_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
  let mut config_path =
    config_dir().ok_or("Unable to find config directory")?;
  config_path.push("hygg");
  std::fs::create_dir_all(&config_path)?;
  config_path.push(".progress.jsonl");
  Ok(config_path)
}

fn save_progress(
  document_hash: u64,
  offset: usize,
  total_lines: usize,
) -> Result<(), Box<dyn std::error::Error>> {
  let percentage = (offset as f64 / total_lines as f64) * 100.0;
  let event = Event::UpdateProgress {
    timestamp: Utc::now(),
    document_hash,
    offset,
    total_lines,
    percentage,
  };
  let serialized = serde_json::to_string(&event)?;
  let progress_file_path = get_progress_file_path()?;
  let mut file =
    OpenOptions::new().create(true).append(true).open(progress_file_path)?;
  file.write_all(serialized.as_bytes())?;
  file.write_all(b"\n")?;
  Ok(())
}

fn load_progress(
  document_hash: u64,
) -> Result<Progress, Box<dyn std::error::Error>> {
  let progress_file_path = get_progress_file_path()?;
  let file = OpenOptions::new().read(true).open(progress_file_path)?;
  let reader = io::BufReader::new(file);
  let mut latest_progress: Option<Progress> = None;

  for line in reader.lines() {
    let line = line?;
    let event: Event = serde_json::from_str(&line)?;
    #[allow(irrefutable_let_patterns)]
    if let Event::UpdateProgress {
      document_hash: hash,
      offset,
      total_lines,
      percentage,
      ..
    } = event
    {
      if hash == document_hash {
        latest_progress = Some(Progress {
          document_hash: hash,
          offset,
          total_lines,
          percentage,
        });
      }
    }
  }

  latest_progress
    .ok_or_else(|| "No progress found for the given document hash".into())
}

pub fn run_cli_text_reader(
  lines: Vec<String>,
  col: usize,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut stdout = io::stdout();

  let document_hash = generate_hash(&lines);
  let total_lines = lines.len();
  let mut offset = match load_progress(document_hash) {
    Ok(progress) => {
      (progress.percentage / 100.0 * total_lines as f64).round() as usize
    }
    Err(_) => 0,
  };

  let mut width = terminal::size()?.0 as usize;
  let mut height = terminal::size()?.1 as usize;

  if std::io::stdout().is_terminal() {
    execute!(stdout, terminal::EnterAlternateScreen, Hide)?;
    terminal::enable_raw_mode()?;
  }

  loop {
    if std::io::stdout().is_terminal() {
      execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    }

    let show_line_number = false;
    let center = true;

    let center_offset = if width > col { (width / 2) - col / 2 } else { 0 };

    let center_offset_string =
      if center { " ".repeat(center_offset) } else { "".to_string() };

    for (i, line_orig) in lines.iter().skip(offset).take(height).enumerate() {
      let mut line = line_orig.clone();

      execute!(stdout, MoveTo(0, i as u16))?;

      if show_line_number {
        line = format!("{i} {line}");
      }

      println!("{center_offset_string}{line}");
    }

    stdout.flush()?;

    if std::io::stdout().is_terminal() {
      match event::read()? {
        CEvent::Key(key_event) => match key_event.code {
          KeyCode::Char('j') | KeyCode::Down => {
            if offset + height < total_lines {
              offset += 1;
            }
          }
          KeyCode::Char('k') | KeyCode::Up => {
            if offset > 0 {
              offset -= 1;
            }
          }
          KeyCode::PageDown => {
            if offset + height < total_lines {
              offset += height - 3;
            }
          }
          KeyCode::PageUp => {
            if offset as i32 - height as i32 > 0 {
              offset -= height - 3;
            } else {
              offset = 0;
            }
          }
          KeyCode::Char('q') => break,
          _ => {}
        },
        CEvent::Resize(_, _) => {
          width = terminal::size()?.0 as usize;
          height = terminal::size()?.1 as usize;
        }
        _ => {}
      }
    } else {
      break;
    }

    save_progress(document_hash, offset, total_lines)?;
  }

  if std::io::stdout().is_terminal() {
    execute!(stdout, Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
  }
  Ok(())
}
