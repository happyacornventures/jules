// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Error;
use std::io::Read;
use uuid::Uuid;

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

fn exchange_reducer(state: Value, event: Value) -> Value {
    let mut new_state = state.clone();

    match event["type"].as_str().unwrap() {
        "exchange_created" => {
            let mut payload = event["payload"].clone();
            if payload
                .as_object()
                .and_then(|p| p.get("conversation"))
                .is_none()
            {
                let conversation = Uuid::new_v4().to_string();
                payload
                    .as_object_mut()
                    .unwrap()
                    .insert("conversation".to_string(), json!(conversation));
            }
            new_state
                .as_object_mut()
                .unwrap()
                .insert(event["id"].as_str().unwrap().to_string(), payload);
            // println!("New state: {:?}", new_state);
            return new_state;
        }
        _ => {
            println!("Unknown command: {}", event["type"].as_str().unwrap());
        }
    }
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

    let data: HashMap<String, Value> = HashMap::from([("exchanges".to_string(), json!({}))]);
    let mut listeners: Vec<Box<dyn Fn(&str, &Value, &Value) + Send + Sync>> = Vec::new();
    let reducers: HashMap<String, (Value, fn(Value, Value) -> Value)> = HashMap::from([(
        "exchanges".to_string(),
        (json!({}), exchange_reducer as fn(Value, Value) -> Value),
    )]);

    let machine = Machine::new(data, reducers, Mutex::new(std::mem::take(&mut listeners)));

    let events_str = read_file("exchanges.json", json!({})).unwrap();

    let events: HashMap<String, Value> = serde_json::from_str(&events_str).unwrap();
    let mut sorted_events: Vec<_> = events.values().collect();
    sorted_events.sort_by_key(|e| e["createTime"].as_u64());

    for event in sorted_events {
        let event_type = event["type"].as_str().unwrap().to_string();
        let payload = event["payload"].to_string();
        machine.consume(event_type, Some(payload));
    }

    if args.len() > 1 {
        // Check if --stream flag is present
        let stream = args.contains(&"--stream".to_string());

        // Check if --context flag is present and capture its value
        let context = args
            .iter()
            .position(|arg| arg.starts_with("--context="))
            .and_then(|i| args[i].strip_prefix("--context="))
            .map(|s| s.to_string());

        // Check if --context flag is present and capture its value
        let convo_id = args
            .iter()
            .position(|arg| arg.starts_with("--conversation="))
            .and_then(|i| args[i].strip_prefix("--conversation="))
            .map(|s| s.to_string());

        let exchanges: String = machine.consume("exchanges_requested".to_string(), None);

        let exchanges_map: HashMap<String, Value> = serde_json::from_str(&exchanges).unwrap();
        let mut exchanges_iter = exchanges_map.iter();

        let mut relevant_exchanges: Vec<Value> = Vec::new();

        if let Some(convo_id) = convo_id {
            for (_, exchange) in exchanges_iter {
                if exchange["conversation"].as_str().unwrap() == convo_id {
                    relevant_exchanges.push(exchange.clone());
                }
            }
        }

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
                let mut aggregated_output = String::new();
                let mut buffer = String::new();
                use std::io::BufRead;

                while buf_reader.read_line(&mut buffer).unwrap() > 0 {
                    if !buffer.trim().starts_with("> EOF by user") && !buffer.trim().is_empty() {
                        print!("{}", buffer);
                        aggregated_output.push_str(&buffer);
                        aggregated_output.push('\n');
                    }
                    buffer.clear();
                }

                machine.consume(
                    "exchange_created".to_string(),
                    Some(json!({"prompt": full_prompt, "response": aggregated_output}).to_string()),
                );
            }
            Ok(None) => println!("No output from process."),
            Err(e) => eprintln!("Error executing external process: {}", e),
        }
    } else {
        app_lib::run();
    }
}
