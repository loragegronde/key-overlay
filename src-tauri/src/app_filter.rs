//! Foreground-app matching for app-scoped overlay visibility.
//!
//! When a filter is enabled the overlay is shown only while the focused
//! process name matches (case-insensitive substring). Manual hide via the
//! visibility hotkey still wins.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::OVERLAY_WINDOW;

static WATCHER_STARTED: AtomicBool = AtomicBool::new(false);
/// User has launched the overlay at least once (Go Live).
static OVERLAY_LIVE: AtomicBool = AtomicBool::new(false);
/// User wants the overlay visible (toggled by Ctrl+Shift+O).
static MANUAL_VISIBLE: AtomicBool = AtomicBool::new(true);
static FILTER_ENABLED: AtomicBool = AtomicBool::new(false);
static FILTER_MATCH: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForegroundApp {
    pub process_name: String,
    pub window_title: String,
}

pub fn set_overlay_live(live: bool) {
    OVERLAY_LIVE.store(live, Ordering::SeqCst);
}

pub fn set_manual_visible(visible: bool) {
    MANUAL_VISIBLE.store(visible, Ordering::SeqCst);
}

pub fn toggle_manual_visible() -> bool {
    let next = !MANUAL_VISIBLE.load(Ordering::SeqCst);
    MANUAL_VISIBLE.store(next, Ordering::SeqCst);
    next
}

pub fn is_overlay_live() -> bool {
    OVERLAY_LIVE.load(Ordering::SeqCst)
}

pub fn set_filter(enabled: bool, match_text: String) {
    FILTER_ENABLED.store(enabled, Ordering::SeqCst);
    if let Ok(mut guard) = FILTER_MATCH.lock() {
        *guard = match_text;
    }
}

fn filter_allows(foreground: &ForegroundApp) -> bool {
    if !FILTER_ENABLED.load(Ordering::SeqCst) {
        return true;
    }
    let needle = match FILTER_MATCH.lock() {
        Ok(g) => g.clone(),
        Err(_) => return true,
    };
    let needle = needle.trim().to_lowercase();
    if needle.is_empty() {
        return true;
    }
    foreground.process_name.to_lowercase().contains(&needle)
        || foreground.window_title.to_lowercase().contains(&needle)
}

/// Recomputes whether the overlay window should be shown.
pub fn apply_visibility(app: &AppHandle) {
    let Some(overlay) = app.get_webview_window(OVERLAY_WINDOW) else {
        return;
    };

    let should_show = OVERLAY_LIVE.load(Ordering::SeqCst)
        && MANUAL_VISIBLE.load(Ordering::SeqCst)
        && {
            let fg = get_foreground_app().unwrap_or(ForegroundApp {
                process_name: String::new(),
                window_title: String::new(),
            });
            filter_allows(&fg)
        };

    if should_show {
        let _ = overlay.show();
    } else {
        let _ = overlay.hide();
    }
}

pub fn start_watcher(app: AppHandle) {
    if WATCHER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(move || {
        let mut last_process = String::new();
        loop {
            thread::sleep(Duration::from_millis(250));

            let fg = match get_foreground_app() {
                Ok(fg) => fg,
                Err(_) => continue,
            };

            if fg.process_name != last_process {
                last_process = fg.process_name.clone();
                let _ = app.emit("foreground-app", &fg);
            }

            apply_visibility(&app);
        }
    });
}

#[tauri::command]
pub fn get_foreground_app() -> Result<ForegroundApp, String> {
    #[cfg(windows)]
    {
        windows_foreground()
    }
    #[cfg(not(windows))]
    {
        Ok(ForegroundApp {
            process_name: String::new(),
            window_title: String::new(),
        })
    }
}

#[cfg(windows)]
fn windows_foreground() -> Result<ForegroundApp, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return Err("no foreground window".into());
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return Err("could not resolve process id".into());
        }

        let title_len = GetWindowTextLengthW(hwnd);
        let mut title_buf = vec![0u16; (title_len + 1) as usize];
        let written = GetWindowTextW(hwnd, &mut title_buf);
        title_buf.truncate(written as usize);
        let window_title = OsString::from_wide(&title_buf)
            .to_string_lossy()
            .into_owned();

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|e| format!("OpenProcess failed: {e}"))?;

        let mut path_buf = vec![0u16; MAX_PATH as usize];
        let mut size = path_buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(path_buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(process);
        result.map_err(|e| format!("QueryFullProcessImageNameW failed: {e}"))?;

        path_buf.truncate(size as usize);
        let full_path = OsString::from_wide(&path_buf)
            .to_string_lossy()
            .into_owned();
        let process_name = full_path
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or(&full_path)
            .to_string();

        Ok(ForegroundApp {
            process_name,
            window_title,
        })
    }
}
