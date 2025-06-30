use crate::utils::{
  ensure_config_file_with_defaults, get_hygg_config_file, parse_bool_env_var,
};
use std::fs;
use std::path::PathBuf;

#[derive(Default)]
pub struct AppConfig {
  pub enable_tutorial: Option<bool>,
  pub enable_line_highlighter: Option<bool>,
  pub show_cursor: Option<bool>,
  pub show_progress: Option<bool>,
  pub tutorial_shown: Option<bool>,
}

fn get_config_env_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
  get_hygg_config_file(".env")
}

fn ensure_config_file() -> Result<(), Box<dyn std::error::Error>> {
  let config_path = get_config_env_path()?;
  ensure_config_file_with_defaults(
    &config_path,
    "ENABLE_TUTORIAL=true\nENABLE_LINE_HIGHLIGHTER=true\nSHOW_CURSOR=true\nSHOW_PROGRESS=false\nTUTORIAL_SHOWN=false\n",
  )
}

pub fn load_config() -> AppConfig {
  let mut config = AppConfig::default();

  if let Ok(config_path) = get_config_env_path() {
    if ensure_config_file().is_ok() {
      dotenvy::from_path(config_path).ok();
      config.enable_tutorial = parse_bool_env_var("ENABLE_TUTORIAL");
      config.enable_line_highlighter =
        parse_bool_env_var("ENABLE_LINE_HIGHLIGHTER");
      config.show_cursor = parse_bool_env_var("SHOW_CURSOR");
      config.show_progress = parse_bool_env_var("SHOW_PROGRESS");
      config.tutorial_shown = parse_bool_env_var("TUTORIAL_SHOWN");
    }
  }

  config
}

pub fn save_config(
  config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
  let config_path = get_config_env_path()?;
  
  let existing_config = load_config();

  let enable_tutorial = config.enable_tutorial.or(existing_config.enable_tutorial).unwrap_or(true);
  let enable_line_highlighter = config.enable_line_highlighter.or(existing_config.enable_line_highlighter).unwrap_or(true);
  let show_cursor = config.show_cursor.or(existing_config.show_cursor).unwrap_or(true);
  let show_progress = config.show_progress.or(existing_config.show_progress).unwrap_or(false);
  let tutorial_shown = config.tutorial_shown.or(existing_config.tutorial_shown).unwrap_or(false);

  let content = format!(
    "ENABLE_TUTORIAL={enable_tutorial}\nENABLE_LINE_HIGHLIGHTER={enable_line_highlighter}\nSHOW_CURSOR={show_cursor}\nSHOW_PROGRESS={show_progress}\nTUTORIAL_SHOWN={tutorial_shown}\n"
  );

  fs::write(config_path, content)?;
  Ok(())
}
