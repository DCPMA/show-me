use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

mod capture;
mod openrouter;

const DEFAULT_MODEL: &str = "anthropic/claude-sonnet-4";

/// Shared state holding the last captured screenshot as base64 PNG
struct CaptureState {
    last_screenshot_b64: Option<String>,
    last_question: Option<String>,
}

/// User-configurable settings
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    api_key: String,
    model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: DEFAULT_MODEL.to_string(),
        }
    }
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
async fn submit_question(
    app: tauri::AppHandle,
    question: String,
) -> Result<openrouter::AnalysisResponse, String> {
    // Hide the input bar before capturing so it doesn't appear in the screenshot
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    // Small delay to ensure window is hidden before capture
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // Capture the screen
    let screenshot_b64 = capture::capture_primary_screen()
        .map_err(|e| format!("Screen capture failed: {}", e))?;

    // Store in state
    {
        let state = app.state::<Mutex<CaptureState>>();
        let mut state = state.lock().unwrap();
        state.last_screenshot_b64 = Some(screenshot_b64.clone());
        state.last_question = Some(question.clone());
    }

    let _ = app.emit("capture-complete", &question);

    // Get settings
    let (api_key, model) = {
        let settings = app.state::<Mutex<Settings>>();
        let settings = settings.lock().unwrap();
        (settings.api_key.clone(), settings.model.clone())
    };

    if api_key.is_empty() {
        return Err("No API key configured. Press Cmd+Shift+H, then click the gear icon to add your OpenRouter API key.".into());
    }

    // Call OpenRouter API
    let _ = app.emit("analysis-started", &question);

    let analysis = openrouter::analyze_screenshot(&api_key, &model, &screenshot_b64, &question)
        .await
        .map_err(|e| e.to_string())?;

    let _ = app.emit("analysis-complete", &analysis);

    Ok(analysis)
}

#[tauri::command]
fn get_last_capture(app: tauri::AppHandle) -> Option<String> {
    let state = app.state::<Mutex<CaptureState>>();
    let state = state.lock().unwrap();
    state.last_screenshot_b64.clone()
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Settings {
    let settings = app.state::<Mutex<Settings>>();
    let settings = settings.lock().unwrap();
    settings.clone()
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, api_key: String, model: String) {
    let state = app.state::<Mutex<Settings>>();
    let mut settings = state.lock().unwrap();
    settings.api_key = api_key;
    if !model.is_empty() {
        settings.model = model;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(CaptureState {
            last_screenshot_b64: None,
            last_question: None,
        }))
        .manage(Mutex::new(Settings::default()))
        .setup(|app| {
            // Start with window hidden
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

                let shortcut = tauri_plugin_global_shortcut::Shortcut::new(
                    Some(Modifiers::SUPER | Modifiers::SHIFT),
                    Code::KeyH,
                );

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcut(shortcut)?
                        .with_handler(move |app, _shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                if let Some(window) = app.get_webview_window("main") {
                                    if window.is_visible().unwrap_or(false) {
                                        let _ = window.hide();
                                    } else {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        })
                        .build(),
                )?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hide_window,
            submit_question,
            get_last_capture,
            get_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
