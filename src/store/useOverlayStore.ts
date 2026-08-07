import { create } from "zustand";
import type {
  AlignEdge,
  DrawerTab,
  InputAction,
  KeyConfig,
  KeyStyle,
  LayoutLibrary,
  LayoutTemplateId,
  ProfileConfig,
  ToolMode,
  VisualTheme,
} from "../types";
import { DEFAULT_GRID_SIZE, LIBRARY_SCHEMA_VERSION } from "../types";
import { createKey, createProfileFromTemplate, VISUAL_THEMES } from "../layouts/templates";
import { createDefaultLibrary } from "./persistence";

export type MouseButton = "left" | "right" | "middle";
export type StickId = "PadLS" | "PadRS";

const MOUSE_ZONES: Record<MouseButton, { code: string; label: string }> = {
  left: { code: "Mouseleft", label: "LMB" },
  right: { code: "Mouseright", label: "RMB" },
  middle: { code: "Mousemiddle", label: "MMB" },
};

const STICK_DEFS: Record<StickId, { code: StickId; label: string }> = {
  PadLS: { code: "PadLS", label: "LS" },
  PadRS: { code: "PadRS", label: "RS" },
};

const MAX_HISTORY = 50;
/** After a rebind, ignore canvas arrow/delete briefly (rdev finishes before DOM keydown). */
const SUPPRESS_EDITOR_SHORTCUTS_MS = 400;

function cloneProfile(profile: ProfileConfig): ProfileConfig {
  return {
    ...profile,
    keys: profile.keys.map((k) => ({ ...k, style: { ...k.style } })),
  };
}

interface OverlayState {
  /** The layout currently being edited / shown. */
  profile: ProfileConfig;
  /** All saved layouts. The active entry is kept in sync on switch/create/save. */
  library: LayoutLibrary;
  mode: ToolMode;
  /** False until the profile has been read from disk. */
  hydrated: boolean;

  // --- Ephemeral runtime state. Never written to disk. ---
  /** Codes currently held down. Replaced (never mutated) so selectors fire. */
  activeKeys: Set<string>;
  /** Session press totals, keyed by code. */
  pressCounts: Record<string, number>;
  /** Live joystick axes in [-1, 1], keyed by PadLS / PadRS. */
  stickAxes: Record<string, { x: number; y: number }>;
  selectedKeyIds: string[];
  kps: number;
  kpsHistory: number[];
  /** Which drawer panel is open, or null when the drawer is closed. */
  drawerTab: DrawerTab | null;
  /** When set, the next input event is bound to this key instead of counted. */
  capturingKeyId: string | null;
  /**
   * performance.now() deadline — while in the future, canvas must not treat
   * arrow/delete as move/remove (rebinding those keys).
   */
  suppressEditorShortcutsUntil: number;
  /** Undo stack of profile snapshots (layout edits). */
  past: ProfileConfig[];
  /** Redo stack. */
  future: ProfileConfig[];
  /** Latest editor canvas size — used to spawn keys in view and clamp drags. */
  canvasSize: { width: number; height: number };

  setProfile: (profile: ProfileConfig) => void;
  setLibrary: (library: LayoutLibrary) => void;
  setHydrated: (hydrated: boolean) => void;
  updateProfile: (partial: Partial<ProfileConfig>) => void;
  setProfileName: (name: string) => void;

  createLayout: (name?: string) => void;
  duplicateLayout: (id?: string) => void;
  deleteLayout: (id: string) => void;
  switchLayout: (id: string) => void;
  renameLayout: (id: string, name: string) => void;

  setMode: (mode: ToolMode) => void;
  toggleMode: () => void;

  openDrawer: (tab: DrawerTab) => void;
  closeDrawer: () => void;

