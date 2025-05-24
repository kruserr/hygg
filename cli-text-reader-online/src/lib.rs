mod config;
mod editor;
mod progress;
mod server;
mod tutorial;

use chrono::Utc;
use editor::Editor;
use server::{HyggClient, ReadingProgress};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use tokio::fs::read_to_string;
use uuid::Uuid;

/// Upload a local file to the server for syncing
///
/// This allows users to upload their files to the server so they can be accessed
/// from other clients and their reading progress can be tracked online.
///
/// # Arguments
/// * `local_file_path` - Path to the local file to upload
/// * `user_id` - User identifier
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Result indicating success or error
pub async fn upload_file_to_server(
  local_file_path: String,
  user_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
  // Check if file exists
  if !Path::new(&local_file_path).exists() {
    return Err(format!("File not found: {}", local_file_path).into());
  }

  // Read file content
  let content = read_to_string(&local_file_path).await?.replace('\r', ""); // Normalize line endings

  // Extract filename from path
  let file_name = Path::new(&local_file_path)
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| format!("Invalid file path: {}", local_file_path))?;

  // Create client and upload file
  let client = HyggClient::new(user_id.clone());
  client.upload_file(file_name, &content).await?;

  println!("Successfully uploaded {} to the server", file_name);
  Ok(())
}

pub async fn run_cli_text_reader(
  file_path: String,
  user_id: String,
  col: usize,
) -> Result<(), Box<dyn std::error::Error>> {
  let client = HyggClient::new(user_id.clone());

  // Get file content from server
  let content = client.get_file_content(&file_path).await?;
  let lines: Vec<String> = content.lines().map(String::from).collect();

  // Create or get progress with file-based ID
  // Generate consistent UUID based on file path
  let mut hasher = DefaultHasher::new();
  file_path.hash(&mut hasher);
  let hash = hasher.finish();

  // Create a reproducible UUID by using the hash as seed
  // This ensures the same file always gets the same UUID
  let hash_bytes = hash.to_be_bytes();
  let uuid_bytes = [
    hash_bytes[0],
    hash_bytes[1],
    hash_bytes[2],
    hash_bytes[3],
    hash_bytes[4],
    hash_bytes[5],
    hash_bytes[6],
    hash_bytes[7],
    hash_bytes[0],
    hash_bytes[1],
    hash_bytes[2],
    hash_bytes[3],
    hash_bytes[4],
    hash_bytes[5],
    hash_bytes[6],
    hash_bytes[7],
  ];

  let file_uuid = Uuid::from_bytes(uuid_bytes);
  println!("Using file-based UUID: {}", file_uuid);

  let mut progress = ReadingProgress {
    id: file_uuid,
    file_path,
    position: 0,
    user_id: user_id.clone(),
    last_accessed: Utc::now(),
    lock_holder: None,
    lock_expiry: None,
  };

  // Try to acquire lock - if it fails, open in read-only mode
  let read_only_mode = match client.acquire_lock(&progress).await {
    Ok(locked_progress) => {
      progress = locked_progress;
      false // We have the lock, so not read-only
    }
    Err(e) => {
      // Check if it's a lock error (someone else has the lock)
      if e.to_string().contains("locked by") {
        println!(
          "\nOpening in READ-ONLY mode because file is locked by another user."
        );
        println!("Close and reopen to attempt to acquire the lock.\n");

        // Try to get the current progress to display at the right position
        match client.get_progress(&progress.id.to_string()).await {
          Ok(current_progress) => {
            progress = current_progress;
            true // Read-only mode
          }
          Err(_) => {
            // If we can't get progress, still open read-only at position 0
            true // Read-only mode
          }
        }
      } else {
        // For non-lock errors, propagate the error
        return Err(Box::new(std::io::Error::new(
          std::io::ErrorKind::Other,
          e.to_string(),
        )));
      }
    }
  };

  // Create editor with progress tracking
  let mut editor = Editor::new(lines, col);
  editor.set_position(progress.position);

  let result = if read_only_mode {
    editor.set_read_only(true);
    // Run editor without progress updates
    editor.run()
  } else {
    // Run editor with progress tracking
    let client_clone = client.clone();
    let progress_clone = progress.clone();

    editor.run_with_progress(move |pos| {
      let mut progress_update = progress_clone.clone();
      progress_update.position = pos;
      let client = client_clone.clone();
      tokio::spawn(async move {
        if let Err(e) = client.update_progress(&progress_update).await {
          eprintln!("Failed to update progress: {}", e);
        }
      });
    })
  };

  // Release lock if we had one (not read-only mode)
  if !read_only_mode {
    match client.release_lock(&progress).await {
      Ok(_) => println!(
        "\nLock released successfully. Other users can now edit this file."
      ),
      Err(e) => eprintln!("\nFailed to release lock: {}", e),
    }
  }

  result
}
