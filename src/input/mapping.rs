use std::borrow::Cow;

use rdev::{Button, Key};

pub fn key_code(key: Key) -> Cow<'static, str> {
    let code = match key {
        Key::KeyA => "KeyA",
        Key::KeyB => "KeyB",
        Key::KeyC => "KeyC",
        Key::KeyD => "KeyD",
        Key::KeyE => "KeyE",
        Key::KeyF => "KeyF",
        Key::KeyG => "KeyG",
        Key::KeyH => "KeyH",
        Key::KeyI => "KeyI",
        Key::KeyJ => "KeyJ",
        Key::KeyK => "KeyK",
        Key::KeyL => "KeyL",
        Key::KeyM => "KeyM",
        Key::KeyN => "KeyN",
        Key::KeyO => "KeyO",
        Key::KeyP => "KeyP",
        Key::KeyQ => "KeyQ",
        Key::KeyR => "KeyR",
        Key::KeyS => "KeyS",
        Key::KeyT => "KeyT",
        Key::KeyU => "KeyU",
        Key::KeyV => "KeyV",
        Key::KeyW => "KeyW",
        Key::KeyX => "KeyX",
        Key::KeyY => "KeyY",
        Key::KeyZ => "KeyZ",
        Key::Num0 => "Digit0",
        Key::Num1 => "Digit1",
        Key::Num2 => "Digit2",
        Key::Num3 => "Digit3",
        Key::Num4 => "Digit4",
        Key::Num5 => "Digit5",
        Key::Num6 => "Digit6",
        Key::Num7 => "Digit7",
        Key::Num8 => "Digit8",
        Key::Num9 => "Digit9",
        Key::Space => "Space",
        Key::Escape => "Escape",
        Key::Backspace => "Backspace",
        Key::Tab => "Tab",
        Key::Return => "Enter",
        Key::ControlLeft => "ControlLeft",
        Key::ControlRight => "ControlRight",
        Key::ShiftLeft => "ShiftLeft",
        Key::ShiftRight => "ShiftRight",
        Key::Alt => "AltLeft",
        Key::AltGr => "AltRight",
        Key::MetaLeft | Key::MetaRight => "MetaLeft",
        Key::CapsLock => "CapsLock",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::UpArrow => "ArrowUp",
        Key::DownArrow => "ArrowDown",
        Key::LeftArrow => "ArrowLeft",
        Key::RightArrow => "ArrowRight",
        Key::Minus => "Minus",
        Key::Equal => "Equal",
        Key::BackQuote => "Backquote",
        Key::LeftBracket => "BracketLeft",
        Key::RightBracket => "BracketRight",
        Key::BackSlash => "Backslash",
        Key::IntlBackslash => "IntlBackslash",
        Key::SemiColon => "Semicolon",
        Key::Quote => "Quote",
        Key::Comma => "Comma",
        Key::Dot => "Period",
        Key::Slash => "Slash",
        Key::Insert => "Insert",
        Key::Delete => "Delete",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::PrintScreen => "PrintScreen",
        Key::ScrollLock => "ScrollLock",
        Key::Pause => "Pause",
        Key::NumLock => "NumLock",
        Key::Kp0 => "Numpad0",
        Key::Kp1 => "Numpad1",
        Key::Kp2 => "Numpad2",
        Key::Kp3 => "Numpad3",
        Key::Kp4 => "Numpad4",
        Key::Kp5 => "Numpad5",
        Key::Kp6 => "Numpad6",
        Key::Kp7 => "Numpad7",
        Key::Kp8 => "Numpad8",
        Key::Kp9 => "Numpad9",
        Key::KpReturn => "NumpadEnter",
        Key::KpMinus => "NumpadSubtract",
        Key::KpPlus => "NumpadAdd",
        Key::KpMultiply => "NumpadMultiply",
        Key::KpDivide => "NumpadDivide",
        Key::KpDelete => "NumpadDecimal",
        Key::Function => "Fn",
        Key::Unknown(raw) => return Cow::Owned(format!("Unknown{raw}")),
    };
    Cow::Borrowed(code)
}

pub fn key_label(key: Key) -> Cow<'static, str> {
    let label = match key {
        Key::Space => "Space",
        Key::Backspace => "⌫",
        Key::Return => "↵",
        Key::Escape => "Esc",
        Key::Tab => "Tab",
        Key::ShiftLeft | Key::ShiftRight => "Shift",
        Key::ControlLeft | Key::ControlRight => "Ctrl",
        Key::Alt | Key::AltGr => {
            if cfg!(target_os = "macos") {
                "Option"
            } else {
                "Alt"
            }
        }
        Key::MetaLeft | Key::MetaRight => {
            if cfg!(target_os = "macos") {
                "Cmd"
            } else if cfg!(target_os = "windows") {
                "Win"
            } else {
                "Super"
            }
        }
        Key::CapsLock => "Caps",
        Key::UpArrow => "▲",
        Key::DownArrow => "▼",
        Key::LeftArrow => "◀",
        Key::RightArrow => "▶",
        _ => return key_code(key),
    };
    Cow::Borrowed(label)
}

pub fn button_code(button: Button) -> Cow<'static, str> {
    match button {
        Button::Left => Cow::Borrowed("Mouseleft"),
        Button::Right => Cow::Borrowed("Mouseright"),
        Button::Middle => Cow::Borrowed("Mousemiddle"),
        Button::Unknown(raw) => Cow::Owned(format!("Mouse{raw}")),
    }
}

pub fn button_label(button: Button) -> Cow<'static, str> {
    match button {
        Button::Left => Cow::Borrowed("LMB"),
        Button::Right => Cow::Borrowed("RMB"),
        Button::Middle => Cow::Borrowed("MMB"),
        Button::Unknown(raw) => Cow::Owned(format!("M{raw}")),
    }
}
