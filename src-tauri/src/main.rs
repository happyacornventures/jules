// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod jules;

use jules::{model_exists, download_model, invoke_llama_cli};

#[tokio::main]
async fn main() {
  let args: Vec<String> = std::env::args().collect();

  if !model_exists("models") {
    if let Err(e) = download_model("models", "https://huggingface.co/Qwen/Qwen2-1.5B-Instruct-GGUF/resolve/main/qwen2-1_5b-instruct-q4_0.gguf?download=true").await {
      eprintln!("Error downloading model: {}", e);
      std::process::exit(1);
    }
  }

  if args.len() > 1 {
    // Check if --stream flag is present
    let stream = args.contains(&"--stream".to_string());

    // Check if --context flag is present and capture its value
    let context = args.iter()
      .position(|arg| arg.starts_with("--context="))
      .and_then(|i| args[i].strip_prefix("--context="))
      .map(|s| s.to_string());

    if let Some(ctx) = &context {
      println!("Context: {}", ctx);
    }

    // Find the prompt (first non-flag argument)
    let prompt = args.iter()
      .skip(1)
      .find(|arg| !arg.starts_with("--"))
      .map(|s| s.as_str())
      .unwrap_or("");

    // pass arg as query to invoke_llama_cli
    match invoke_llama_cli(prompt, stream).await {
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
      },
      Ok(None) => println!("No output from process."),
      Err(e) => eprintln!("Error executing external process: {}", e),
    }

    println!("Hello World");
  } else {
    app_lib::run();
  }
}
