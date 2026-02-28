// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod file;
mod hermenia;
mod jules;

use cli::run as run_cli;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        run_cli(args).await;
    } else {
        app_lib::run();
    }
}
