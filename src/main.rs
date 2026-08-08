//! Key Overlay — lightweight pure-Rust editor + transparent HUD.

mod color;
mod input;
mod model;
mod persist;
mod platform;
mod state;
mod templates;
mod ui;

use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use eframe::egui;

use crate::input::{start_listener, InputMsg};
use crate::state::AppState;
use crate::ui::editor::show_editor;
use crate::ui::overlay::show_overlay;

fn main() -> eframe::Result<()> {
    env_logger_stub();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Key Overlay")
            .with_inner_size([1180.0, 740.0])
            .with_min_inner_size([900.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Key Overlay",
        options,
        Box::new(|cc| Ok(Box::<KeyOverlayApp>::new(KeyOverlayApp::new(cc)))),
    )
}

fn env_logger_stub() {
    // Intentionally empty — keep binary lean; eprintln! used for diagnostics.
}

struct KeyOverlayApp {
    state: AppState,
    input_rx: Receiver<InputMsg>,
    last_native_flags: Instant,
    #[cfg(windows)]
    _hotkeys: Option<WindowsShell>,
}

impl KeyOverlayApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras_install(cc);
        platform::start_watcher();
        let input_rx = start_listener();
        let state = AppState::load();
        platform::set_filter(
            state.profile().target_app_enabled,
            state.profile().target_app_match.clone(),
        );

        #[cfg(windows)]
        let _hotkeys = WindowsShell::start();
        #[cfg(not(windows))]
        let _hotkeys = ();

        Self {
            state,
            input_rx,
            last_native_flags: Instant::now(),
            #[cfg(windows)]
            _hotkeys,
        }
    }
}

fn egui_extras_install(_cc: &eframe::CreationContext<'_>) {}

impl eframe::App for KeyOverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.input_rx.try_recv() {
            self.state.handle_input(msg);
        }
        self.state.tick_kps();
        self.state.autosave_tick();

        // Soft-poll hotkey / visibility intents from platform shell.
        poll_global_actions(ctx, &mut self.state);

        show_editor(ctx, &mut self.state);

        if self.state.overlay_open
            || platform::SHOULD_SHOW_OVERLAY.load(std::sync::atomic::Ordering::SeqCst)
            || platform::is_positioning()
        {
            show_overlay(ctx, &mut self.state);
        }

        if self.last_native_flags.elapsed() > Duration::from_millis(200) {
            platform::apply_native_window_flags("Key Overlay HUD");
            self.last_native_flags = Instant::now();
        }

        // Continuous repaint while keys may be held / sticks move.
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.state.flush_save();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}

fn poll_global_actions(ctx: &egui::Context, state: &mut AppState) {
    #[cfg(windows)]
    {
        while let Some(action) = WindowsShell::poll() {
            match action {
                ShellAction::ToggleVisibility => {
                    platform::toggle_manual_visible();
                }
                ShellAction::LockOverlay => {
                    platform::finish_positioning();
                    state.flash("Overlay locked (click-through)");
                }
                ShellAction::FocusEditor => {
                    state.editor_request_focus = true;
                }
            }
        }
    }

    // Editor-focused fallbacks (also cover Linux/macOS where global-hotkey isn't wired).
    ctx.input(|i| {
        let chord = i.modifiers.ctrl && i.modifiers.shift;
        if chord && i.key_pressed(egui::Key::O) {
            platform::toggle_manual_visible();
        }
        if chord && i.key_pressed(egui::Key::L) {
            platform::finish_positioning();
        }
        if chord && i.key_pressed(egui::Key::E) {
            state.editor_request_focus = true;
        }
    });

    if state.editor_request_focus {
        state.editor_request_focus = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
    }
}

#[cfg(windows)]
enum ShellAction {
    ToggleVisibility,
    LockOverlay,
    FocusEditor,
}

#[cfg(windows)]
struct WindowsShell {
    // Keep manager alive.
    _mgr: global_hotkey::GlobalHotKeyManager,
}

#[cfg(windows)]
impl WindowsShell {
    fn start() -> Option<Self> {
        use global_hotkey::hotkey::{Code, HotKey, Modifiers};
        use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

        let mgr = GlobalHotKeyManager::new().ok()?;
        let o = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyO);
        let l = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyL);
        let e = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyE);
        mgr.register(o).ok()?;
        mgr.register(l).ok()?;
        mgr.register(e).ok()?;

        // Store codes for matching in poll via thread-local / once_cell.
        HOTKEY_IDS.lock().replace(HotkeyIds {
            toggle: o.id(),
            lock: l.id(),
            editor: e.id(),
        });

        // Drain receiver in poll using GlobalHotKeyEvent::receiver
        let _ = GlobalHotKeyEvent::receiver();
        let _ = HotKeyState::Pressed;

        // Tray icon (best-effort).
        if let Ok(icon) = load_tray_icon() {
            let tray = tray_icon::TrayIconBuilder::new()
                .with_tooltip("Key Overlay")
                .with_icon(icon)
                .with_menu(&tray_icon::menu::Menu::new())
                .build();
            let _ = tray;
        }

        Some(Self { _mgr: mgr })
    }

    fn poll() -> Option<ShellAction> {
        use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
        let ids = HOTKEY_IDS.lock().clone()?;
        let rx = GlobalHotKeyEvent::receiver();
        while let Ok(ev) = rx.try_recv() {
            if ev.state != HotKeyState::Pressed {
                continue;
            }
            if ev.id == ids.toggle {
                return Some(ShellAction::ToggleVisibility);
            }
            if ev.id == ids.lock {
                return Some(ShellAction::LockOverlay);
            }
            if ev.id == ids.editor {
                return Some(ShellAction::FocusEditor);
            }
        }
        None
    }
}

#[cfg(windows)]
#[derive(Clone)]
struct HotkeyIds {
    toggle: u32,
    lock: u32,
    editor: u32,
}

#[cfg(windows)]
static HOTKEY_IDS: once_cell::sync::Lazy<parking_lot::Mutex<Option<HotkeyIds>>> =
    once_cell::sync::Lazy::new(|| parking_lot::Mutex::new(None));

#[cfg(windows)]
fn load_tray_icon() -> Result<tray_icon::Icon, ()> {
    // 32x32 cyan square fallback.
    let size = 32;
    let mut rgba = vec![0u8; size * size * 4];
    for px in rgba.chunks_exact_mut(4) {
        px[0] = 34;
        px[1] = 211;
        px[2] = 238;
        px[3] = 255;
    }
    tray_icon::Icon::from_rgba(rgba, size as u32, size as u32).map_err(|_| ())
}