  selectKey: (id: string, additive?: boolean) => void;
  selectAll: () => void;
  clearSelection: () => void;
  updateKey: (id: string, partial: Partial<KeyConfig>) => void;
  updateKeyStyle: (id: string, partial: Partial<KeyStyle>) => void;
  updateSelectedKeys: (partial: Partial<KeyConfig>) => void;
  updateSelectedKeyStyles: (partial: Partial<KeyStyle>) => void;
  updateAllKeyStyles: (partial: Partial<KeyStyle>) => void;
  /** Applies one delta to every selected key. Called once, on pointerup. */
  nudgeSelectedKeys: (dx: number, dy: number) => void;
  alignSelectedKeys: (edge: AlignEdge) => void;
  addKey: (code?: string, label?: string) => void;
  addMouseZone: (button: MouseButton) => void;
  addControllerPad: () => void;
  addJoystick: (stick: StickId) => void;
  setCanvasSize: (width: number, height: number) => void;
  clampKeysToCanvas: () => void;
  setStickAxes: (code: string, x: number, y: number) => void;
  removeKey: (id: string) => void;
  removeSelectedKeys: () => void;
  duplicateKey: (id: string) => void;

  setSnapToGrid: (snapToGrid: boolean) => void;
  setGridSize: (gridSize: number) => void;

  startCapture: (id: string) => void;
  cancelCapture: () => void;

  loadTemplate: (templateId: LayoutTemplateId) => void;
  applyTheme: (theme: VisualTheme) => void;

  undo: () => void;
  redo: () => void;

  handleInputEvent: (code: string, action: InputAction, label: string) => void;
  tickKps: () => void;
}

/** Snapshot current profile into the undo stack before applying an edit. */
function withHistory(
  s: Pick<OverlayState, "profile" | "past">,
): Pick<OverlayState, "past" | "future"> {
  return {
    past: [...s.past.slice(-(MAX_HISTORY - 1)), cloneProfile(s.profile)],
    future: [],
  };
}

/**
 * Buttons stay focused after click; Space would re-activate them on keyup
 * instead of binding. Blur so the rebind keystroke is unambiguous.
 */
function blurActiveElement() {
  const el = document.activeElement;
  if (el instanceof HTMLElement) el.blur();
}

function withKeys(profile: ProfileConfig, keys: KeyConfig[]): ProfileConfig {
  return { ...profile, keys, updatedAt: new Date().toISOString() };
}

/**
 * Hand-editing colours means the layout no longer matches whichever preset was
 * applied, so the theme selector stops claiming otherwise.
 */
function asCustomTheme(profile: ProfileConfig, keys: KeyConfig[]): ProfileConfig {
  return { ...withKeys(profile, keys), globalTheme: "custom" };
}

/** Writes the working profile back into the library list (and sets it active). */
function flushIntoLibrary(library: LayoutLibrary, profile: ProfileConfig): LayoutLibrary {
  const profiles = library.profiles.some((p) => p.id === profile.id)
    ? library.profiles.map((p) => (p.id === profile.id ? profile : p))
    : [...library.profiles, profile];
  return {
    version: LIBRARY_SCHEMA_VERSION,
    activeId: profile.id,
    profiles,
  };
}

