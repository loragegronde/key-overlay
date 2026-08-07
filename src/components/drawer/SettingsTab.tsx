import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Crosshair, Download } from "lucide-react";
import { useEffect, useState } from "react";
import {
  HOTKEY_TOGGLE_LOCK,
  HOTKEY_TOGGLE_VISIBILITY,
} from "../../types";
import { importProfile } from "../../store/persistence";
import { useOverlayStore } from "../../store/useOverlayStore";
import { Field, SectionHeading, Slider, Toggle } from "../ui/controls";

interface ForegroundApp {
  processName: string;
  windowTitle: string;
}

export function SettingsTab() {
  const name = useOverlayStore((s) => s.profile.name);
  const windowOpacity = useOverlayStore((s) => s.profile.windowOpacity);
  const showKpsMeter = useOverlayStore((s) => s.profile.showKpsMeter);
  const snapToGrid = useOverlayStore((s) => s.profile.snapToGrid);
  const gridSize = useOverlayStore((s) => s.profile.gridSize);
  const targetAppEnabled = useOverlayStore((s) => s.profile.targetAppEnabled);
  const targetAppMatch = useOverlayStore((s) => s.profile.targetAppMatch);
  const setProfileName = useOverlayStore((s) => s.setProfileName);
  const updateProfile = useOverlayStore((s) => s.updateProfile);
  const setSnapToGrid = useOverlayStore((s) => s.setSnapToGrid);
  const setGridSize = useOverlayStore((s) => s.setGridSize);

  const [importError, setImportError] = useState<string | null>(null);
  const [foreground, setForeground] = useState<ForegroundApp | null>(null);

  useEffect(() => {
    void invoke<ForegroundApp>("get_foreground_app")
      .then(setForeground)
      .catch(() => setForeground(null));

    const unlisten = listen<ForegroundApp>("foreground-app", (e) => {
      setForeground(e.payload);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const onImport = async () => {
    setImportError(null);
    try {
      const path = await open({
        title: "Import profile",
        multiple: false,
        directory: false,
        filters: [{ name: "Key Overlay profile", extensions: ["json"] }],
      });
      if (typeof path !== "string") return;
      useOverlayStore.getState().setProfile(await importProfile(path));
    } catch (error) {
      console.error("import failed", error);
      setImportError(error instanceof Error ? error.message : "could not read that file");
    }
  };

  const useCurrentApp = () => {
    if (!foreground?.processName) return;
    // Strip the extension so "Celeste.exe" and "Celeste" both match.
    const match = foreground.processName.replace(/\.exe$/i, "");
    updateProfile({ targetAppEnabled: true, targetAppMatch: match });
  };

  return (
    <div className="space-y-6">
      <section className="space-y-3">
        <SectionHeading>Profile</SectionHeading>
        <Field label="Name">
          <input
            type="text"
            value={name}
            onChange={(e) => setProfileName(e.target.value)}
            className="w-full rounded-lg border border-white/10 bg-black/40 px-3 py-2 text-sm text-white outline-none focus:border-cyan-500/50"
          />
        </Field>

        <button
          type="button"
          onClick={() => void onImport()}
          className="flex w-full items-center justify-center gap-2 rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-xs text-white/70 transition hover:bg-white/10 hover:text-white"
        >
          <Download className="h-3.5 w-3.5" />
          Import profile from a file
        </button>
        {importError && <p className="text-[10px] text-red-400">{importError}</p>}
      </section>

      <section className="space-y-3">
        <SectionHeading>Show only in app</SectionHeading>
        <p className="text-[10px] leading-snug text-white/40">
          When enabled, the overlay appears only while the focused window matches. Switch away
          (e.g. leave Celeste) and it hides automatically.
        </p>
        <Toggle
          label="Enable app filter"
          checked={targetAppEnabled}
          onChange={(v) => updateProfile({ targetAppEnabled: v })}
        />
        <Field label="Process or window title contains">
          <input
            type="text"
            value={targetAppMatch}
            onChange={(e) => updateProfile({ targetAppMatch: e.target.value })}
            placeholder="e.g. Celeste"
            className="w-full rounded-lg border border-white/10 bg-black/40 px-3 py-2 font-mono text-sm text-white outline-none focus:border-cyan-500/50"
          />
        </Field>
        <button
          type="button"
          onClick={useCurrentApp}
          className="flex w-full items-center justify-center gap-2 rounded-lg border border-cyan-500/30 bg-cyan-950/40 px-3 py-2 text-xs text-cyan-200 transition hover:bg-cyan-950/70"
        >
          <Crosshair className="h-3.5 w-3.5" />
          Use currently focused app
        </button>
        {foreground && (
          <p className="text-[10px] text-white/35">
            Focused now:{" "}
            <span className="font-mono text-white/60">{foreground.processName || "—"}</span>
            {foreground.windowTitle ? (
              <span className="text-white/30"> — {foreground.windowTitle}</span>
            ) : null}
          </p>
        )}
      </section>

      <section className="space-y-3">
        <SectionHeading>Overlay</SectionHeading>
        <Slider
          label="Window opacity"
          value={windowOpacity}
          min={0.1}
          max={1}
          step={0.05}
          onChange={(v) => updateProfile({ windowOpacity: v })}
          format={(v) => `${Math.round(v * 100)}%`}
        />
        <Toggle
          label="Show KPS meter"
          checked={showKpsMeter}
          onChange={(v) => updateProfile({ showKpsMeter: v })}
        />
      </section>

      <section className="space-y-3">
        <SectionHeading>Grid</SectionHeading>
        <Toggle label="Snap to grid" checked={snapToGrid} onChange={setSnapToGrid} />
        <Slider
          label="Grid size"
          value={gridSize}
          min={2}
          max={50}
          onChange={setGridSize}
          format={(v) => `${v}px`}
        />
      </section>

      <section className="space-y-2">
        <SectionHeading>Hotkeys</SectionHeading>
        <HotkeyRow
          combo={HOTKEY_TOGGLE_VISIBILITY}
          description="Show or hide the live overlay (still respects the app filter)"
        />
        <HotkeyRow
          combo={HOTKEY_TOGGLE_LOCK}
          description="Finish placing the overlay / re-lock click-through"
        />
        <HotkeyRow combo="Ctrl+Shift+E" description="Reopen this editor window" />
      </section>
    </div>
  );
}

function HotkeyRow({ combo, description }: { combo: string; description: string }) {
  return (
    <div className="rounded-lg border border-white/5 bg-black/20 p-2.5">
      <code className="font-mono text-xs text-cyan-300">{combo}</code>
      <p className="mt-0.5 text-[10px] text-white/40">{description}</p>
    </div>
  );
}
