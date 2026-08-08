//! Fallback key/mouse capture from egui events (needed when rdev is silent).

use egui::{Event, Key, PointerButton};

use crate::input::{InputAction, InputMsg};
use crate::state::AppState;

/// While rebinding, consume egui key/mouse-down events into a binding.
pub fn capture_from_egui(ctx: &egui::Context, state: &mut AppState) {
    if state.capturing.is_none() || state.bind_suppressed() {
        return;
    }

    let mut bind: Option<(String, String)> = None;

    ctx.input(|i| {
        for ev in &i.events {
            match ev {
                Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } => {
                    if *key == Key::Escape {
                        continue;
                    }
                    if let Some(mapped) = map_egui_key(*key) {
                        bind = Some(mapped);
                        break;
                    }
                }
                Event::PointerButton {
                    button,
                    pressed: true,
                    ..
                } => {
                    bind = match button {
                        PointerButton::Primary => Some(("Mouseleft".into(), "LMB".into())),
                        PointerButton::Secondary => Some(("Mouseright".into(), "RMB".into())),
                        PointerButton::Middle => Some(("Mousemiddle".into(), "MMB".into())),
                        _ => None,
                    };
                    if bind.is_some() {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    if let Some((code, label)) = bind {
        state.handle_input(InputMsg::Key {
            code,
            label,
            action: InputAction::Down,
            timestamp: 0,
        });
    }
}

fn map_egui_key(key: Key) -> Option<(String, String)> {
    let (code, label): (&str, &str) = match key {
        Key::A => ("KeyA", "A"),
        Key::B => ("KeyB", "B"),
        Key::C => ("KeyC", "C"),
        Key::D => ("KeyD", "D"),
        Key::E => ("KeyE", "E"),
        Key::F => ("KeyF", "F"),
        Key::G => ("KeyG", "G"),
        Key::H => ("KeyH", "H"),
        Key::I => ("KeyI", "I"),
        Key::J => ("KeyJ", "J"),
        Key::K => ("KeyK", "K"),
        Key::L => ("KeyL", "L"),
        Key::M => ("KeyM", "M"),
        Key::N => ("KeyN", "N"),
        Key::O => ("KeyO", "O"),
        Key::P => ("KeyP", "P"),
        Key::Q => ("KeyQ", "Q"),
        Key::R => ("KeyR", "R"),
        Key::S => ("KeyS", "S"),
        Key::T => ("KeyT", "T"),
        Key::U => ("KeyU", "U"),
        Key::V => ("KeyV", "V"),
        Key::W => ("KeyW", "W"),
        Key::X => ("KeyX", "X"),
        Key::Y => ("KeyY", "Y"),
        Key::Z => ("KeyZ", "Z"),
        Key::Num0 => ("Digit0", "0"),
        Key::Num1 => ("Digit1", "1"),
        Key::Num2 => ("Digit2", "2"),
        Key::Num3 => ("Digit3", "3"),
        Key::Num4 => ("Digit4", "4"),
        Key::Num5 => ("Digit5", "5"),
        Key::Num6 => ("Digit6", "6"),
        Key::Num7 => ("Digit7", "7"),
        Key::Num8 => ("Digit8", "8"),
        Key::Num9 => ("Digit9", "9"),
        Key::Space => ("Space", "Space"),
        Key::Enter => ("Enter", "↵"),
        Key::Tab => ("Tab", "Tab"),
        Key::Backspace => ("Backspace", "⌫"),
        Key::Delete => ("Delete", "Del"),
        Key::ArrowUp => ("ArrowUp", "↑"),
        Key::ArrowDown => ("ArrowDown", "↓"),
        Key::ArrowLeft => ("ArrowLeft", "←"),
        Key::ArrowRight => ("ArrowRight", "→"),
        Key::Home => ("Home", "Home"),
        Key::End => ("End", "End"),
        Key::PageUp => ("PageUp", "PgUp"),
        Key::PageDown => ("PageDown", "PgDn"),
        Key::Insert => ("Insert", "Ins"),
        Key::F1 => ("F1", "F1"),
        Key::F2 => ("F2", "F2"),
        Key::F3 => ("F3", "F3"),
        Key::F4 => ("F4", "F4"),
        Key::F5 => ("F5", "F5"),
        Key::F6 => ("F6", "F6"),
        Key::F7 => ("F7", "F7"),
        Key::F8 => ("F8", "F8"),
        Key::F9 => ("F9", "F9"),
        Key::F10 => ("F10", "F10"),
        Key::F11 => ("F11", "F11"),
        Key::F12 => ("F12", "F12"),
        Key::Minus => ("Minus", "-"),
        Key::Equals => ("Equal", "="),
        Key::OpenBracket => ("BracketLeft", "["),
        Key::CloseBracket => ("BracketRight", "]"),
        Key::Backslash => ("Backslash", "\\"),
        Key::Semicolon => ("Semicolon", ";"),
        Key::Quote => ("Quote", "'"),
        Key::Comma => ("Comma", ","),
        Key::Period => ("Period", "."),
        Key::Slash => ("Slash", "/"),
        Key::Backtick => ("Backquote", "`"),
        Key::Escape => return None,
        _ => return None,
    };
    Some((code.into(), label.into()))
}
