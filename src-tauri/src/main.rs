// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Error;
use std::io::Read;
use uuid::Uuid;

mod cli;
mod file;
mod hermenia;
mod jules;

use cli::run as run_cli;
use hermenia::Machine;
use jules::{download_model, invoke_llama_cli, model_exists};

use std::collections::HashMap;
use std::fs::{self, create_dir_all, File};
use std::io::prelude::*;
use std::io::Result;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tauri::Manager;

use file::{read_file, write_file};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        run_cli(args).await;
    } else {
        app_lib::run();
    }
}
