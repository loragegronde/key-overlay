import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { motion } from "framer-motion";
import { Minus, MonitorPlay, Power, Settings, Upload } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { exportProfile, saveLibrary } from "../store/persistence";
import { LIBRARY_SCHEMA_VERSION } from "../types";
import { useOverlayStore } from "../store/useOverlayStore";

type Status = { text: string; tone: "ok" | "error" } | null;

const STATUS_MS = 2500;

/**
 * Window controls for the editor, revealed by hovering the bottom edge.
 * Closing the editor does not quit the app once the overlay is live.
 */
export function FloatingControlBar() {
  const openDrawer = useOverlayStore((s) => s.openDrawer);

  const [hovered, setHovered] = useState(false);
  const [status, setStatus] = useState<Status>(null);
  const statusTimer = useRef<ReturnType<typeof setTimeout>>();

  const flash = useCallback((text: string, tone: "ok" | "error") => {
    setStatus({ text, tone });
    clearTimeout(statusTimer.current);
    statusTimer.current = setTimeout(() => setStatus(null), STATUS_MS);
  }, []);

  const onExport = useCallback(async () => {
    const profile = useOverlayStore.getState().profile;
    try {
      const path = await save({
        title: "Export profile",
        defaultPath: `${profile.name.replace(/[^\w-]+/g, "-")}.json`,
        filters: [{ name: "Key Overlay profile", extensions: ["json"] }],
      });
      if (!path) return;
      await exportProfile(path, profile);
      flash(`Exported to ${path.split(/[\\/]/).pop()}`, "ok");
    } catch (error) {
      console.error("export failed", error);
      flash("Export failed", "error");
    }
  }, [flash]);

  const placeOverlay = useCallback(async () => {
    try {
      const { profile, library } = useOverlayStore.getState();
      const profiles = library.profiles.some((p) => p.id === profile.id)
        ? library.profiles.map((p) => (p.id === profile.id ? profile : p))
        : [...library.profiles, profile];
      await saveLibrary({
        version: LIBRARY_SCHEMA_VERSION,
        activeId: profile.id,
        profiles,
      });
      await invoke("launch_overlay", { positioning: true });
      flash("Overlay ready — drag it, then Ctrl+Shift+L to lock", "ok");
    } catch (error) {
      console.error("launch failed", error);
      flash("Could not place overlay", "error");
    }
  }, [flash]);

  return (
    <div
      className="pointer-events-auto absolute bottom-0 left-1/2 z-[150] flex -translate-x-1/2 flex-col items-center pt-10"
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
    >
      <motion.div
        initial={false}
        animate={{ opacity: hovered ? 1 : 0, y: hovered ? 0 : 10 }}
        transition={{ duration: 0.15, ease: "easeOut" }}
        style={{ pointerEvents: hovered ? "auto" : "none" }}
        className="flex flex-col items-center gap-1"
      >
        {status && (
          <span
            className={`rounded-full px-3 py-1 text-[10px] ${
              status.tone === "ok"
                ? "bg-emerald-950/80 text-emerald-300"
                : "bg-red-950/80 text-red-300"
            }`}
          >
            {status.text}
          </span>
        )}

        <div className="flex items-center gap-1 rounded-full border border-white/10 bg-slate-900/90 px-2 py-1.5 shadow-2xl backdrop-blur-md">
          <BarButton
            icon={MonitorPlay}
            label="Place overlay on screen"
            onClick={() => void placeOverlay()}
          />
          <BarButton
            icon={Settings}
            label="Open settings"
            onClick={() => openDrawer("settings")}
          />
          <BarButton icon={Upload} label="Export profile to a file" onClick={() => void onExport()} />
          <span className="mx-0.5 h-5 w-px bg-white/10" />
          <BarButton
            icon={Minus}
            label="Minimise editor"
            onClick={() => void invoke("minimize_window").catch(console.error)}
          />
          <BarButton
            icon={Power}
            label="Close editor (overlay keeps running)"
            danger
            onClick={() => void invoke("close_editor").catch(console.error)}
          />
        </div>
      </motion.div>

      <motion.span
        initial={false}
        animate={{ opacity: hovered ? 0 : 1 }}
        transition={{ duration: 0.15 }}
        className="mb-1.5 mt-1.5 h-1 w-12 rounded-full bg-white/25"
      />
    </div>
  );
}

function BarButton({
  icon: Icon,
  label,
  onClick,
  danger,
}: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  onClick: () => void;
  danger?: boolean;
}) {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      onClick={onClick}
      className={`rounded-full p-2 transition ${
        danger
          ? "text-white/60 hover:bg-red-950/60 hover:text-red-400"
          : "text-white/60 hover:bg-white/10 hover:text-white"
      }`}
    >
      <Icon className="h-4 w-4" />
    </button>
  );
}
