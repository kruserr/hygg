use cli_justify;
use cli_pdf_to_text;
use cli_text_reader;
use redirect_stderr;

use clap::Parser;
use std::{env, fmt::format};

/// Simplifying the way you read
#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None,
    help_template = concat!(
        "{before-help}{name} {version}\n",
        "{author-with-newline}{about-with-newline}",
        "Repository: ", env!("CARGO_PKG_REPOSITORY"), "\n",
        "License: ", env!("CARGO_PKG_LICENSE"), "\n\n",
        "{usage-heading} {usage}\n\n",
        "{all-args}{after-help}\n"
    )
)]
struct Args {
  /// Input file to process
  file: Option<String>,

  /// Set the column width
  #[arg(short, long, default_value = "110")]
  col: usize,

  /// Use OCR to extract text from scanned PDF documents
  /// Depends on ocrmypdf and tesseract-ocr lang e.g.
  /// sudo apt install ocrmypdf tesseract-ocr-eng
  #[arg(short, long, default_value = "false")]
  ocr: bool,

  /// Use the hygg server upload
  #[arg(short, long)]
  upload: Option<String>,

  /// Use the hygg server list
  #[arg(short, long, default_value = "false")]
  list: bool,

  /// Use the hygg server read
  #[arg(short, long)]
  read: Option<String>,
}

pub fn which(binary: &str) -> Option<std::path::PathBuf> {
  if let Ok(paths) = env::var("PATH") {
    for path in env::split_paths(&paths) {
      let full_path = path.join(binary);
      if full_path.is_file() {
        return Some(full_path);
      }
    }
  }
  return None;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  redirect_stderr::redirect_stderr().expect("Failed to redirect stderr");

  let args = Args::parse();

  // Handle server operations (commented out as in original)
  // if let Some(server_upload) = &args.upload {
  //   // TODO function to upload a local file to the server here
  //   return Ok(());
  // }

  // if args.list {
  //   // TODO function to list all file hashes, progress and original file names here
  //   return Ok(());
  // }

  // if let Some(server_read) = &args.read {
  //   // TODO function to download file with the hash of `server_read`
  //   // and store locally with the hash as name here
  //   return Ok(());
  // }

  // Get the file to process - either from args or from command line
  let file = if let Some(file) = args.file {
    file
  } else {
    // If no file provided via clap, try to get it from command line args (for backward compatibility)
    std::env::args().next_back().unwrap_or_else(|| {
      eprintln!("Error: No input file provided");
      std::process::exit(1);
    })
  };
  let temp_file = format!("{file}-{}", uuid::Uuid::new_v4());

  let content = if (args.ocr && which("ocrmypdf").is_some()) {
    let output = std::process::Command::new("ocrmypdf")
      .arg("--force-ocr")
      .arg(&file)
      .arg(&temp_file)
      .output()
      .map_err(|e| e.to_string())?;

    #[allow(unused_variables)]
    let result = (String::from_utf8_lossy(&output.stdout)
      + String::from_utf8_lossy(&output.stderr))
    .to_string();

    // println!("{result}");

    cli_pdf_to_text::pdf_to_text(&temp_file)?
  } else {
    cli_epub_to_text::epub_to_text(&file)
      .or(cli_pdf_to_text::pdf_to_text(&file))?
  };

  let lines = cli_justify::justify(&content, args.col);

  cli_text_reader::run_cli_text_reader(lines, args.col)?;

  if std::path::Path::new(&temp_file).exists() {
    std::fs::remove_file(&temp_file)?;
  }

  Ok(())
}