export const useOverlayStore = create<OverlayState>()((set, get) => ({
  profile: createProfileFromTemplate("wasd-gaming"),
  library: createDefaultLibrary(),
  mode: "EDIT",
  hydrated: false,
  canvasSize: { width: 800, height: 560 },

  activeKeys: new Set<string>(),
  pressCounts: {},
  stickAxes: {},
  selectedKeyIds: [],
  kps: 0,
  kpsHistory: [],
  drawerTab: null,
  capturingKeyId: null,
  suppressEditorShortcutsUntil: 0,
  past: [],
  future: [],

  setProfile: (profile) =>
    set((s) => ({
      profile,
      library: flushIntoLibrary(s.library, profile),
      selectedKeyIds: [],
      past: [],
      future: [],
    })),

  setLibrary: (library) => {
    const active =
      library.profiles.find((p) => p.id === library.activeId) ?? library.profiles[0];
    if (!active) return;
    set({
      library: { ...library, activeId: active.id },
      profile: active,
      selectedKeyIds: [],
      capturingKeyId: null,
      past: [],
      future: [],
    });
  },

  setHydrated: (hydrated) => set({ hydrated }),

  updateProfile: (partial) =>
    set((s) => ({
      profile: { ...s.profile, ...partial, updatedAt: new Date().toISOString() },
    })),

  setProfileName: (name) =>
    set((s) => ({ profile: { ...s.profile, name, updatedAt: new Date().toISOString() } })),

  createLayout: (name) =>
    set((s) => {
      const library = flushIntoLibrary(s.library, s.profile);
      const fresh = createProfileFromTemplate("custom", name ?? `Layout ${library.profiles.length + 1}`);
      return {
        library: {
          version: LIBRARY_SCHEMA_VERSION,
          activeId: fresh.id,
          profiles: [...library.profiles, fresh],
        },
        profile: fresh,
        selectedKeyIds: [],
        capturingKeyId: null,
        past: [],
        future: [],
      };
    }),

  duplicateLayout: (id) =>
    set((s) => {
      const library = flushIntoLibrary(s.library, s.profile);
      const source = library.profiles.find((p) => p.id === (id ?? s.profile.id));
      if (!source) return s;
      const now = new Date().toISOString();
      const copy: ProfileConfig = {
        ...source,
        id: `profile-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        name: `${source.name} copy`,
        keys: source.keys.map((k) => ({
          ...k,
          id: `${k.id}-copy-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
          style: { ...k.style },
        })),
        createdAt: now,
        updatedAt: now,
        templateId: "custom",
      };
      return {
        library: {
          version: LIBRARY_SCHEMA_VERSION,
          activeId: copy.id,
          profiles: [...library.profiles, copy],
        },
        profile: copy,
        selectedKeyIds: [],
        capturingKeyId: null,
        past: [],
        future: [],
      };
    }),

  deleteLayout: (id) =>
    set((s) => {
      if (s.library.profiles.length <= 1) return s;
      const library = flushIntoLibrary(s.library, s.profile);
      const profiles = library.profiles.filter((p) => p.id !== id);
      if (profiles.length === library.profiles.length) return s;
      const active =
        id === library.activeId
          ? profiles[0]
          : profiles.find((p) => p.id === library.activeId) ?? profiles[0];
      return {
        library: { version: LIBRARY_SCHEMA_VERSION, activeId: active.id, profiles },
        profile: active,
        selectedKeyIds: [],
        capturingKeyId: null,
        past: [],
        future: [],
      };
    }),

  switchLayout: (id) =>
    set((s) => {
      if (id === s.profile.id) return s;
      const library = flushIntoLibrary(s.library, s.profile);
      const next = library.profiles.find((p) => p.id === id);
      if (!next) return s;
      return {
        library: { ...library, activeId: id },
        profile: next,
        selectedKeyIds: [],
        capturingKeyId: null,
        activeKeys: new Set(),
        past: [],
        future: [],
      };
    }),

  renameLayout: (id, name) =>
    set((s) => {
      const trimmed = name.trim();
      if (!trimmed) return s;
      const library = flushIntoLibrary(s.library, s.profile);
      const profiles = library.profiles.map((p) =>
        p.id === id ? { ...p, name: trimmed, updatedAt: new Date().toISOString() } : p,
      );
      const profile =
        s.profile.id === id
          ? { ...s.profile, name: trimmed, updatedAt: new Date().toISOString() }
          : s.profile;
      return {
        library: { ...library, profiles },
        profile,
      };
    }),

  setMode: (mode) =>
    set((s) =>
      s.mode === mode ? s : { mode, selectedKeyIds: [], drawerTab: null, capturingKeyId: null },
    ),

  toggleMode: () =>
    set((s) => ({
      mode: s.mode === "EDIT" ? "OVERLAY" : "EDIT",
      selectedKeyIds: [],
      drawerTab: null,
      capturingKeyId: null,
    })),

  openDrawer: (tab) => set({ drawerTab: tab }),
  closeDrawer: () => set({ drawerTab: null, capturingKeyId: null }),

  selectKey: (id, additive = false) =>
    set((s) => ({
      selectedKeyIds: additive
        ? s.selectedKeyIds.includes(id)
          ? s.selectedKeyIds.filter((k) => k !== id)
          : [...s.selectedKeyIds, id]
        : [id],
    })),

  selectAll: () => set((s) => ({ selectedKeyIds: s.profile.keys.map((k) => k.id) })),

  clearSelection: () => set((s) => (s.selectedKeyIds.length === 0 ? s : { selectedKeyIds: [] })),

  updateKey: (id, partial) =>
    set((s) => ({
      ...withHistory(s),
      profile: withKeys(
        s.profile,
        // Only the edited key gets a new object identity, so every other
        // KeyElement stays memoised.
        s.profile.keys.map((k) => (k.id === id ? { ...k, ...partial } : k)),
      ),
    })),

  updateKeyStyle: (id, partial) =>
    set((s) => ({
      profile: asCustomTheme(
        s.profile,
        s.profile.keys.map((k) =>
          k.id === id ? { ...k, style: { ...k.style, ...partial } } : k,
        ),
      ),
    })),

  updateSelectedKeys: (partial) =>
    set((s) => ({
      ...withHistory(s),
      profile: withKeys(
        s.profile,
        s.profile.keys.map((k) =>
          s.selectedKeyIds.includes(k.id) ? { ...k, ...partial } : k,
        ),
      ),
    })),

  updateSelectedKeyStyles: (partial) =>
    set((s) => ({
      profile: asCustomTheme(
        s.profile,
        s.profile.keys.map((k) =>
          s.selectedKeyIds.includes(k.id) ? { ...k, style: { ...k.style, ...partial } } : k,
        ),
      ),
    })),

  updateAllKeyStyles: (partial) =>
    set((s) => ({
      profile: asCustomTheme(
        s.profile,
        s.profile.keys.map((k) => ({ ...k, style: { ...k.style, ...partial } })),
      ),
    })),

  nudgeSelectedKeys: (dx, dy) =>
    set((s) => {
      if ((dx === 0 && dy === 0) || s.selectedKeyIds.length === 0) return s;
      const { width: cw, height: ch } = s.canvasSize;
      return {
        ...withHistory(s),
        profile: withKeys(
          s.profile,
          s.profile.keys.map((k) => {
            if (!s.selectedKeyIds.includes(k.id)) return k;
            const w = k.width * k.scale;
            const h = k.height * k.scale;
            return {
              ...k,
              x: Math.round(Math.max(0, Math.min(k.x + dx, Math.max(0, cw - w)))),
              y: Math.round(Math.max(0, Math.min(k.y + dy, Math.max(0, ch - h)))),
            };
          }),
        ),
      };
    }),

  setCanvasSize: (width, height) =>
    set((s) =>
      s.canvasSize.width === width && s.canvasSize.height === height
        ? s
        : { canvasSize: { width, height } },
    ),

  clampKeysToCanvas: () =>
    set((s) => {
      const { width: cw, height: ch } = s.canvasSize;
      let changed = false;
      const keys = s.profile.keys.map((k) => {
        const w = k.width * k.scale;
        const h = k.height * k.scale;
        const x = Math.round(Math.max(0, Math.min(k.x, Math.max(0, cw - w))));
        const y = Math.round(Math.max(0, Math.min(k.y, Math.max(0, ch - h))));
        if (x === k.x && y === k.y) return k;
        changed = true;
        return { ...k, x, y };
      });
      return changed ? { profile: withKeys(s.profile, keys) } : s;
    }),

  alignSelectedKeys: (edge) =>
    set((s) => {
      const selected = s.profile.keys.filter((k) => s.selectedKeyIds.includes(k.id));
      if (selected.length < 2) return s;

      const left = Math.min(...selected.map((k) => k.x));
      const right = Math.max(...selected.map((k) => k.x + k.width));
      const top = Math.min(...selected.map((k) => k.y));
      const bottom = Math.max(...selected.map((k) => k.y + k.height));

      const place = (k: KeyConfig): Partial<KeyConfig> => {
        switch (edge) {
          case "left":
            return { x: left };
          case "right":
            return { x: right - k.width };
          case "center":
            return { x: Math.round((left + right) / 2 - k.width / 2) };
          case "top":
            return { y: top };
          case "bottom":
            return { y: bottom - k.height };
          case "middle":
            return { y: Math.round((top + bottom) / 2 - k.height / 2) };
        }
      };

      return {
        ...withHistory(s),
        profile: withKeys(
          s.profile,
          s.profile.keys.map((k) =>
            s.selectedKeyIds.includes(k.id) ? { ...k, ...place(k) } : k,
          ),
        ),
      };
    }),

  addKey: (code, label) =>
    set((s) => {
      const { width: cw, height: ch } = s.canvasSize;
      const newKey = createKey({
        label: label ?? "New Key",
        code: code ?? "KeyA",
        x: Math.max(24, Math.round(cw / 2 - 28)),
        y: Math.max(24, Math.round(ch / 2 - 28)),
      });
      blurActiveElement();
      return {
        ...withHistory(s),
        profile: withKeys(s.profile, [...s.profile.keys, newKey]),
        selectedKeyIds: [newKey.id],
        // Drop straight into rebind so the key is never left on a placeholder
        // code the user did not choose.
        capturingKeyId: newKey.id,
      };
    }),

  addMouseZone: (button) =>
    set((s) => {
      const { code, label } = MOUSE_ZONES[button];
      const { width: cw, height: ch } = s.canvasSize;
      const zone = createKey({
        code,
        label,
        x: Math.max(24, Math.round(cw / 2 - 36)),
        y: Math.max(24, Math.round(ch / 2 - 36)),
        width: 72,
        height: 72,
        shape: "circle",
      });
      return {
        ...withHistory(s),
        profile: withKeys(s.profile, [...s.profile.keys, zone]),
        selectedKeyIds: [zone.id],
      };
    }),

  addControllerPad: () =>
    set((s) => {
      const pad = createProfileFromTemplate("controller");
      // Offset into the current canvas rather than stacking at 0,0 under the chrome.
      const ox = 40;
      const oy = 40;
      const keys = pad.keys.map((k) =>
        createKey({
          ...k,
          id: undefined,
          x: k.x + ox,
          y: k.y + oy,
          style: { ...k.style },
        }),
      );
      return {
        ...withHistory(s),
        profile: withKeys(s.profile, [...s.profile.keys, ...keys]),
        selectedKeyIds: keys.map((k) => k.id),
      };
    }),

  addJoystick: (stick) =>
    set((s) => {
      const { code, label } = STICK_DEFS[stick];
      const { width: cw, height: ch } = s.canvasSize;
      const zone = createKey({
        code,
        label,
        x: Math.max(24, Math.round(cw / 2 - 40)),
        y: Math.max(24, Math.round(ch / 2 - 40)),
        width: 88,
        height: 88,
        shape: "stick",
        style: {
          ...createKey({ code, label }).style,
          borderRadius: 44,
          showPressCount: false,
          showDuration: false,
        },
      });
      return {
        ...withHistory(s),
        profile: withKeys(s.profile, [...s.profile.keys, zone]),
        selectedKeyIds: [zone.id],
      };
    }),

  setStickAxes: (code, x, y) =>
    set((s) => {
      const prev = s.stickAxes[code];
      if (
        prev &&
        Math.abs(prev.x - x) < 0.02 &&
        Math.abs(prev.y - y) < 0.02
      ) {
        return s;
      }
      return { stickAxes: { ...s.stickAxes, [code]: { x, y } } };
    }),

  removeKey: (id) =>
    set((s) => ({
      ...withHistory(s),
      profile: withKeys(
        s.profile,
        s.profile.keys.filter((k) => k.id !== id),
      ),
      selectedKeyIds: s.selectedKeyIds.filter((k) => k !== id),
      capturingKeyId: s.capturingKeyId === id ? null : s.capturingKeyId,
    })),

  removeSelectedKeys: () =>
    set((s) => {
      if (s.selectedKeyIds.length === 0) return s;
      return {
        ...withHistory(s),
        profile: withKeys(
          s.profile,
          s.profile.keys.filter((k) => !s.selectedKeyIds.includes(k.id)),
        ),
        selectedKeyIds: [],
        capturingKeyId: null,
      };
    }),

  duplicateKey: (id) =>
    set((s) => {
      const source = s.profile.keys.find((k) => k.id === id);
      if (!source) return s;
      const clone = createKey({
        ...source,
        id: undefined,
        x: source.x + 20,
        y: source.y + 20,
        style: { ...source.style },
      });
      return {
        ...withHistory(s),
        profile: withKeys(s.profile, [...s.profile.keys, clone]),
        selectedKeyIds: [clone.id],
      };
    }),

  setSnapToGrid: (snapToGrid) =>
    set((s) => ({ profile: { ...s.profile, snapToGrid, updatedAt: new Date().toISOString() } })),

  setGridSize: (gridSize) =>
    set((s) => ({
      profile: {
        ...s.profile,
        gridSize: Math.min(100, Math.max(2, Math.round(gridSize) || DEFAULT_GRID_SIZE)),
        updatedAt: new Date().toISOString(),
      },
    })),

  startCapture: (id) => {
    blurActiveElement();
    set({ capturingKeyId: id });
  },
  cancelCapture: () => set((s) => (s.capturingKeyId === null ? s : { capturingKeyId: null })),

  loadTemplate: (templateId) =>
    set((s) => {
      const template = createProfileFromTemplate(templateId);
      return {
        ...withHistory(s),
        profile: {
          ...template,
          id: s.profile.id,
          // Template choice replaces the layout, not the user's preferences.
          showKpsMeter: s.profile.showKpsMeter,
          windowOpacity: s.profile.windowOpacity,
          snapToGrid: s.profile.snapToGrid,
          gridSize: s.profile.gridSize,
          targetAppEnabled: s.profile.targetAppEnabled,
          targetAppMatch: s.profile.targetAppMatch,
          globalTheme: s.profile.globalTheme,
          createdAt: s.profile.createdAt,
        },
        selectedKeyIds: [],
      };
    }),

  applyTheme: (theme) =>
    set((s) => {
      const themeStyle = VISUAL_THEMES[theme].globalStyle;
      return {
        ...withHistory(s),
        profile: {
          ...withKeys(
            s.profile,
            s.profile.keys.map((k) => ({ ...k, style: { ...k.style, ...themeStyle } })),
          ),
          globalTheme: theme,
        },
      };
    }),

  undo: () =>
    set((s) => {
      if (s.past.length === 0) return s;
      const previous = s.past[s.past.length - 1];
      return {
        profile: previous,
        past: s.past.slice(0, -1),
        future: [cloneProfile(s.profile), ...s.future].slice(0, MAX_HISTORY),
        selectedKeyIds: [],
        capturingKeyId: null,
      };
    }),

  redo: () =>
    set((s) => {
      if (s.future.length === 0) return s;
      const next = s.future[0];
      return {
        profile: next,
        future: s.future.slice(1),
        past: [...s.past, cloneProfile(s.profile)].slice(-MAX_HISTORY),
        selectedKeyIds: [],
        capturingKeyId: null,
      };
    }),

  handleInputEvent: (code, action, label) => {
    if (action === "down") {
      // Rebind consumes the event instead of counting it, so the keystroke
      // that assigns a binding does not also register as a press.
      const capturingKeyId = get().capturingKeyId;
      if (capturingKeyId) {
        set((s) => ({
          ...withHistory(s),
          capturingKeyId: null,
          // rdev often delivers the bind before the DOM keydown; keep arrows
          // from nudging the selection for a short window after.
          suppressEditorShortcutsUntil: performance.now() + SUPPRESS_EDITOR_SHORTCUTS_MS,
          profile: withKeys(
            s.profile,
            s.profile.keys.map((k) => {
              if (k.id !== capturingKeyId) return k;
              // Keep a hand-edited display name; only auto-fill for placeholders.
              const keepLabel =
                k.label &&
                k.label !== "New Key" &&
                k.label !== k.code &&
                k.label !== "…";
              return {
                ...k,
                code,
                label: keepLabel ? k.label : label === " " ? "Space" : label || code,
              };
            }),
          ),
        }));
        return;
      }

      set((s) => {
        // Ignore OS auto-repeat: a held key must not re-allocate state or
        // inflate its press counter dozens of times a second.
        if (s.activeKeys.has(code)) return s;

        const activeKeys = new Set(s.activeKeys);
        activeKeys.add(code);

        const now = Date.now();
        const kpsHistory = s.kpsHistory.filter((t) => now - t < 1000);
        kpsHistory.push(now);

        return {
          activeKeys,
          pressCounts: { ...s.pressCounts, [code]: (s.pressCounts[code] ?? 0) + 1 },
          kpsHistory,
          kps: kpsHistory.length,
        };
      });
      return;
    }

    set((s) => {
      if (!s.activeKeys.has(code)) return s;
      const activeKeys = new Set(s.activeKeys);
      activeKeys.delete(code);
      return { activeKeys };
    });
  },

  // Only writes when a timestamp actually expired, so an idle app is silent.
  tickKps: () => {
    const { kpsHistory, kps } = get();
    if (kpsHistory.length === 0) {
      if (kps !== 0) set({ kps: 0 });
      return;
    }
    const now = Date.now();
    const next = kpsHistory.filter((t) => now - t < 1000);
    if (next.length === kpsHistory.length) return;
    set({ kpsHistory: next, kps: next.length });
  },
}));
