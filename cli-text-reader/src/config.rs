use dirs::config_dir;
use std::fs;
use std::path::PathBuf;

#[derive(Default)]
pub struct AppConfig {
  pub enable_tutorial: Option<bool>,
  pub enable_line_highlighter: Option<bool>,
}

fn get_config_env_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
  let mut config_path =
    config_dir().ok_or("Unable to find config directory")?;
  config_path.push("hygg");
  std::fs::create_dir_all(&config_path)?;
  config_path.push(".env");
  Ok(config_path)
}

fn ensure_config_file() -> Result<(), Box<dyn std::error::Error>> {
  let config_path = get_config_env_path()?;
  if !config_path.exists() {
    fs::write(
      config_path,
      "ENABLE_TUTORIAL=true\nENABLE_LINE_HIGHLIGHTER=true\n",
    )?;
  }
  Ok(())
}

pub fn load_config() -> AppConfig {
  let mut config = AppConfig::default();

  if let Ok(config_path) = get_config_env_path() {
    if ensure_config_file().is_ok() {
      dotenvy::from_path(config_path).ok();
      if let Ok(val) = std::env::var("ENABLE_TUTORIAL") {
        config.enable_tutorial = Some(val.to_lowercase() == "true");
      }
      if let Ok(val) = std::env::var("ENABLE_LINE_HIGHLIGHTER") {
        config.enable_line_highlighter = Some(val.to_lowercase() == "true");
      }
    }
  }

  config
}
