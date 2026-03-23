mod cli;
mod file;
mod hermenia;
mod jules;

use serde_json::json;
use crate::cli::Rumi;
use jules::{download_model, invoke_llama_cli, model_exists};
use tauri::Manager;
use serde_json::Value;

#[tauri::command]
async fn prompt(prompt: String) {
    match invoke_llama_cli(&prompt, true).await {
        Ok(_) => println!("External process executed successfully"),
        Err(e) => eprintln!("Error executing external process: {}", e),
    };
}

#[tauri::command]
async fn dispatch(
    _app: tauri::AppHandle,
    event: String,
    payload: Option<String>,
    // rumi: tauri::State<Rumi>,
) -> String {
    println!("Dispatching event: {}", event);

    match event.as_str() {
        "get_exchanges" => {
            let mut rumi = Rumi::new();
            let exchanges = rumi.get_exchanges();
            return serde_json::to_string(&exchanges).unwrap()
        }
        _ => {
            println!("Unknown command: {}", event);
        }
    }

    // extract payload.prompt
    let prompt = serde_json::from_str::<Value>(payload.as_deref().unwrap_or("{}")).unwrap()["prompt"].as_str().unwrap_or("").to_string();
    println!("Dispatching prompt: {}", prompt);

    let mut rumi = Rumi::new();

    let response = rumi.chat(prompt.clone(), true, None).await;
    println!("Response: {}", response["response"]);

    serde_json::to_string(&json!({"prompt": prompt.clone(), "response": response["response"].clone(), "conversation": response["conversation"].clone()})).unwrap()

    // let hydrated_event = hydrate_event(event.clone(), payload.as_deref().unwrap_or("{}"));
    // let data = machine.other_consume(hydrated_event);
    // serde_json::to_string(&data).unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut rumi = Rumi::new();

    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(rumi);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![prompt, dispatch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
