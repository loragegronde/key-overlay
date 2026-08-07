import type {
  KeyConfig,
  KeyStyle,
  LayoutTemplateId,
  ProfileConfig,
  VisualTheme,
} from "../types";
import { DEFAULT_GRID_SIZE, DEFAULT_KEY_STYLE, PROFILE_SCHEMA_VERSION } from "../types";

let keyCounter = 0;

export function createKey(
  partial: Partial<KeyConfig> & { code: string; label: string },
): KeyConfig {
  keyCounter += 1;
  const { style: partialStyle, ...rest } = partial;
  return {
    id: rest.id ?? `key-${keyCounter}-${Date.now()}`,
    code: rest.code,
    label: rest.label,
    x: rest.x ?? 100,
    y: rest.y ?? 100,
    width: rest.width ?? 56,
    height: rest.height ?? 56,
    shape: rest.shape ?? "rectangle",
    rotation: rest.rotation ?? 0,
    scale: rest.scale ?? 1,
    style: { ...DEFAULT_KEY_STYLE, ...partialStyle },
  };
}

const ROW = (y: number, keys: [string, string][], startX = 40) =>
  keys.map(([code, label], i) =>
    createKey({
      code,
      label,
      x: startX + i * 58,
      y,
      width: code === "Space" ? 280 : code.startsWith("F") && code.length <= 3 ? 44 : 52,
      height: 52,
    }),
  );

/**
 * The alphanumeric block shared by the full-size and tenkeyless presets.
 *
 * A function rather than a constant so each template gets its own key objects
 * instead of aliasing the same ids.
 */
const alphanumericBlock = (): KeyConfig[] =>
  [
    ...ROW(40, [
      ["Escape", "Esc"],
      ["F1", "F1"],
      ["F2", "F2"],
      ["F3", "F3"],
      ["F4", "F4"],
      ["F5", "F5"],
      ["F6", "F6"],
      ["F7", "F7"],
      ["F8", "F8"],
      ["F9", "F9"],
      ["F10", "F10"],
      ["F11", "F11"],
      ["F12", "F12"],
    ]),
    ...ROW(100, [
      ["Backquote", "`"],
      ["Digit1", "1"],
      ["Digit2", "2"],
      ["Digit3", "3"],
      ["Digit4", "4"],
      ["Digit5", "5"],
      ["Digit6", "6"],
      ["Digit7", "7"],
      ["Digit8", "8"],
      ["Digit9", "9"],
      ["Digit0", "0"],
      ["Minus", "-"],
      ["Equal", "="],
      ["Backspace", "⌫"],
    ]),
    ...ROW(160, [
      ["Tab", "Tab"],
      ["KeyQ", "Q"],
      ["KeyW", "W"],
      ["KeyE", "E"],
      ["KeyR", "R"],
      ["KeyT", "T"],
      ["KeyY", "Y"],
      ["KeyU", "U"],
      ["KeyI", "I"],
      ["KeyO", "O"],
      ["KeyP", "P"],
      ["BracketLeft", "["],
      ["BracketRight", "]"],
      ["Backslash", "\\"],
    ]),
    ...ROW(220, [
      ["CapsLock", "Caps"],
      ["KeyA", "A"],
      ["KeyS", "S"],
      ["KeyD", "D"],
      ["KeyF", "F"],
      ["KeyG", "G"],
      ["KeyH", "H"],
      ["KeyJ", "J"],
      ["KeyK", "K"],
      ["KeyL", "L"],
      ["Semicolon", ";"],
      ["Quote", "'"],
      ["Enter", "↵"],
    ]),
    ...ROW(280, [
      ["ShiftLeft", "Shift"],
      ["KeyZ", "Z"],
      ["KeyX", "X"],
      ["KeyC", "C"],
      ["KeyV", "V"],
      ["KeyB", "B"],
      ["KeyN", "N"],
      ["KeyM", "M"],
      ["Comma", ","],
      ["Period", "."],
      ["Slash", "/"],
      ["ShiftRight", "Shift"],
    ]),
    ...ROW(340, [
      ["ControlLeft", "Ctrl"],
      ["MetaLeft", "Win"],
      ["AltLeft", "Alt"],
      ["Space", "Space"],
      ["AltRight", "Alt"],
      ["MetaRight", "Win"],
      ["ControlRight", "Ctrl"],
    ]),
  ].flat();

