import { invoke } from "@tauri-apps/api/core";
import type {
  KeyConfig,
  KeyShape,
  KeyStyle,
  LayoutLibrary,
  LayoutTemplateId,
  PressEffect,
  ProfileConfig,
  VisualTheme,
} from "../types";
import {
  DEFAULT_GRID_SIZE,
  DEFAULT_KEY_STYLE,
  LIBRARY_SCHEMA_VERSION,
  PRESS_EFFECT_LABELS,
  PROFILE_SCHEMA_VERSION,
} from "../types";
import { createProfileFromTemplate, LAYOUT_TEMPLATES, VISUAL_THEMES } from "../layouts/templates";

const LEGACY_STORAGE_KEY = "key-overlay-storage";

export type LibrarySource = "file" | "migrated" | "default";

export function createDefaultProfile(): ProfileConfig {
  return createProfileFromTemplate("wasd-gaming");
}

export function createDefaultLibrary(): LayoutLibrary {
  const profile = createDefaultProfile();
  return {
    version: LIBRARY_SCHEMA_VERSION,
    activeId: profile.id,
    profiles: [profile],
  };
}

/**
 * Loads the layout library. Accepts v4 library files and migrates a lone
 * v1–v3 profile (or the old localStorage blob) into a one-entry library.
 */
export async function loadLibrary(): Promise<{
  library: LayoutLibrary;
  source: LibrarySource;
}> {
  let stored: unknown = null;
  try {
    stored = await invoke("load_layout");
  } catch (error) {
    console.error("load_layout failed, falling back to defaults", error);
  }

  const fromFile = normalizeLibrary(stored);
  if (fromFile) return { library: fromFile, source: "file" };

  const legacy = readLegacyProfile();
  if (legacy) {
    const library: LayoutLibrary = {
      version: LIBRARY_SCHEMA_VERSION,
      activeId: legacy.id,
      profiles: [legacy],
    };
    try {
      await saveLibrary(library);
      window.localStorage.removeItem(LEGACY_STORAGE_KEY);
    } catch (error) {
      console.error("could not persist migrated library", error);
    }
    return { library, source: "migrated" };
  }

  return { library: createDefaultLibrary(), source: "default" };
}

/** @deprecated Prefer loadLibrary — kept for call sites that only need the active profile. */
export async function loadProfile(): Promise<{
  profile: ProfileConfig;
  source: LibrarySource;
}> {
  const { library, source } = await loadLibrary();
  const profile =
    library.profiles.find((p) => p.id === library.activeId) ?? library.profiles[0];
  return { profile: profile ?? createDefaultProfile(), source };
}

export async function saveLibrary(library: LayoutLibrary): Promise<void> {
  const profiles = library.profiles.map((p) => ({
    ...p,
    version: PROFILE_SCHEMA_VERSION,
  }));
  await invoke("save_layout", {
    config: {
      version: LIBRARY_SCHEMA_VERSION,
      activeId: library.activeId,
      profiles,
    },
  });
}

/**
 * Persists `profile` as the active layout. Merges into an existing library on
 * disk when present so other saved layouts are not wiped.
 */
export async function saveProfile(profile: ProfileConfig): Promise<void> {
  const next: ProfileConfig = { ...profile, version: PROFILE_SCHEMA_VERSION };
  let library: LayoutLibrary;
  try {
    const stored = await invoke("load_layout");
    const existing = normalizeLibrary(stored);
    if (existing) {
      const profiles = existing.profiles.some((p) => p.id === next.id)
        ? existing.profiles.map((p) => (p.id === next.id ? next : p))
        : [...existing.profiles, next];
      library = {
        version: LIBRARY_SCHEMA_VERSION,
        activeId: next.id,
        profiles,
      };
    } else {
      library = {
        version: LIBRARY_SCHEMA_VERSION,
        activeId: next.id,
        profiles: [next],
      };
    }
  } catch {
    library = {
      version: LIBRARY_SCHEMA_VERSION,
      activeId: next.id,
      profiles: [next],
    };
  }
  await saveLibrary(library);
}

export async function exportProfile(path: string, profile: ProfileConfig): Promise<void> {
  await invoke("export_profile", {
    path,
    config: { ...profile, version: PROFILE_SCHEMA_VERSION },
  });
}

