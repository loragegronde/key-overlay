// Without this the release build is a console subsystem binary, which opens a
// stray terminal window behind the transparent overlay on Windows. Debug builds
// keep the console so panics and `eprintln!` stay visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    key_overlay_lib::run();
}
