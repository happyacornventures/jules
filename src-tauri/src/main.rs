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

fn state_identity(state: Value, event: Value) -> Value {
    state
}

fn persist_events(key: &str, value: &Value, event: &Value) {
    let existing_events_str = read_file("exchanges.json", json!({})).unwrap();
    let mut events: HashMap<String, Value> = serde_json::from_str(&existing_events_str).unwrap();
    let event_id = event["id"].as_str().unwrap().to_string();
    events.insert(event_id, event.clone());
    write_file("exchanges.json", &json!(events)).expect("Failed to write to events file");
}

fn exchange_reducer(state: Value, event: Value) -> Value {
    let mut new_state = state.clone();

    match event["type"].as_str().unwrap() {
        "exchange_created" => {
            new_state.as_object_mut().unwrap().insert(
                event["id"].as_str().unwrap().to_string(),
                event["payload"].clone(),
            );
            return new_state;
        }
        _ => {
            println!("Unknown command: {}", event["type"].as_str().unwrap());
        }
    }
    state
}

fn hydrate_event(event: &Value) -> Value {
    let id = Uuid::new_v4().to_string();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;

    let mut new_event = event.as_object().cloned().unwrap_or_default();
    new_event.insert("id".to_string(), json!(id));
    new_event.insert("createTime".to_string(), json!(timestamp));

    // println!("hydrated event {:?}", json!(new_event));

    json!(new_event)
}

fn conversation_interpreter(event: &Value) -> Value {
    if event["payload"]
        .as_object()
        .and_then(|p| p.get("conversation"))
        .map_or(true, |v| v.is_null())
    {
        let mut new_event = event.clone();
        if let Some(payload) = new_event.get_mut("payload").and_then(|p| p.as_object_mut()) {
            payload.insert(
                "conversation".to_string(),
                json!(Uuid::new_v4().to_string()),
            );
        }
        return json!(new_event);
    }
    event.clone()
}

fn timestamp_interpreter(event: &Value) -> Value {
    let mut new_event = event.clone();
    if let Some(payload) = new_event.get_mut("payload").and_then(|p| p.as_object_mut()) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        payload.insert("createTime".to_string(), json!(timestamp));
    }
    json!(new_event)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        run_cli(args).await;
    } else {
        app_lib::run();
    }
}
