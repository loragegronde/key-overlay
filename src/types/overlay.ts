/**
 * Press animations.
 *
 * These are driven entirely by framer-motion. An earlier version built Tailwind
 * class names at runtime (`animate-${effect}`), which the Tailwind scanner
 * cannot see and therefore purged, so half the effects silently did nothing.
 */
export type PressEffect = "glow" | "glow-pulse" | "key-drop" | "border-ripple" | "none";

export type VisualTheme =
  | "cyberpunk"
  | "glassmorphism"
  | "retro-arcade"
  | "stealth-minimal"
  | "rgb-wave"
  | "custom";

export type LayoutTemplateId =
  | "full-100"
  | "tkl"
  | "60-percent"
  | "wasd-gaming"
  | "fighting-arcade"
  | "streamer-hud"
  | "controller"
  | "custom";

export type KeyShape = "rectangle" | "circle" | "stick";

/**
 * EDIT lets the user arrange keys; OVERLAY is the locked, click-through HUD.
 *
 * This replaces the old pair of `editMode` / `clickThrough` booleans, which had
 * to be kept in opposition by hand at every call site.
 */
export type ToolMode = "EDIT" | "OVERLAY";

/** Which panel of the customization drawer is open, or null when it is closed. */
export type DrawerTab = "visuals" | "themes" | "animations" | "layouts" | "settings";

export type AlignEdge = "left" | "center" | "right" | "top" | "middle" | "bottom";

/** Whether a drawer edit hits the current selection or every key. */
export type StyleScope = "selection" | "all";

export interface KeyStyle {
  backgroundColor: string;
  borderColor: string;
  activeGlowColor: string;
  textColor: string;
  borderRadius: number;
  opacity: number;
  fontFamily: string;
  fontSize: number;
  pressEffect: PressEffect;
  showLabel: boolean;
  showPressCount: boolean;
  showDuration: boolean;
}

/**
 * Static configuration for one key. Nothing here changes when the key is
 * pressed — live press state lives in the store's `activeKeys` / `pressCounts`
 * so that a keystroke never rebuilds this object.
 */
export interface KeyConfig {
  id: string;
  /** Matched verbatim against `InputEventPayload.code`. */
  code: string;
  label: string;
  x: number;
  y: number;
  width: number;
  height: number;
  shape: KeyShape;
  rotation: number;
  scale: number;
  style: KeyStyle;
}

/**
 * Bumped whenever the persisted shape changes incompatibly.
 *
 * v2 renamed the press effects (pulse/ripple/bounce/trail) and two theme ids
 * (neumorphism, rgb-gradient), added the grid fields, and dropped
 * `soundEnabled` / `hotkeyToggleOverlay`, neither of which had an implementation
 * behind it.
 *
 * v3 adds app-scoped visibility: show the HUD only while a matching process
 * (e.g. Celeste) is focused.
 *
 * v4 wraps one-or-more profiles in a library (`activeId` + `profiles[]`) so
 * users can create, copy and switch saved layouts. Individual profiles still
 * carry `version` for per-profile field migrations.
 */
export const PROFILE_SCHEMA_VERSION = 4;
export const LIBRARY_SCHEMA_VERSION = 4;

/** One editable overlay layout. */
export interface ProfileConfig {
  version: number;
  id: string;
  name: string;
  templateId: LayoutTemplateId;
  keys: KeyConfig[];
  globalTheme: VisualTheme;
  showKpsMeter: boolean;
  /** 0.1–1, applied to the overlay canvas. */
  windowOpacity: number;
  snapToGrid: boolean;
  gridSize: number;
  /** When true, the HUD is shown only while the focused app matches. */
  targetAppEnabled: boolean;
  /**
   * Case-insensitive substring matched against the foreground process name
   * (e.g. `Celeste`) or window title.
   */
  targetAppMatch: string;
  createdAt: string;
  updatedAt: string;
}

/** What `save_layout` writes to disk from v4 onward. */
export interface LayoutLibrary {
  version: number;
  activeId: string;
  profiles: ProfileConfig[];
}

export const DEFAULT_KEY_STYLE: KeyStyle = {
  backgroundColor: "rgba(20, 20, 40, 0.75)",
  borderColor: "rgba(0, 255, 255, 0.5)",
  activeGlowColor: "rgba(0, 255, 255, 0.9)",
  textColor: "#ffffff",
  borderRadius: 8,
  opacity: 1,
  fontFamily: "JetBrains Mono",
  fontSize: 14,
  pressEffect: "glow",
  showLabel: true,
  showPressCount: false,
  showDuration: false,
};

/**
 * Registered statically in `src-tauri/src/lib.rs`. They live here as constants
 * rather than as profile fields because nothing can rebind them yet, and a
 * persisted string that the shortcut registration ignores is just a lie.
 */
export const HOTKEY_TOGGLE_VISIBILITY = "Ctrl+Shift+O";
export const HOTKEY_TOGGLE_LOCK = "Ctrl+Shift+L";
export const HOTKEY_OPEN_EDITOR = "Ctrl+Shift+E";

export const DEFAULT_GRID_SIZE = 10;

export const PRESS_EFFECT_LABELS: Record<PressEffect, string> = {
  glow: "Glow",
  "glow-pulse": "Glow Pulse",
  "key-drop": "Key Drop",
  "border-ripple": "Border Ripple",
  none: "None",
};