/** The cluster that makes "full-size" actually differ from "tenkeyless". */
const numpadBlock = (): KeyConfig[] => {
  const x = 880;
  return [
    ...ROW(100, [
      ["NumLock", "Num"],
      ["NumpadDivide", "/"],
      ["NumpadMultiply", "*"],
      ["NumpadSubtract", "-"],
    ], x),
    ...ROW(160, [
      ["Numpad7", "7"],
      ["Numpad8", "8"],
      ["Numpad9", "9"],
      ["NumpadAdd", "+"],
    ], x),
    ...ROW(220, [
      ["Numpad4", "4"],
      ["Numpad5", "5"],
      ["Numpad6", "6"],
    ], x),
    ...ROW(280, [
      ["Numpad1", "1"],
      ["Numpad2", "2"],
      ["Numpad3", "3"],
      ["NumpadEnter", "↵"],
    ], x),
    ...ROW(340, [
      ["Numpad0", "0"],
      ["NumpadDecimal", "."],
    ], x),
  ].flat();
};

export const LAYOUT_TEMPLATES: Record<
  LayoutTemplateId,
  { name: string; description: string; keys: KeyConfig[] }
> = {
  "full-100": {
    name: "Full 100%",
    description: "Full-size board including the numpad cluster",
    keys: [...alphanumericBlock(), ...numpadBlock()],
  },
  tkl: {
    name: "TKL",
    description: "Tenkeyless — the full board minus the numpad",
    keys: alphanumericBlock(),
  },
  "60-percent": {
    name: "60%",
    description: "Compact 60% layout",
    keys: [
      ...ROW(60, [
        ["Escape", "Esc"],
        ["Digit1", "1"],
        ["Digit2", "2"],
        ["Digit3", "3"],
        ["Digit4", "4"],
        ["Digit5", "5"],
        ["Digit6", "6"],
        ["Digit7", "7"],
        ["Digit8", "8"],
        ["Digit9", "9"],
        ["Digit0", "0"],
        ["Minus", "-"],
        ["Equal", "="],
        ["Backspace", "⌫"],
      ]),
      ...ROW(120, [
        ["Tab", "Tab"],
        ["KeyQ", "Q"],
        ["KeyW", "W"],
        ["KeyE", "E"],
        ["KeyR", "R"],
        ["KeyT", "T"],
        ["KeyY", "Y"],
        ["KeyU", "U"],
        ["KeyI", "I"],
        ["KeyO", "O"],
        ["KeyP", "P"],
        ["BracketLeft", "["],
        ["BracketRight", "]"],
        ["Backslash", "\\"],
      ]),
      ...ROW(180, [
        ["CapsLock", "Caps"],
        ["KeyA", "A"],
        ["KeyS", "S"],
        ["KeyD", "D"],
        ["KeyF", "F"],
        ["KeyG", "G"],
        ["KeyH", "H"],
        ["KeyJ", "J"],
        ["KeyK", "K"],
        ["KeyL", "L"],
        ["Semicolon", ";"],
        ["Quote", "'"],
        ["Enter", "↵"],
      ]),
      ...ROW(240, [
        ["ShiftLeft", "Shift"],
        ["KeyZ", "Z"],
        ["KeyX", "X"],
        ["KeyC", "C"],
        ["KeyV", "V"],
        ["KeyB", "B"],
        ["KeyN", "N"],
        ["KeyM", "M"],
        ["Comma", ","],
        ["Period", "."],
        ["Slash", "/"],
        ["ShiftRight", "Shift"],
      ]),
      ...ROW(300, [
        ["ControlLeft", "Ctrl"],
        ["MetaLeft", "Win"],
        ["AltLeft", "Alt"],
        ["Space", "Space"],
        ["AltRight", "Alt"],
        ["ControlRight", "Ctrl"],
      ]),
    ].flat(),
  },
  "wasd-gaming": {
    name: "WASD Gaming",
    description: "FPS movement cluster + common binds",
    keys: [
      createKey({ label: "W", code: "KeyW", x: 120, y: 80, width: 64, height: 64 }),
      createKey({ label: "A", code: "KeyA", x: 50, y: 150, width: 64, height: 64 }),
      createKey({ label: "S", code: "KeyS", x: 120, y: 150, width: 64, height: 64 }),
      createKey({ label: "D", code: "KeyD", x: 190, y: 150, width: 64, height: 64 }),
      createKey({ label: "Space", code: "Space", x: 280, y: 150, width: 120, height: 48 }),
      createKey({ label: "Shift", code: "ShiftLeft", x: 280, y: 80, width: 80, height: 48 }),
      createKey({ label: "Ctrl", code: "ControlLeft", x: 280, y: 210, width: 80, height: 48 }),
      createKey({ label: "E", code: "KeyE", x: 420, y: 80, width: 56, height: 56 }),
      createKey({ label: "R", code: "KeyR", x: 420, y: 150, width: 56, height: 56 }),
      createKey({ label: "Q", code: "KeyQ", x: 420, y: 220, width: 56, height: 56 }),
      createKey({ label: "LMB", code: "Mouseleft", x: 520, y: 100, width: 72, height: 72, shape: "circle" }),
      createKey({ label: "RMB", code: "Mouseright", x: 520, y: 190, width: 72, height: 72, shape: "circle" }),
    ],
  },
  "fighting-arcade": {
    name: "Arcade Stick",
    description: "6-button fighting game layout",
    keys: [
      createKey({ label: "↑", code: "ArrowUp", x: 80, y: 60, width: 56, height: 56 }),
      createKey({ label: "←", code: "ArrowLeft", x: 20, y: 120, width: 56, height: 56 }),
      createKey({ label: "↓", code: "ArrowDown", x: 80, y: 120, width: 56, height: 56 }),
      createKey({ label: "→", code: "ArrowRight", x: 140, y: 120, width: 56, height: 56 }),
      createKey({ label: "LP", code: "KeyU", x: 280, y: 80, width: 64, height: 64, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(239,68,68,0.7)" } }),
      createKey({ label: "MP", code: "KeyI", x: 360, y: 80, width: 64, height: 64, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(249,115,22,0.7)" } }),
      createKey({ label: "HP", code: "KeyO", x: 440, y: 80, width: 64, height: 64, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(234,179,8,0.7)" } }),
      createKey({ label: "LK", code: "KeyJ", x: 280, y: 160, width: 64, height: 64, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(59,130,246,0.7)" } }),
      createKey({ label: "MK", code: "KeyK", x: 360, y: 160, width: 64, height: 64, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(139,92,246,0.7)" } }),
      createKey({ label: "HK", code: "KeyL", x: 440, y: 160, width: 64, height: 64, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(236,72,153,0.7)" } }),
    ],
  },
  "streamer-hud": {
    name: "Streamer HUD",
    description: "Minimal overlay for stream displays",
    keys: [
      createKey({ label: "KPS", code: "KeyW", x: 40, y: 40, width: 100, height: 48, style: { ...DEFAULT_KEY_STYLE, showPressCount: true, backgroundColor: "rgba(0,0,0,0.5)" } }),
      createKey({ label: "Mic", code: "KeyM", x: 160, y: 40, width: 64, height: 48 }),
      createKey({ label: "Cam", code: "KeyC", x: 240, y: 40, width: 64, height: 48 }),
      createKey({ label: "Scene", code: "F1", x: 320, y: 40, width: 72, height: 48 }),
    ],
  },
  controller: {
    name: "Controller",
    description: "Xbox-style pad with face buttons, D-pad and both joysticks",
    keys: [
      createKey({ label: "LB", code: "PadLB", x: 40, y: 40, width: 72, height: 40 }),
      createKey({ label: "LT", code: "PadLT", x: 40, y: 90, width: 72, height: 40 }),
      createKey({ label: "RB", code: "PadRB", x: 400, y: 40, width: 72, height: 40 }),
      createKey({ label: "RT", code: "PadRT", x: 400, y: 90, width: 72, height: 40 }),
      createKey({
        label: "LS",
        code: "PadLS",
        x: 60,
        y: 160,
        width: 96,
        height: 96,
        shape: "stick",
        style: { ...DEFAULT_KEY_STYLE, borderRadius: 48 },
      }),
      createKey({
        label: "RS",
        code: "PadRS",
        x: 280,
        y: 260,
        width: 96,
        height: 96,
        shape: "stick",
        style: { ...DEFAULT_KEY_STYLE, borderRadius: 48 },
      }),
      createKey({ label: "Y", code: "PadY", x: 440, y: 160, width: 56, height: 56, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(234,179,8,0.75)" } }),
      createKey({ label: "X", code: "PadX", x: 390, y: 210, width: 56, height: 56, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(59,130,246,0.75)" } }),
      createKey({ label: "B", code: "PadB", x: 490, y: 210, width: 56, height: 56, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(239,68,68,0.75)" } }),
      createKey({ label: "A", code: "PadA", x: 440, y: 260, width: 56, height: 56, shape: "circle", style: { ...DEFAULT_KEY_STYLE, backgroundColor: "rgba(34,197,94,0.75)" } }),
      createKey({ label: "↑", code: "PadUp", x: 200, y: 160, width: 44, height: 44 }),
      createKey({ label: "←", code: "PadLeft", x: 156, y: 204, width: 44, height: 44 }),
      createKey({ label: "↓", code: "PadDown", x: 200, y: 248, width: 44, height: 44 }),
      createKey({ label: "→", code: "PadRight", x: 244, y: 204, width: 44, height: 44 }),
      createKey({ label: "Back", code: "PadBack", x: 220, y: 120, width: 56, height: 32 }),
      createKey({ label: "Start", code: "PadStart", x: 286, y: 120, width: 56, height: 32 }),
    ],
  },
  custom: {
    name: "Custom",
    description: "Start from scratch",
    keys: [],
  },
};

export const VISUAL_THEMES: Record<
  VisualTheme,
  { name: string; globalStyle: Partial<KeyStyle> }
> = {
  cyberpunk: {
    name: "Cyberpunk",
    globalStyle: {
      backgroundColor: "rgba(10, 10, 30, 0.8)",
      borderColor: "rgba(255, 0, 128, 0.8)",
      activeGlowColor: "rgba(0, 255, 255, 0.9)",
      textColor: "#ff0080",
      borderRadius: 8,
      opacity: 1,
      pressEffect: "glow",
    },
  },
  glassmorphism: {
    name: "Glassmorphism",
    globalStyle: {
      backgroundColor: "rgba(255, 255, 255, 0.1)",
      borderColor: "rgba(255, 255, 255, 0.3)",
      activeGlowColor: "rgba(255, 255, 255, 0.5)",
      textColor: "#ffffff",
      borderRadius: 12,
      opacity: 0.7,
      pressEffect: "border-ripple",
    },
  },
  "retro-arcade": {
    name: "Retro Arcade",
    globalStyle: {
      backgroundColor: "rgba(20, 0, 40, 0.85)",
      borderColor: "rgba(255, 215, 0, 0.9)",
      activeGlowColor: "rgba(255, 100, 0, 0.9)",
      textColor: "#ffd700",
      borderRadius: 4,
      opacity: 1,
      fontFamily: "Orbitron",
      pressEffect: "glow-pulse",
    },
  },
  "stealth-minimal": {
    name: "Stealth Minimal",
    globalStyle: {
      backgroundColor: "rgba(12, 12, 14, 0.55)",
      borderColor: "rgba(255, 255, 255, 0.14)",
      activeGlowColor: "rgba(235, 235, 240, 0.75)",
      textColor: "#e6e6ea",
      borderRadius: 6,
      opacity: 0.9,
      pressEffect: "key-drop",
    },
  },
  "rgb-wave": {
    name: "RGB Wave",
    globalStyle: {
      backgroundColor: "rgba(0, 0, 0, 0.6)",
      borderColor: "rgba(255, 255, 255, 0.2)",
      activeGlowColor: "rgba(168, 85, 247, 0.9)",
      textColor: "#ffffff",
      borderRadius: 10,
      opacity: 1,
      pressEffect: "glow-pulse",
    },
  },
  custom: {
    name: "Custom",
    globalStyle: {},
  },
};

export function createProfileFromTemplate(
  templateId: LayoutTemplateId,
  name?: string,
): ProfileConfig {
  const template = LAYOUT_TEMPLATES[templateId];
  const now = new Date().toISOString();
  return {
    version: PROFILE_SCHEMA_VERSION,
    id: `profile-${Date.now()}`,
    name: name ?? template.name,
    templateId,
    keys: template.keys.map((k) => ({
      ...k,
      id: `${k.id}-clone-${Date.now()}-${Math.random()}`,
      style: { ...k.style },
    })),
    globalTheme: "cyberpunk",
    showKpsMeter: true,
    windowOpacity: 1,
    snapToGrid: false,
    gridSize: DEFAULT_GRID_SIZE,
    targetAppEnabled: false,
    targetAppMatch: "",
    createdAt: now,
    updatedAt: now,
  };
}
