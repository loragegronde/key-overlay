//! Key Overlay — lightweight pure-Rust editor + transparent HUD.

// Release Windows builds are a real GUI app (no console). Closing PowerShell
// after a detached launch will not kill the process.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod color;
mod hud_control;
mod input;
mod model;
mod persist;
mod platform;
mod state;
mod templates;
mod ui;

use std::sync::mpsc::Receiver;
use std::time::Duration;

use eframe::egui;

use crate::input::{apply_egui_presses, start_listener, InputMsg};
use crate::state::AppState;
use crate::ui::editor::show_editor;
use crate::ui::overlay::run_overlay;

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--overlay") {
        return run_overlay();
    }
    run_editor()
}

fn run_editor() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Key Overlay")
            .with_inner_size([1180.0, 740.0])
            .with_min_inner_size([900.0, 560.0]),
        // glow: required for transparent HUD process; editor stays opaque.
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Key Overlay",
        options,
        Box::new(|cc| Ok(Box::<EditorApp>::new(EditorApp::new(cc)))),
    )
}

struct EditorApp {
    state: AppState,
    input_rx: Receiver<InputMsg>,
    #[cfg(windows)]
    _hotkeys: Option<WindowsShell>,
}

impl EditorApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        platform::start_watcher();
        let input_rx = start_listener();
        let state = AppState::load();
        platform::set_filter(
            state.profile().target_app_enabled,
            state.profile().target_app_match.clone(),
        );

        #[cfg(windows)]
        let _hotkeys = WindowsShell::start();

        Self {
            state,
            input_rx,
            #[cfg(windows)]
            _hotkeys,
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.input_rx.try_recv() {
            self.state.handle_input(msg);
        }
        apply_egui_presses(ctx, &mut self.state);
        self.state.autosave_tick();
        poll_global_actions(ctx, &mut self.state);
        show_editor(ctx, &mut self.state);
        let busy = !self.state.active_keys.is_empty() || self.state.capturing.is_some();
        ctx.request_repaint_after(Duration::from_millis(if busy { 8 } else { 16 }));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.state.flush_save();
    }
}

fn poll_global_actions(ctx: &egui::Context, state: &mut AppState) {
    #[cfg(windows)]
    {
        while let Some(action) = WindowsShell::poll() {
            match action {
                ShellAction::ToggleVisibility => {
                    let ctrl = hud_control::toggle_visible();
                    platform::set_manual_visible(ctrl.visible);
                    state.flash(if ctrl.visible {
                        "Overlay shown"
                    } else {
                        "Overlay hidden"
                    });
                }
                ShellAction::ToggleLock => {
                    let ctrl = hud_control::toggle_lock();
                    state.flash(if ctrl.locked {
                        "Overlay locked (click-through)"
                    } else {
                        "Overlay unlocked — drag to reposition"
                    });
                }
                ShellAction::FocusEditor => {
                    state.editor_request_focus = true;
                }
            }
        }
    }

    ctx.input(|i| {
        let chord = i.modifiers.ctrl && i.modifiers.shift;
        if chord && i.key_pressed(egui::Key::O) {
            let ctrl = hud_control::toggle_visible();
            platform::set_manual_visible(ctrl.visible);
            state.flash(if ctrl.visible {
                "Overlay shown"
            } else {
                "Overlay hidden"
            });
        }
        if chord && i.key_pressed(egui::Key::L) {
            let ctrl = hud_control::toggle_lock();
            state.flash(if ctrl.locked {
                "Overlay locked (click-through)"
            } else {
                "Overlay unlocked — drag to reposition"
            });
        }
        if chord && i.key_pressed(egui::Key::E) {
            state.editor_request_focus = true;
        }
    });

    if state.editor_request_focus {
        state.editor_request_focus = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }
}

#[cfg(windows)]
enum ShellAction {
    ToggleVisibility,
    ToggleLock,
    FocusEditor,
}

#[cfg(windows)]
struct WindowsShell {
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

        HOTKEY_IDS.lock().replace(HotkeyIds {
            toggle: o.id(),
            lock: l.id(),
            editor: e.id(),
        });

        let _ = GlobalHotKeyEvent::receiver();
        let _ = HotKeyState::Pressed;

        if let Ok(icon) = load_tray_icon() {
            let _ = tray_icon::TrayIconBuilder::new()
                .with_tooltip("Key Overlay")
                .with_icon(icon)
                .build();
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
                return Some(ShellAction::ToggleLock);
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
