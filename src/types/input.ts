// Wire contract shared with src-tauri/src/input_listener.rs.
//
// The Rust side asserts these exact field names and string values in its unit
// tests. Do not rename anything here without changing both sides together.

export type InputDevice = "keyboard" | "mouse" | "gamepad";

export type InputAction = "down" | "up";

export interface InputEventPayload {
  device: InputDevice;
  action: InputAction;
  /** Stable identity, e.g. "KeyW", "Digit4", "Mouseleft". */
  code: string;
  label: string;
  /** Milliseconds since the Unix epoch. */
  timestamp: number;
}
