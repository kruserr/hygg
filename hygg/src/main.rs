use cli_justify;
use cli_pdf_to_text;
use cli_text_reader;
use redirect_stderr;

use std::{env, fmt::format};

use getopts;

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

pub fn print_help_menu(args: Vec<String>, opts: getopts::Options) {
  let brief = format!("Usage: {} FILE [options]", args[0]);
  print!("{}", opts.usage(&brief));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  redirect_stderr::redirect_stderr().expect("Failed to redirect stderr");

  let args: Vec<String> = env::args().collect();
  let mut opts = getopts::Options::new();

  opts.optflag("h", "help", "print this help menu");

  opts.optopt("c", "col", "set the column, defaults to 110", "NUMBER");

  opts.optopt(
    "o",
    "ocr",
    "
    use ocr to extract text from scanned pdf documents,
    depends on ocrmypdf and a tesseract-ocr lang e.g.
    sudo apt install ocrmypdf tesseract-ocr-eng
  ",
    "BOOLEAN",
  );

  opts.optopt(
    "u",
    "upload",
    "
    use the hygg server upload,
  ",
    "STRING",
  );

  opts.optopt(
    "l",
    "list",
    "
    use the hygg server list,
  ",
    "BOOLEAN",
  );

  opts.optopt(
    "r",
    "read",
    "
    use the hygg server read,
  ",
    "STRING",
  );

  let matches = opts.parse(&args[1..])?;

  if (matches.opt_present("h") || args.len() < 2) {
    print_help_menu(args, opts);
    return Ok(());
  }

  let col = match matches.opt_str("c") {
    Some(x) => x.parse().unwrap_or(110),
    None => 110,
  };

  let ocr = match matches.opt_str("ocr") {
    Some(x) => x.parse().unwrap_or(false),
    None => false,
  };

  let server_upload = match matches.opt_str("upload") {
    Some(x) => x.parse().unwrap_or("".to_owned()),
    None => "".to_owned(),
  };

  let server_list = match matches.opt_str("list") {
    Some(x) => x.parse().unwrap_or(false),
    None => false,
  };

  let server_read = match matches.opt_str("read") {
    Some(x) => x.parse().unwrap_or("".to_owned()),
    None => "".to_owned(),
  };

  println!("
    server_upload: {server_upload},
    server_list: {server_list},
    server_read: {server_read}
  ");

  if (!server_upload.is_empty()) {
    // TODO function to upload a local file to the server here

    return Ok(());
  }

  if(server_list) {
    // TODO function to list all file hashes, progress and original file names here

    return Ok(());
  }

  if (!server_read.is_empty()) {
    // TODO function to download file with the hash of `server_read`
    // and store locally with the hash as name here

    // let server_file_path = ""; // TODO get server file hash local path here

    // let content = cli_epub_to_text::epub_to_text(&server_file_path)
    //   .or(cli_pdf_to_text::pdf_to_text(&server_file_path))?;
    // let lines = cli_justify::justify(&content, col);
    // cli_text_reader_online::run_cli_text_reader(lines, col)?;

    return Ok(());
  }

  // TODO refactor the above into the assignment of `file` and `temp_file` vars
  // such that ocr also works for files from the server
  let file = std::env::args().last().unwrap();
  let temp_file = format!("{file}-{}", uuid::Uuid::new_v4());

  let content = if (ocr && which("ocrmypdf").is_some()) {
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

  let lines = cli_justify::justify(&content, col);

  cli_text_reader::run_cli_text_reader(lines, col)?;

  if std::path::Path::new(&temp_file).exists() {
    std::fs::remove_file(&temp_file)?;
  }

  Ok(())
}
