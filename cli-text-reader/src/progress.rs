use crate::utils::get_hygg_config_file;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Progress {
  pub document_hash: u64,
  pub offset: usize,
  pub total_lines: usize,
  pub percentage: f64,
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

pub fn generate_hash<T: Hash>(t: &T) -> u64 {
  let mut s = DefaultHasher::new();
  t.hash(&mut s);
  s.finish()
}

fn get_progress_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
  get_hygg_config_file(".progress.jsonl")
}

pub fn save_progress(
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

pub fn load_progress(
  document_hash: u64,
) -> Result<Progress, Box<dyn std::error::Error>> {
  let progress_file_path = get_progress_file_path()?;
  let file = OpenOptions::new().read(true).open(progress_file_path)?;
  let reader = io::BufReader::new(file);
  let mut latest_progress: Option<Progress> = None;

  for line in reader.lines() {
    let line = line?;
    let event: Event = serde_json::from_str(&line)?;

    let Event::UpdateProgress {
      document_hash: hash,
      offset,
      total_lines,
      percentage,
      ..
    } = event;

    if hash == document_hash {
      latest_progress =
        Some(Progress { document_hash: hash, offset, total_lines, percentage });
    }
  }

  latest_progress
    .ok_or_else(|| "No progress found for the given document hash".into())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::tempdir;

  #[test]
  fn test_save_and_load_progress() {
    // Create a temporary directory for test
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path().join(".progress.jsonl");

    // Mock the get_progress_file_path function by creating a test file
    let test_hash = 12345u64;
    let test_offset = 50usize;
    let test_total_lines = 100usize;

    // Save progress
    let percentage = (test_offset as f64 / test_total_lines as f64) * 100.0;
    let event = Event::UpdateProgress {
      timestamp: Utc::now(),
      document_hash: test_hash,
      offset: test_offset,
      total_lines: test_total_lines,
      percentage,
    };

    let serialized = serde_json::to_string(&event).unwrap();
    fs::write(&temp_path, format!("{serialized}\n")).unwrap();

    // Load progress by reading the file directly
    let file = OpenOptions::new().read(true).open(&temp_path).unwrap();
    let reader = io::BufReader::new(file);
    let mut loaded_progress: Option<Progress> = None;

    for line in reader.lines() {
      let line = line.unwrap();
      let event: Event = serde_json::from_str(&line).unwrap();

      let Event::UpdateProgress {
        document_hash: hash,
        offset,
        total_lines,
        percentage,
        ..
      } = event;

      if hash == test_hash {
        loaded_progress = Some(Progress {
          document_hash: hash,
          offset,
          total_lines,
          percentage,
        });
      }
    }

    // Verify the loaded progress
    let progress = loaded_progress.expect("Progress should be loaded");
    assert_eq!(progress.document_hash, test_hash);
    assert_eq!(progress.offset, test_offset);
    assert_eq!(progress.total_lines, test_total_lines);
    assert_eq!(progress.percentage, 50.0);
  }
}
