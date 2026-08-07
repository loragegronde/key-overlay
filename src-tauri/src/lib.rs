mod app_filter;
mod commands;
mod input_listener;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    DeviceEventFilter, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// Transparent always-on-top HUD. Input events are emitted here.
pub const OVERLAY_WINDOW: &str = "overlay";
/// Decorated editor where layouts are built.
pub const EDITOR_WINDOW: &str = "editor";

// Back-compat alias used by older comments/docs in the tree.
pub const MAIN_WINDOW: &str = OVERLAY_WINDOW;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ctrl+Shift+O — show/hide the live overlay (respects app filter).
    // Ctrl+Shift+L — lock/unlock click-through on the overlay (or finish placing).
    // Ctrl+Shift+E — reopen the editor.
    let visibility_shortcut =
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyO);
    let lock_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyL);
    let editor_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyE);

    tauri::Builder::default()
        // Without this, on Windows a focused Tauri/WebView2 window swallows
        // keyboard events before rdev's low-level hook sees them — mouse still
        // works, which is exactly the "can bind mouse but not keys" bug.
        // See tauri-apps/tauri#14770.
        .device_event_filter(DeviceEventFilter::Always)
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler({
                    let visibility = visibility_shortcut.clone();
                    let lock = lock_shortcut.clone();
                    let editor = editor_shortcut.clone();
                    move |app, shortcut, event| {
                        if event.state() != ShortcutState::Pressed {
                            return;
                        }
                        if shortcut == &visibility {
                            if !app_filter::is_overlay_live() {
                                return;
                            }
                            app_filter::toggle_manual_visible();
                            app_filter::apply_visibility(app);
                        } else if shortcut == &lock {
                            let _ = app.emit("hotkey-toggle-lock", ());
                        } else if shortcut == &editor {
                            if let Some(window) = app.get_webview_window(EDITOR_WINDOW) {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(move |app| {
            if app.get_webview_window(OVERLAY_WINDOW).is_none() {
                eprintln!(
                    "warning: no window labelled '{OVERLAY_WINDOW}' — input events will go nowhere"
                );
            }

            app.global_shortcut().register(visibility_shortcut)?;
            app.global_shortcut().register(lock_shortcut)?;
            app.global_shortcut().register(editor_shortcut)?;

            input_listener::start_input_listener(app.handle().clone())?;
            app_filter::start_watcher(app.handle().clone());

            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_overlay =
                MenuItem::with_id(app, "show_overlay", "Show Overlay", true, None::<&str>)?;
            let show_editor =
                MenuItem::with_id(app, "show_editor", "Open Editor", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_editor, &show_overlay, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show_editor" => {
                        if let Some(window) = app.get_webview_window(EDITOR_WINDOW) {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "show_overlay" => {
                        if app_filter::is_overlay_live() {
                            app_filter::set_manual_visible(true);
                            app_filter::apply_visibility(app);
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window(EDITOR_WINDOW) {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            input_listener::start_input_listener,
            input_listener::stop_input_listener,
            commands::toggle_click_through,
            commands::set_always_on_top,
            commands::minimize_window,
            commands::quit_app,
            commands::close_editor,
            commands::open_editor,
            commands::launch_overlay,
            commands::finish_positioning,
            commands::set_app_filter,
            commands::save_layout,
            commands::load_layout,
            commands::export_profile,
            commands::import_profile,
            app_filter::get_foreground_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
