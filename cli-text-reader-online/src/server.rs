use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

const SERVER_URL: &str = "http://localhost:3001";

#[derive(Debug, Error)]
pub enum ClientError {
  #[error("HTTP error: {0}")]
  Http(#[from] reqwest::Error),
  #[error("Lock error: {0}")]
  Lock(String),
  #[error("Upload error: {0}")]
  Upload(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadingProgress {
  pub id: Uuid,
  pub file_path: String,
  pub position: usize,
  pub user_id: String,
  pub last_accessed: DateTime<Utc>,
  pub lock_holder: Option<String>,
  pub lock_expiry: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct HyggClient {
  client: reqwest::Client,
  user_id: String,
}

impl HyggClient {
  pub fn new(user_id: String) -> Self {
    Self { client: reqwest::Client::new(), user_id }
  }

  pub async fn get_file_content(&self, file_path: &str) -> Result<String> {
    // Extract just the filename for server request
    let file_name = std::path::Path::new(file_path)
      .file_name()
      .and_then(|name| name.to_str())
      .unwrap_or("sample.txt");

    let url = format!("{}/file/{}", SERVER_URL, file_name);
    println!("Requesting file from: {}", url);
    let content =
      self.client.get(&url).send().await?.error_for_status()?.text().await?;
    Ok(content)
  }

  pub async fn acquire_lock(
    &self,
    progress: &ReadingProgress,
  ) -> Result<ReadingProgress> {
    let url = format!("{}/progress/lock", SERVER_URL);
    let response = self.client.post(&url).json(progress).send().await?;

    if !response.status().is_success() {
      let error = response.text().await?;
      return Err(ClientError::Lock(error).into());
    }

    let progress = response.json().await?;
    Ok(progress)
  }

  pub async fn update_progress(
    &self,
    progress: &ReadingProgress,
  ) -> Result<ReadingProgress> {
    let url = format!("{}/progress/update", SERVER_URL);
    let response = self.client.post(&url).json(progress).send().await?;

    if !response.status().is_success() {
      let error = response.text().await?;
      return Err(ClientError::Lock(error).into());
    }

    let progress = response.json().await?;
    Ok(progress)
  }

  pub async fn get_progress(
    &self,
    progress_id: &str,
  ) -> Result<ReadingProgress> {
    let url = format!("{}/progress/{}", SERVER_URL, progress_id);
    let response = self.client.get(&url).send().await?;

    if !response.status().is_success() {
      let error = response.text().await?;
      return Err(
        ClientError::Lock(format!("Failed to get progress: {}", error)).into(),
      );
    }

    let progress = response.json().await?;
    Ok(progress)
  }

  pub async fn release_lock(
    &self,
    progress: &ReadingProgress,
  ) -> Result<ReadingProgress> {
    let url = format!("{}/progress/release", SERVER_URL);
    let response = self.client.post(&url).json(progress).send().await?;

    if !response.status().is_success() {
      let error = response.text().await?;
      return Err(
        ClientError::Lock(format!("Failed to release lock: {}", error)).into(),
      );
    }

    let progress = response.json().await?;
    Ok(progress)
  }

  pub async fn upload_file(
    &self,
    file_path: &str,
    content: &str,
  ) -> Result<()> {
    let url = format!("{}/file/upload", SERVER_URL);

    #[derive(Serialize)]
    struct FileUpload {
      file_path: String,
      content: String,
    }

    let upload = FileUpload {
      file_path: file_path.to_string(),
      content: content.to_string(),
    };

    let response = self.client.post(&url).json(&upload).send().await?;

    if !response.status().is_success() {
      let error = response.text().await?;
      return Err(
        ClientError::Upload(format!("Failed to upload file: {}", error)).into(),
      );
    }

    Ok(())
  }
}
