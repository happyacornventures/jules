// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Error;
use std::io::Read;

mod file;
mod hermenia;
mod jules;

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

fn state_identity(state: Value, event: Value) -> Value {
    // This function is a placeholder for state that does not change
    // It simply returns the state as is, without modification
    println!("State identity called with event: {}", event);
    state
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if !model_exists("models") {
        if let Err(e) = download_model("models", "https://huggingface.co/Qwen/Qwen2-1.5B-Instruct-GGUF/resolve/main/qwen2-1_5b-instruct-q4_0.gguf?download=true").await {
      eprintln!("Error downloading model: {}", e);
      std::process::exit(1);
    }
    }

    let data: HashMap<String, Value> = HashMap::from([("messages".to_string(), json!({}))]);
    let mut listeners: Vec<Box<dyn Fn(&str, &Value, &Value) + Send + Sync>> = Vec::new();
    let reducers: HashMap<String, (Value, fn(Value, Value) -> Value)> = HashMap::from([(
        "messages".to_string(),
        (json!({}), state_identity as fn(Value, Value) -> Value),
    )]);

    let machine = Machine::new(data, reducers, Mutex::new(std::mem::take(&mut listeners)));

    if args.len() > 1 {
        // Check if --stream flag is present
        let stream = args.contains(&"--stream".to_string());

        // Check if --context flag is present and capture its value
        let context = args
            .iter()
            .position(|arg| arg.starts_with("--context="))
            .and_then(|i| args[i].strip_prefix("--context="))
            .map(|s| s.to_string());

        let mut context_content: String = String::new();

        if let Some(ctx) = &context {
            println!("Context: {}", ctx);
            context_content = read_file(ctx, json!({})).unwrap_or_else(|e| {
                eprintln!("Error reading context file: {}", e);
                String::new()
            });
        }

        // alternate between user and assistant tags for each line in context
        // let conversation: String = context_content.split("\n")
        //   .filter(|line| !line.trim().is_empty())
        //   .collect::<Vec<&str>>().into_iter().map(|line| format!("<|im_start|>user\n{}<|im_end|>", line)).collect::<String>();

        // println!("{}", conversation);

        // Find the prompt (first non-flag argument)
        let prompt = args
            .iter()
            .skip(1)
            .find(|arg| !arg.starts_with("--"))
            .map(|s| s.as_str())
            .unwrap_or("");

        let full_prompt = if context_content.is_empty() {
            prompt.to_string()
        } else {
            format!("{}\n\n{}", context_content, prompt)
        };

        // pass arg as query to invoke_llama_cli
        match invoke_llama_cli(&full_prompt, stream).await {
            Ok(Some(reader)) => {
                let mut buf_reader = reader;
                let mut buffer = String::new();
                use std::io::BufRead;

                while buf_reader.read_line(&mut buffer).unwrap() > 0 {
                    if !buffer.trim().starts_with("> EOF by user") && !buffer.trim().is_empty() {
                        print!("{}", buffer);
                    }
                    buffer.clear();
                }
            }
            Ok(None) => println!("No output from process."),
            Err(e) => eprintln!("Error executing external process: {}", e),
        }
    } else {
        app_lib::run();
    }
}
