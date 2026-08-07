import {
  AlignCenterHorizontal,
  AlignCenterVertical,
  AlignEndHorizontal,
  AlignEndVertical,
  AlignStartHorizontal,
  AlignStartVertical,
  CircleDot,
  Gamepad2,
  Grid3x3,
  Layers,
  MonitorPlay,
  MousePointerClick,
  Palette,
  Plus,
  Settings,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AlignEdge } from "../types";
import { HOTKEY_TOGGLE_VISIBILITY } from "../types";
import { saveLibrary } from "../store/persistence";
import { LIBRARY_SCHEMA_VERSION } from "../types";
import { useOverlayStore, type MouseButton, type StickId } from "../store/useOverlayStore";
import { KpsMeter } from "./KpsMeter";

const ALIGN_ACTIONS: {
  edge: AlignEdge;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}[] = [
  { edge: "left", label: "Align left", icon: AlignStartVertical },
  { edge: "center", label: "Align centre", icon: AlignCenterVertical },
  { edge: "right", label: "Align right", icon: AlignEndVertical },
  { edge: "top", label: "Align top", icon: AlignStartHorizontal },
  { edge: "middle", label: "Align middle", icon: AlignCenterHorizontal },
  { edge: "bottom", label: "Align bottom", icon: AlignEndHorizontal },
];

const MOUSE_BUTTONS: { button: MouseButton; label: string }[] = [
  { button: "left", label: "Left click" },
  { button: "right", label: "Right click" },
  { button: "middle", label: "Middle click" },
];

const STICKS: { stick: StickId; label: string }[] = [
  { stick: "PadLS", label: "Left stick (LS)" },
  { stick: "PadRS", label: "Right stick (RS)" },
];

async function persistActiveLibrary() {
  const { profile, library } = useOverlayStore.getState();
  const profiles = library.profiles.some((p) => p.id === profile.id)
    ? library.profiles.map((p) => (p.id === profile.id ? profile : p))
    : [...library.profiles, profile];
  await saveLibrary({
    version: LIBRARY_SCHEMA_VERSION,
    activeId: profile.id,
    profiles,
  });
}

/**
 * Docked left sidebar — stays out of the canvas so keys never sit under it.
 */
