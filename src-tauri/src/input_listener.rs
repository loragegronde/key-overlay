//! System-wide keyboard and mouse capture.
//!
//! Everything the OS gives us is normalised into a single [`InputEventPayload`]
//! and pushed to the frontend on one channel, [`INPUT_EVENT`].
//!
//! The platform hook sits behind [`InputBackend`]. `rdev` is the only backend
//! today; on Linux it binds to X11 and receives nothing under a native Wayland
//! session. A libei/evdev backend can be added by implementing the trait and
//! returning it from [`select_backend`] — the payload the frontend consumes is
//! deliberately backend-independent so that swap does not reach the UI.

use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::UNIX_EPOCH;

use rdev::{Button, Event, EventType, Key};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// The single event channel the frontend subscribes to.
pub const INPUT_EVENT: &str = "input-event";

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputDevice {
    Keyboard,
    Mouse,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputAction {
    Down,
    Up,
}

/// One normalised input event.
///
/// `code` is the stable identity the frontend matches bindings against; `label`
/// is display text only. Both borrow `'static` strings for every key we know
/// about, so the common path allocates nothing.
#[derive(Clone, Serialize)]
pub struct InputEventPayload {
    pub device: InputDevice,
    pub action: InputAction,
    pub code: Cow<'static, str>,
    pub label: Cow<'static, str>,
    /// Milliseconds since the Unix epoch.
    pub timestamp: u64,
}

type EventSink = Box<dyn FnMut(InputEventPayload) + Send + 'static>;

trait InputBackend: Send + 'static {
    fn name(&self) -> &'static str;

    /// Captures input until the backend fails. Note that the `rdev`
    /// implementation never returns on the success path.
    fn run(self: Box<Self>, sink: EventSink) -> Result<(), String>;
}

struct RdevBackend;

impl InputBackend for RdevBackend {
    fn name(&self) -> &'static str {
        "rdev"
    }

    fn run(self: Box<Self>, mut sink: EventSink) -> Result<(), String> {
        rdev::listen(move |event| {
            if let Some(payload) = translate(event) {
                sink(payload);
            }
        })
        .map_err(|err| format!("{err:?}"))
    }
}

fn select_backend() -> Box<dyn InputBackend> {
    Box::new(RdevBackend)
}

/// Whether the backend thread has been spawned. The hook is process-wide and
/// cannot be uninstalled, so this only ever goes false again if the backend
/// itself failed.
static BACKEND_STARTED: AtomicBool = AtomicBool::new(false);

/// Whether captured events are forwarded to the UI. This is what start/stop
/// actually toggle.
static FORWARDING: AtomicBool = AtomicBool::new(false);

/// Starts forwarding input events, spawning the platform backend on first call.
#[tauri::command]
pub fn start_input_listener(app: AppHandle) -> Result<(), String> {
    FORWARDING.store(true, Ordering::Relaxed);

    if BACKEND_STARTED.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let backend = select_backend();
    let backend_name = backend.name();

    // Broadcast to every webview. The editor needs these events for key
    // rebinding ("Press any key…"); the overlay needs them for the HUD.
    // `emit_to(overlay)` alone left the editor deaf, so Add Key → capture
    // never completed.
    let sink: EventSink = Box::new(move |payload| {
        if !FORWARDING.load(Ordering::Relaxed) {
            return;
        }
        let _ = app.emit(INPUT_EVENT, payload);
    });

    // The handle is intentionally dropped: there is nothing joinable here.
    let spawned = thread::Builder::new()
        .name("input-listener".into())
        .spawn(move || {
            if let Err(err) = backend.run(sink) {
                FORWARDING.store(false, Ordering::Relaxed);
                BACKEND_STARTED.store(false, Ordering::SeqCst);
                eprintln!("input listener backend '{backend_name}' stopped: {err}");
            }
        });

    if let Err(err) = spawned {
        FORWARDING.store(false, Ordering::Relaxed);
        BACKEND_STARTED.store(false, Ordering::SeqCst);
        return Err(format!("could not spawn input listener thread: {err}"));
    }

    Ok(())
}

/// Stops forwarding input events to the UI.
///
/// This deliberately does not tear the OS hook down. `rdev::listen` blocks for
/// the lifetime of the process and offers no cancellation, so the thread cannot
/// be joined — the previous implementation kept a `JoinHandle` and called
/// `join()` here, which could only ever hang. The hook is released when the
/// process exits. Replacing rdev with a cancellable backend is what would make
/// a real stop possible.
#[tauri::command]
pub fn stop_input_listener() -> Result<(), String> {
    FORWARDING.store(false, Ordering::Relaxed);
    Ok(())
}

