//! Global keyboard/mouse (rdev) + gamepad (gilrs) capture.

mod egui_bind;
mod mapping;

pub use egui_bind::{apply_egui_presses, capture_from_egui};

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use gilrs::{Axis, Button, Gilrs};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

pub use mapping::{button_code, button_label, key_code, key_label};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Down,
    Up,
}

#[derive(Debug, Clone)]
pub enum InputMsg {
    Key {
        code: String,
        label: String,
        action: InputAction,
    },
    StickAxes {
        code: String,
        x: f32,
        y: f32,
    },
}

static FORWARDING: AtomicBool = AtomicBool::new(true);
static STARTED: AtomicBool = AtomicBool::new(false);
static TX: Lazy<Mutex<Option<Sender<InputMsg>>>> = Lazy::new(|| Mutex::new(None));

pub fn start_listener() -> Receiver<InputMsg> {
    let (tx, rx) = mpsc::channel();
    *TX.lock() = Some(tx.clone());
    FORWARDING.store(true, Ordering::Relaxed);

    if !STARTED.swap(true, Ordering::SeqCst) {
        let tx_keys = tx.clone();
        thread::Builder::new()
            .name("rdev-input".into())
            .spawn(move || {
                if let Err(err) = rdev::listen(move |event| {
                    if !FORWARDING.load(Ordering::Relaxed) {
                        return;
                    }
                    if let Some(msg) = translate_rdev(event) {
                        let _ = tx_keys.send(msg);
                    }
                }) {
                    eprintln!("rdev keyboard/mouse listener failed: {err:?}");
                    eprintln!("rebind will still work via the egui fallback while the editor is focused");
                }
            })
            .ok();

        let tx_pad = tx;
        thread::Builder::new()
            .name("gilrs-input".into())
            .spawn(move || gamepad_loop(tx_pad))
            .ok();
    }

    rx
}


fn translate_rdev(event: rdev::Event) -> Option<InputMsg> {
    use rdev::EventType;
    let (code, label, action) = match event.event_type {
        EventType::KeyPress(key) => (key_code(key), key_label(key), InputAction::Down),
        EventType::KeyRelease(key) => (key_code(key), key_label(key), InputAction::Up),
        EventType::ButtonPress(button) => {
            (button_code(button), button_label(button), InputAction::Down)
        }
        EventType::ButtonRelease(button) => {
            (button_code(button), button_label(button), InputAction::Up)
        }
        _ => return None,
    };
    Some(InputMsg::Key {
        code: code.into_owned(),
        label: label.into_owned(),
        action,
    })
}

fn gamepad_loop(tx: Sender<InputMsg>) {
    let Ok(mut gilrs) = Gilrs::new() else {
        eprintln!("gilrs: no gamepad support on this platform");
        return;
    };

    let mut prev_buttons: HashMap<(usize, String), bool> = HashMap::new();
    let mut prev_dirs: HashMap<(usize, String), bool> = HashMap::new();
    let mut prev_stick: HashMap<(usize, String), bool> = HashMap::new();
    let mut prev_axes: HashMap<String, (f32, f32)> = HashMap::new();

    loop {
        while let Some(_ev) = gilrs.next_event() {}

        for (idx, (_id, gamepad)) in gilrs.gamepads().enumerate() {
            for (button, code, label) in PAD_BUTTONS {
                let pressed = gamepad.is_pressed(*button);
                let key = (idx, (*code).to_string());
                let was = *prev_buttons.get(&key).unwrap_or(&false);
                if pressed != was {
                    let action = if pressed {
                        InputAction::Down
                    } else {
                        InputAction::Up
                    };
                    let _ = tx.send(InputMsg::Key {
                        code: (*code).into(),
                        label: (*label).into(),
                        action,
                    });
                }
                prev_buttons.insert(key, pressed);
            }

            let lx = gamepad.value(Axis::LeftStickX);
            let ly = -gamepad.value(Axis::LeftStickY);
            let rx = gamepad.value(Axis::RightStickX);
            let ry = -gamepad.value(Axis::RightStickY);
            for (code, x, y) in [("PadLS", lx, ly), ("PadRS", rx, ry)] {
                let prev = prev_axes.get(code).copied().unwrap_or((0.0, 0.0));
                if (prev.0 - x).abs() > 0.02 || (prev.1 - y).abs() > 0.02 {
                    let _ = tx.send(InputMsg::StickAxes {
                        code: code.into(),
                        x,
                        y,
                    });
                    prev_axes.insert(code.into(), (x, y));
                }
            }

            for (code, x, y, label) in [
                ("PadLS", lx, ly, "LS"),
                ("PadRS", rx, ry, "RS"),
            ] {
                let active = (x * x + y * y).sqrt() >= 0.18;
                let key = (idx, code.to_string());
                let was = *prev_stick.get(&key).unwrap_or(&false);
                if active != was {
                    let _ = tx.send(InputMsg::Key {
                        code: code.into(),
                        label: label.into(),
                        action: if active {
                            InputAction::Down
                        } else {
                            InputAction::Up
                        },
                    });
                }
                prev_stick.insert(key, active);
            }

            for (axis, neg_code, neg_label, pos_code, pos_label, invert) in [
                (Axis::LeftStickX, "PadLSLeft", "LS←", "PadLSRight", "LS→", false),
                (Axis::LeftStickY, "PadLSUp", "LS↑", "PadLSDown", "LS↓", true),
                (Axis::RightStickX, "PadRSLeft", "RS←", "PadRSRight", "RS→", false),
                (Axis::RightStickY, "PadRSUp", "RS↑", "PadRSDown", "RS↓", true),
            ] {
                let mut v = gamepad.value(axis);
                if invert {
                    v = -v;
                }
                for (active, code, label) in [
                    (v < -0.55, neg_code, neg_label),
                    (v > 0.55, pos_code, pos_label),
                ] {
                    let key = (idx, code.to_string());
                    let was = *prev_dirs.get(&key).unwrap_or(&false);
                    if active != was {
                        let _ = tx.send(InputMsg::Key {
                            code: code.into(),
                            label: label.into(),
                            action: if active {
                                InputAction::Down
                            } else {
                                InputAction::Up
                            },
                        });
                    }
                    prev_dirs.insert(key, active);
                }
            }

        }

        thread::sleep(std::time::Duration::from_millis(8));
    }
}

const PAD_BUTTONS: &[(Button, &str, &str)] = &[
    (Button::South, "PadA", "A"),
    (Button::East, "PadB", "B"),
    (Button::West, "PadX", "X"),
    (Button::North, "PadY", "Y"),
    (Button::LeftTrigger, "PadLB", "LB"),
    (Button::RightTrigger, "PadRB", "RB"),
    (Button::LeftTrigger2, "PadLT", "LT"),
    (Button::RightTrigger2, "PadRT", "RT"),
    (Button::Select, "PadBack", "Back"),
    (Button::Start, "PadStart", "Start"),
    (Button::LeftThumb, "PadL3", "L3"),
    (Button::RightThumb, "PadR3", "R3"),
    (Button::DPadUp, "PadUp", "↑"),
    (Button::DPadDown, "PadDown", "↓"),
    (Button::DPadLeft, "PadLeft", "←"),
    (Button::DPadRight, "PadRight", "→"),
];
