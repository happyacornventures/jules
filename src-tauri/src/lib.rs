mod cli;
mod file;
mod hermenia;
mod jules;

use jules::{download_model, invoke_llama_cli, model_exists};

#[tauri::command]
async fn prompt(prompt: String) {
    match invoke_llama_cli(&prompt, true).await {
        Ok(_) => println!("External process executed successfully"),
        Err(e) => eprintln!("Error executing external process: {}", e),
    };
}

#[tauri::command]
fn dispatch(
    _app: tauri::AppHandle,
    event: String,
    payload: Option<String>,
    // machine: tauri::State<Machine>,
) -> String {
    println!("Dispatching event: {}", event);
    serde_json::to_string(&json!({})).unwrap()
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![prompt])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
