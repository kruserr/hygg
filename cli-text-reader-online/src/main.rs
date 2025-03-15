use cli_text_reader_online;
use uuid::Uuid;

use std::env;
use getopts;

pub fn print_help_menu(args: Vec<String>, opts: getopts::Options) {
    let brief = format!("Usage: {} FILE [options]", args[0]);
    print!("{}", opts.usage(&brief));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optopt("c", "col", "set the column, defaults to 110", "NUMBER");
    opts.optopt("u", "user", "set the user ID", "USER_ID");
    opts.optflag("h", "help", "print this help menu");

    let matches = opts.parse(&args[1..])?;

    if matches.opt_present("h") || matches.free.is_empty() {
        print_help_menu(args, opts);
        return Ok(());
    }

    let col: usize = match matches.opt_str("c") {
        Some(x) => x.parse().unwrap_or(110),
        None => 110,
    };

    let user_id = matches.opt_str("u")
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let file_path = matches.free[0].clone();

    cli_text_reader_online::run_cli_text_reader(
        file_path,
        user_id,
        col,
    ).await?;

    Ok(())
}
