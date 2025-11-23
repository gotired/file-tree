mod repository;
mod services;
mod types;

use repository::Handle;

use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    // Check for CLI arguments
    let args: Vec<String> = env::args().collect();
    let is_tui_mode = args.contains(&"--tui".to_string());

    if is_tui_mode {
        Handle::tui()
    } else {
        Handle::cli()
    }
}
