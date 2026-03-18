use std::sync::Mutex;

use tauri::{Emitter, Manager};

mod capture;

/// Shared state holding the last captured screenshot as base64 PNG
struct CaptureState {
    last_screenshot_b64: Option<String>,
    last_question: Option<String>,
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn submit_question(app: tauri::AppHandle, question: String) -> Result<String, String> {
    // Hide the input bar before capturing so it doesn't appear in the screenshot
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    // Small delay to ensure window is hidden before capture
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Capture the screen
    let screenshot_b64 = capture::capture_primary_screen()
        .map_err(|e| format!("Screen capture failed: {}", e))?;

    // Store in state for later use by AI pipeline
    let state = app.state::<Mutex<CaptureState>>();
    {
        let mut state = state.lock().unwrap();
        state.last_screenshot_b64 = Some(screenshot_b64.clone());
        state.last_question = Some(question.clone());
    }

    println!(
        "Captured screenshot ({} bytes base64) for question: {}",
        screenshot_b64.len(),
        question
    );

    // Emit event so frontend can show confirmation
    let _ = app.emit("capture-complete", &question);

    Ok(screenshot_b64)
}

#[tauri::command]
fn get_last_capture(app: tauri::AppHandle) -> Option<String> {
    let state = app.state::<Mutex<CaptureState>>();
    let state = state.lock().unwrap();
    state.last_screenshot_b64.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(CaptureState {
            last_screenshot_b64: None,
            last_question: None,
        }))
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