export async function importProfile(path: string): Promise<ProfileConfig> {
  const raw: unknown = await invoke("import_profile", { path });
  const profile = normalizeProfile(raw);
  if (!profile) throw new Error("that file is not a Key Overlay profile");
  return profile;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function str(value: unknown, fallback: string): string {
  return typeof value === "string" && value.length > 0 ? value : fallback;
}

function num(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function bool(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

const LEGACY_THEME_ALIASES: Record<string, VisualTheme> = {
  neumorphism: "stealth-minimal",
  "rgb-gradient": "rgb-wave",
};

const LEGACY_PRESS_EFFECT_ALIASES: Record<string, PressEffect> = {
  pulse: "glow-pulse",
  ripple: "border-ripple",
  bounce: "key-drop",
  trail: "glow",
};

function theme(value: unknown): VisualTheme {
  if (typeof value !== "string") return "cyberpunk";
  if (value in VISUAL_THEMES) return value as VisualTheme;
  return LEGACY_THEME_ALIASES[value] ?? "cyberpunk";
}

function pressEffect(value: unknown): PressEffect {
  if (typeof value !== "string") return DEFAULT_KEY_STYLE.pressEffect;
  if (value in PRESS_EFFECT_LABELS) return value as PressEffect;
  return LEGACY_PRESS_EFFECT_ALIASES[value] ?? DEFAULT_KEY_STYLE.pressEffect;
}

function templateId(value: unknown): LayoutTemplateId {
  return typeof value === "string" && value in LAYOUT_TEMPLATES
    ? (value as LayoutTemplateId)
    : "custom";
}

function shape(value: unknown): KeyShape {
  if (value === "circle" || value === "stick") return value;
  return "rectangle";
}

function normalizeStyle(raw: unknown): KeyStyle {
  if (!isRecord(raw)) return { ...DEFAULT_KEY_STYLE };
  return {
    backgroundColor: str(raw.backgroundColor, DEFAULT_KEY_STYLE.backgroundColor),
    borderColor: str(raw.borderColor, DEFAULT_KEY_STYLE.borderColor),
    activeGlowColor: str(raw.activeGlowColor, DEFAULT_KEY_STYLE.activeGlowColor),
    textColor: str(raw.textColor, DEFAULT_KEY_STYLE.textColor),
    borderRadius: num(raw.borderRadius, DEFAULT_KEY_STYLE.borderRadius),
    opacity: num(raw.opacity, DEFAULT_KEY_STYLE.opacity),
    fontFamily: str(raw.fontFamily, DEFAULT_KEY_STYLE.fontFamily),
    fontSize: num(raw.fontSize, DEFAULT_KEY_STYLE.fontSize),
    pressEffect: pressEffect(raw.pressEffect),
    showLabel: bool(raw.showLabel, DEFAULT_KEY_STYLE.showLabel),
    showPressCount: bool(raw.showPressCount, DEFAULT_KEY_STYLE.showPressCount),
    showDuration: bool(raw.showDuration, DEFAULT_KEY_STYLE.showDuration),
  };
}

function normalizeKey(raw: unknown, index: number): KeyConfig | null {
  if (!isRecord(raw)) return null;
  const code = typeof raw.code === "string" ? raw.code : null;
  if (!code) return null;

  return {
    id: str(raw.id, `key-${index}-${Date.now()}`),
    code,
    label: str(raw.label, code),
    x: num(raw.x, 100),
    y: num(raw.y, 100),
    width: num(raw.width, 56),
    height: num(raw.height, 56),
    shape: shape(raw.shape),
    rotation: num(raw.rotation, 0),
    scale: num(raw.scale, 1),
    style: normalizeStyle(raw.style),
  };
}

export function normalizeProfile(raw: unknown): ProfileConfig | null {
  if (!isRecord(raw) || !Array.isArray(raw.keys)) return null;

  const version = num(raw.version, 0);
  // A library file mistakenly passed here has profiles[] and no keys — reject.
  if (Array.isArray(raw.profiles)) return null;
  if (version > PROFILE_SCHEMA_VERSION) {
    console.warn(
      `profile is version ${version}, this build understands ${PROFILE_SCHEMA_VERSION}; ignoring it`,
    );
    return null;
  }

  const keys = raw.keys
    .map((key, index) => normalizeKey(key, index))
    .filter((key): key is KeyConfig => key !== null);

  const now = new Date().toISOString();
  return {
    version: PROFILE_SCHEMA_VERSION,
    id: str(raw.id, `profile-${Date.now()}`),
    name: str(raw.name, "My Layout"),
    templateId: templateId(raw.templateId),
    keys,
    globalTheme: theme(raw.globalTheme),
    showKpsMeter: bool(raw.showKpsMeter, true),
    windowOpacity: clampOpacity(num(raw.windowOpacity, 1)),
    snapToGrid: bool(raw.snapToGrid, false),
    gridSize: clampGridSize(num(raw.gridSize, DEFAULT_GRID_SIZE)),
    targetAppEnabled: bool(raw.targetAppEnabled, false),
    targetAppMatch: str(raw.targetAppMatch, ""),
    createdAt: str(raw.createdAt, now),
    updatedAt: str(raw.updatedAt, now),
  };
}

export function normalizeLibrary(raw: unknown): LayoutLibrary | null {
  if (!isRecord(raw)) return null;

  // v4 library shape
  if (Array.isArray(raw.profiles)) {
    const version = num(raw.version, 0);
    if (version > LIBRARY_SCHEMA_VERSION) {
      console.warn(
        `layout library is version ${version}, this build understands ${LIBRARY_SCHEMA_VERSION}; ignoring it`,
      );
      return null;
    }
    const profiles = raw.profiles
      .map((p) => normalizeProfile(p))
      .filter((p): p is ProfileConfig => p !== null);
    if (profiles.length === 0) return null;
    const activeId =
      typeof raw.activeId === "string" && profiles.some((p) => p.id === raw.activeId)
        ? raw.activeId
        : profiles[0].id;
    return { version: LIBRARY_SCHEMA_VERSION, activeId, profiles };
  }

  // Lone v1–v3 profile → one-entry library
  const single = normalizeProfile(raw);
  if (!single) return null;
  return {
    version: LIBRARY_SCHEMA_VERSION,
    activeId: single.id,
    profiles: [single],
  };
}

function clampGridSize(value: number): number {
  return Math.min(100, Math.max(2, Math.round(value)));
}

function clampOpacity(value: number): number {
  return Math.min(1, Math.max(0.1, value));
}

function readLegacyProfile(): ProfileConfig | null {
  let raw: string | null = null;
  try {
    raw = window.localStorage.getItem(LEGACY_STORAGE_KEY);
  } catch {
    return null;
  }
  if (!raw) return null;

  try {
    const parsed: unknown = JSON.parse(raw);
    if (!isRecord(parsed) || !isRecord(parsed.state)) return null;

    const { layout, settings } = parsed.state;
    if (!isRecord(layout) || !Array.isArray(layout.keys)) return null;
    const legacySettings = isRecord(settings) ? settings : {};

    const keys = layout.keys
      .map((key, index) => legacyKeyToConfig(key, index))
      .filter((key): key is KeyConfig => key !== null);
    if (keys.length === 0) return null;

    const now = new Date().toISOString();
    return {
      version: PROFILE_SCHEMA_VERSION,
      id: str(layout.id, `profile-${Date.now()}`),
      name: str(layout.name, "Migrated Layout"),
      templateId: templateId(layout.templateId),
      keys,
      globalTheme: theme(layout.globalTheme),
      showKpsMeter: bool(legacySettings.showKpsMeter, true),
      windowOpacity: 1,
      snapToGrid: false,
      gridSize: DEFAULT_GRID_SIZE,
      targetAppEnabled: false,
      targetAppMatch: "",
      createdAt: str(layout.createdAt, now),
      updatedAt: now,
    };
  } catch (error) {
    console.error("could not parse legacy layout, ignoring it", error);
    return null;
  }
}

function legacyKeyToConfig(raw: unknown, index: number): KeyConfig | null {
  if (!isRecord(raw)) return null;

  const binding = isRecord(raw.binding) ? raw.binding : null;
  if (!binding) return null;

  const code =
    binding.type === "mouse" && typeof binding.button === "string"
      ? `Mouse${binding.button}`
      : typeof binding.code === "string"
        ? binding.code
        : null;
  if (!code) return null;

  return normalizeKey({ ...raw, code, shape: "rectangle" }, index);
}