/// Converts a backend event into the wire payload. Returns `None` for events we
/// do not visualise (mouse movement, wheel).
///
/// This runs on the hook thread under `panic = "abort"`, so it must not panic:
/// no unwraps, and every enum arm is total.
fn translate(event: Event) -> Option<InputEventPayload> {
    let (device, action, code, label) = match event.event_type {
        EventType::KeyPress(key) => (
            InputDevice::Keyboard,
            InputAction::Down,
            key_code(key),
            key_label(key),
        ),
        EventType::KeyRelease(key) => (
            InputDevice::Keyboard,
            InputAction::Up,
            key_code(key),
            key_label(key),
        ),
        EventType::ButtonPress(button) => (
            InputDevice::Mouse,
            InputAction::Down,
            button_code(button),
            button_label(button),
        ),
        EventType::ButtonRelease(button) => (
            InputDevice::Mouse,
            InputAction::Up,
            button_code(button),
            button_label(button),
        ),
        _ => return None,
    };

    let timestamp = event
        .time
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default();

    Some(InputEventPayload {
        device,
        action,
        code,
        label,
        timestamp,
    })
}

/// Maps a physical key to a W3C `KeyboardEvent.code` style identifier.
///
/// `MetaLeft`/`MetaRight` intentionally collapse to a single code so either
/// Windows/Command key lights the same overlay element.
fn key_code(key: Key) -> Cow<'static, str> {
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

fn key_label(key: Key) -> Cow<'static, str> {
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
        Key::UpArrow => "↑",
        Key::DownArrow => "↓",
        Key::LeftArrow => "←",
        Key::RightArrow => "→",
        _ => return key_code(key),
    };

    Cow::Borrowed(label)
}

/// Mouse codes are lowercase-suffixed on purpose: the frontend matches them
/// case-insensitively against `mouse${binding.button}`.
fn button_code(button: Button) -> Cow<'static, str> {
    match button {
        Button::Left => Cow::Borrowed("Mouseleft"),
        Button::Right => Cow::Borrowed("Mouseright"),
        Button::Middle => Cow::Borrowed("Mousemiddle"),
        Button::Unknown(raw) => Cow::Owned(format!("Mouse{raw}")),
    }
}

fn button_label(button: Button) -> Cow<'static, str> {
    match button {
        Button::Left => Cow::Borrowed("LMB"),
        Button::Right => Cow::Borrowed("RMB"),
        Button::Middle => Cow::Borrowed("MMB"),
        Button::Unknown(raw) => Cow::Owned(format!("M{raw}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The other half of this contract is `InputEventPayload` in
    /// `src/types/index.ts`. If these names change, that file changes too.
    #[test]
    fn payload_serialises_to_the_shape_the_frontend_expects() {
        let payload = InputEventPayload {
            device: InputDevice::Keyboard,
            action: InputAction::Down,
            code: key_code(Key::KeyW),
            label: key_label(Key::KeyW),
            timestamp: 1_700_000_000_000,
        };

        let json = serde_json::to_value(&payload).expect("payload must serialise");

        assert_eq!(json["device"], "keyboard");
        assert_eq!(json["action"], "down");
        assert_eq!(json["code"], "KeyW");
        assert_eq!(json["label"], "KeyW");
        assert_eq!(json["timestamp"], 1_700_000_000_000u64);
    }

    #[test]
    fn device_and_action_serialise_as_lowercase_literals() {
        assert_eq!(serde_json::to_value(InputDevice::Mouse).unwrap(), "mouse");
        assert_eq!(serde_json::to_value(InputAction::Up).unwrap(), "up");
    }

    /// `useOverlayStore` matches mouse codes as
    /// ``code.toLowerCase() === `mouse${binding.button}` ``.
    #[test]
    fn mouse_codes_match_the_frontend_binding_format() {
        for (button, expected) in [
            (Button::Left, "mouseleft"),
            (Button::Right, "mouseright"),
            (Button::Middle, "mousemiddle"),
        ] {
            assert_eq!(button_code(button).to_lowercase(), expected);
        }
    }

    /// Mouse movement and wheel events must not reach the UI.
    #[test]
    fn unvisualised_events_are_dropped() {
        let event = Event {
            time: std::time::SystemTime::now(),
            name: None,
            event_type: EventType::MouseMove { x: 1.0, y: 2.0 },
        };

        assert!(translate(event).is_none());
    }
}
