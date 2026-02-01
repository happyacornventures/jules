use std::collections::HashMap;
use std::fs::{self, create_dir_all, File};
use std::io::prelude::*;
use std::io::Result;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tauri::Manager;

pub fn write_file(file_name: &str, content: &Value) -> Result<()> {
    let data = json!(content);
    File::create(file_name)?;
    fs::write(file_name, data.to_string())?;
    Ok(())
}

pub fn read_file(file_name: &str, default_value: Value) -> Result<String> {
    let mut buffer = String::new();
    let mut file = match File::open(file_name) {
        Ok(file) => file,
        Err(_) => {
            write_file(file_name, &default_value)?;
            File::open(file_name)?
        }
    };

    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}