export function Toolbar() {
  const showKpsMeter = useOverlayStore((s) => s.profile.showKpsMeter);
  const profileName = useOverlayStore((s) => s.profile.name);
  const snapToGrid = useOverlayStore((s) => s.profile.snapToGrid);
  const selectedCount = useOverlayStore((s) => s.selectedKeyIds.length);
  const drawerTab = useOverlayStore((s) => s.drawerTab);
  const addKey = useOverlayStore((s) => s.addKey);
  const addMouseZone = useOverlayStore((s) => s.addMouseZone);
  const addControllerPad = useOverlayStore((s) => s.addControllerPad);
  const addJoystick = useOverlayStore((s) => s.addJoystick);
  const removeSelectedKeys = useOverlayStore((s) => s.removeSelectedKeys);
  const alignSelectedKeys = useOverlayStore((s) => s.alignSelectedKeys);
  const setSnapToGrid = useOverlayStore((s) => s.setSnapToGrid);
  const openDrawer = useOverlayStore((s) => s.openDrawer);

  const [mouseMenuOpen, setMouseMenuOpen] = useState(false);
  const [stickMenuOpen, setStickMenuOpen] = useState(false);
  const [launching, setLaunching] = useState(false);

  const placeOverlay = async () => {
    setLaunching(true);
    try {
      await persistActiveLibrary();
      await invoke("launch_overlay", { positioning: true });
    } catch (error) {
      console.error("could not launch overlay", error);
    } finally {
      setLaunching(false);
    }
  };

  return (
    <aside className="flex h-full w-56 shrink-0 flex-col gap-3 border-r border-white/10 bg-slate-950/90 p-3 backdrop-blur-md">
      <div className="rounded-xl border border-white/10 bg-slate-900/80 p-3">
        <h1 className="font-display text-sm font-bold tracking-wider text-cyan-400">
          KEY OVERLAY
        </h1>
        <p className="mt-0.5 truncate text-[10px] text-white/40">{profileName}</p>
      </div>

      <button
        type="button"
        disabled={launching}
        onClick={() => void placeOverlay()}
        className="flex items-center justify-center gap-2 rounded-xl border border-emerald-500/40 bg-emerald-950/60 px-3 py-2.5 text-xs font-medium text-emerald-400 transition hover:bg-emerald-950/80 disabled:opacity-50"
      >
        <MonitorPlay className="h-4 w-4" />
        {launching ? "Placing…" : "Place Overlay"}
      </button>

      {showKpsMeter && (
        <KpsMeter className="rounded-lg border border-cyan-500/30 bg-black/40 px-3 py-1.5 text-center font-mono text-sm text-cyan-400" />
      )}

      <div className="flex flex-1 flex-col gap-1 overflow-y-auto rounded-xl border border-white/10 bg-slate-900/80 p-2">
        <ToolbarButton icon={Plus} label="Add Custom Key" onClick={() => addKey()} />

        <div className="relative">
          <ToolbarButton
            icon={MousePointerClick}
            label="Mouse Click Zone"
            onClick={() => setMouseMenuOpen((open) => !open)}
            active={mouseMenuOpen}
          />
          {mouseMenuOpen && (
            <div className="mt-1 rounded-lg border border-white/10 bg-slate-950 p-1.5">
              {MOUSE_BUTTONS.map(({ button, label }) => (
                <button
                  key={button}
                  type="button"
                  onClick={() => {
                    addMouseZone(button);
                    setMouseMenuOpen(false);
                  }}
                  className="w-full rounded-md px-2.5 py-1.5 text-left text-xs text-white/70 transition hover:bg-white/5 hover:text-white"
                >
                  {label}
                </button>
              ))}
            </div>
          )}
        </div>

        <ToolbarButton
          icon={Gamepad2}
          label="Add Controller Pad"
          onClick={() => addControllerPad()}
        />

        <div className="relative">
          <ToolbarButton
            icon={CircleDot}
            label="Add Joystick"
            onClick={() => setStickMenuOpen((open) => !open)}
            active={stickMenuOpen}
          />
          {stickMenuOpen && (
            <div className="mt-1 rounded-lg border border-white/10 bg-slate-950 p-1.5">
              {STICKS.map(({ stick, label }) => (
                <button
                  key={stick}
                  type="button"
                  onClick={() => {
                    addJoystick(stick);
                    setStickMenuOpen(false);
                  }}
                  className="w-full rounded-md px-2.5 py-1.5 text-left text-xs text-white/70 transition hover:bg-white/5 hover:text-white"
                >
                  {label}
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="my-1 h-px bg-white/10" />

        <ToolbarButton
          icon={Palette}
          label="Customize"
          onClick={() => openDrawer("visuals")}
          active={drawerTab === "visuals" || drawerTab === "themes" || drawerTab === "animations"}
        />
        <ToolbarButton
          icon={Layers}
          label="Layouts"
          onClick={() => openDrawer("layouts")}
          active={drawerTab === "layouts"}
        />
        <ToolbarButton
          icon={Settings}
          label="Settings"
          onClick={() => openDrawer("settings")}
          active={drawerTab === "settings"}
        />

        <div className="my-1 h-px bg-white/10" />

        <ToolbarButton
          icon={Grid3x3}
          label={snapToGrid ? "Grid Snap: On" : "Grid Snap: Off"}
          onClick={() => setSnapToGrid(!snapToGrid)}
          active={snapToGrid}
        />

        {selectedCount > 0 && (
          <ToolbarButton
            icon={Trash2}
            label={`Delete (${selectedCount})`}
            onClick={removeSelectedKeys}
            danger
          />
        )}

        {selectedCount > 1 && (
          <div className="mt-1 rounded-lg border border-white/10 bg-black/30 p-2">
            <p className="px-0.5 pb-1.5 text-[10px] uppercase tracking-wider text-white/40">
              Align {selectedCount}
            </p>
            <div className="grid grid-cols-3 gap-1">
              {ALIGN_ACTIONS.map(({ edge, label, icon: Icon }) => (
                <button
                  key={edge}
                  type="button"
                  title={label}
                  aria-label={label}
                  onClick={() => alignSelectedKeys(edge)}
                  className="flex items-center justify-center rounded-lg p-2 text-white/60 transition hover:bg-white/5 hover:text-white"
                >
                  <Icon className="h-4 w-4" />
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      <p className="text-[10px] leading-relaxed text-white/35">
        Drag keys in the centre canvas. Ctrl+Z / Ctrl+Y undo and redo. Place
        Overlay, drag the HUD, then Ctrl+Shift+L to lock.{" "}
        {HOTKEY_TOGGLE_VISIBILITY} toggles visibility.
      </p>
    </aside>
  );
}

function ToolbarButton({
  icon: Icon,
  label,
  onClick,
  active,
  danger,
}: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  onClick: () => void;
  active?: boolean;
  danger?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-xs transition ${
        danger
          ? "text-red-400 hover:bg-red-950/50"
          : active
            ? "bg-cyan-950/60 text-cyan-400"
            : "text-white/70 hover:bg-white/5 hover:text-white"
      }`}
    >
      <Icon className="h-3.5 w-3.5 shrink-0" />
      {label}
    </button>
  );
}
