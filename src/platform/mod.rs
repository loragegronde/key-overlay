//! OS-specific helpers: click-through overlay + foreground app filter.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use once_cell::sync::Lazy;

static OVERLAY_LIVE: AtomicBool = AtomicBool::new(false);
static MANUAL_VISIBLE: AtomicBool = AtomicBool::new(true);
static FILTER_ENABLED: AtomicBool = AtomicBool::new(false);
static POSITIONING: AtomicBool = AtomicBool::new(false);
static CLICK_THROUGH: AtomicBool = AtomicBool::new(false);
static FILTER_MATCH: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static WATCHER: AtomicBool = AtomicBool::new(false);
/// Shared flag for UI: whether overlay viewport should be shown.
pub static SHOULD_SHOW_OVERLAY: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Default)]
pub struct ForegroundApp {
    pub process_name: String,
    pub window_title: String,
}

pub fn set_overlay_live(live: bool) {
    OVERLAY_LIVE.store(live, Ordering::SeqCst);
    recompute();
}

pub fn set_manual_visible(visible: bool) {
    MANUAL_VISIBLE.store(visible, Ordering::SeqCst);
    recompute();
}

pub fn toggle_manual_visible() -> bool {
    let next = !MANUAL_VISIBLE.load(Ordering::SeqCst);
    MANUAL_VISIBLE.store(next, Ordering::SeqCst);
    recompute();
    next
}

#[allow(dead_code)]
pub fn is_manual_visible() -> bool {
    MANUAL_VISIBLE.load(Ordering::SeqCst)
}

pub fn set_filter(enabled: bool, match_text: String) {
    FILTER_ENABLED.store(enabled, Ordering::SeqCst);
    if let Ok(mut g) = FILTER_MATCH.lock() {
        *g = match_text;
    }
    recompute();
}

pub fn set_positioning(enabled: bool) {
    POSITIONING.store(enabled, Ordering::SeqCst);
    if enabled {
        set_click_through(false);
    }
}

pub fn is_positioning() -> bool {
    POSITIONING.load(Ordering::SeqCst)
}

pub fn set_click_through(enabled: bool) {
    CLICK_THROUGH.store(enabled, Ordering::SeqCst);
}

#[allow(dead_code)]
pub fn is_click_through() -> bool {
    CLICK_THROUGH.load(Ordering::SeqCst)
}

pub fn finish_positioning() {
    POSITIONING.store(false, Ordering::SeqCst);
    set_click_through(true);
}

fn filter_allows(fg: &ForegroundApp) -> bool {
    if !FILTER_ENABLED.load(Ordering::SeqCst) {
        return true;
    }
    let needle = FILTER_MATCH
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    if needle.is_empty() {
        return true;
    }
    fg.process_name.to_lowercase().contains(&needle)
        || fg.window_title.to_lowercase().contains(&needle)
}

pub fn recompute() {
    let fg = get_foreground_app().unwrap_or_default();
    let show = OVERLAY_LIVE.load(Ordering::SeqCst)
        && MANUAL_VISIBLE.load(Ordering::SeqCst)
        && filter_allows(&fg);
    SHOULD_SHOW_OVERLAY.store(show, Ordering::SeqCst);
}

pub fn start_watcher() {
    if WATCHER.swap(true, Ordering::SeqCst) {
        return;
    }
    thread::spawn(|| loop {
        thread::sleep(Duration::from_millis(250));
        recompute();
    });
}

pub fn get_foreground_app() -> Result<ForegroundApp, String> {
    #[cfg(windows)]
    {
        windows_impl::foreground()
    }
    #[cfg(not(windows))]
    {
        Ok(ForegroundApp::default())
    }
}

/// Apply click-through / layered styles to a native HWND / X11 window when possible.
pub fn apply_native_window_flags(_window_title_hint: &str) {
    #[cfg(windows)]
    {
        windows_impl::apply_click_through_to_topmost();
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, SetWindowLongW,
        GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_TOOLWINDOW,
        FindWindowW, GetWindowLongW,
    };

    pub fn foreground() -> Result<ForegroundApp, String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return Err("no foreground window".into());
            }
            let mut title_buf = [0u16; 512];
            let title_len = GetWindowTextW(hwnd, &mut title_buf);
            let window_title = OsString::from_wide(&title_buf[..title_len as usize])
                .to_string_lossy()
                .into_owned();

            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            let process_name = process_name(pid).unwrap_or_default();
            Ok(ForegroundApp {
                process_name,
                window_title,
            })
        }
    }

    unsafe fn process_name(pid: u32) -> Option<String> {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; MAX_PATH as usize];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, Default::default(), PWSTR(buf.as_mut_ptr()), &mut size);
        let _ = CloseHandle(handle);
        if ok.is_err() {
            return None;
        }
        let path = OsString::from_wide(&buf[..size as usize])
            .to_string_lossy()
            .into_owned();
        Some(
            std::path::Path::new(&path)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or(path),
        )
    }

    pub fn apply_click_through_to_topmost() {
        unsafe {
            // Best-effort: find our overlay window by title.
            let title: Vec<u16> = "Key Overlay HUD\0".encode_utf16().collect();
            let hwnd = FindWindowW(None, windows::core::PCWSTR(title.as_ptr()));
            if hwnd.0.is_null() {
                return;
            }
            let mut style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            style |= WS_EX_LAYERED.0 as i32 | WS_EX_TOOLWINDOW.0 as i32;
            if super::is_click_through() && !super::is_positioning() {
                style |= WS_EX_TRANSPARENT.0 as i32;
            } else {
                style &= !(WS_EX_TRANSPARENT.0 as i32);
            }
            SetWindowLongW(hwnd, GWL_EXSTYLE, style);
            let _ = hwnd;
            let _ = HWND::default();
        }
    }
}
