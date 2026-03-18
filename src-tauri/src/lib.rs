use tauri::Manager;

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn submit_question(app: tauri::AppHandle, question: String) {
    // Hide the input bar after submission
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    // TODO: DW-4 will handle passing `question` to the capture pipeline
    println!("Question submitted: {}", question);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Start with window hidden
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Code, Modifiers, ShortcutState,
                };

                let shortcut =
                    tauri_plugin_global_shortcut::Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyH);

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
        .invoke_handler(tauri::generate_handler![hide_window, submit_question])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
