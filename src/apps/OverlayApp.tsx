import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useOverlayStore } from "../store/useOverlayStore";
import { loadLibrary } from "../store/persistence";
import { useGamepadInput } from "../hooks/useGamepadInput";
import { OverlayCanvas } from "../components/OverlayCanvas";
import { KpsMeter } from "../components/KpsMeter";
import type { InputEventPayload } from "../types";
import { HOTKEY_TOGGLE_LOCK } from "../types";

const INPUT_EVENT = "input-event";
const HOTKEY_TOGGLE_LOCK_EVENT = "hotkey-toggle-lock";

/**
 * Transparent HUD window. While placing, the whole window is draggable; after
 * lock (hotkey) it becomes click-through.
 */
export function OverlayApp() {
  useGamepadInput();

  const setLibrary = useOverlayStore((s) => s.setLibrary);
  const setHydrated = useOverlayStore((s) => s.setHydrated);
  const setMode = useOverlayStore((s) => s.setMode);
  const handleInputEvent = useOverlayStore((s) => s.handleInputEvent);
  const tickKps = useOverlayStore((s) => s.tickKps);
  const hydrated = useOverlayStore((s) => s.hydrated);
  const showKpsMeter = useOverlayStore((s) => s.profile.showKpsMeter);
  const [positioning, setPositioning] = useState(true);

  const reload = useCallback(async () => {
    const { library } = await loadLibrary();
    setLibrary(library);
    setHydrated(true);
    setMode("OVERLAY");
  }, [setLibrary, setHydrated, setMode]);

  useEffect(() => {
    void reload();
  }, [reload]);

  useEffect(() => {
    const unlistenInput = listen<InputEventPayload>(INPUT_EVENT, (e) => {
      handleInputEvent(e.payload.code, e.payload.action, e.payload.label);
    });

    const unlistenProfile = listen("profile-changed", () => {
      void reload();
    });

    const unlistenLaunched = listen("overlay-launched", () => {
      setPositioning(true);
      void reload();
    });

    const unlistenFinished = listen("positioning-finished", () => {
      setPositioning(false);
    });

    const unlistenLock = listen(HOTKEY_TOGGLE_LOCK_EVENT, () => {
      void invoke("finish_positioning")
        .then(() => setPositioning(false))
        .catch(console.error);
    });

    invoke("start_input_listener").catch(console.error);

    return () => {
      unlistenInput.then((fn) => fn());
      unlistenProfile.then((fn) => fn());
      unlistenLaunched.then((fn) => fn());
      unlistenFinished.then((fn) => fn());
      unlistenLock.then((fn) => fn());
    };
  }, [handleInputEvent, reload]);

  useEffect(() => {
    const interval = setInterval(tickKps, 200);
    return () => clearInterval(interval);
  }, [tickKps]);

  useEffect(() => {
    invoke("toggle_click_through", { enabled: !positioning }).catch(console.error);
  }, [positioning]);

  if (!hydrated) return null;

  return (
    <div
      className={`relative h-screen w-screen overflow-hidden ${
        positioning ? "cursor-grab active:cursor-grabbing" : ""
      }`}
      style={{ background: "transparent" }}
      onPointerDown={
        positioning
          ? (e) => {
              e.preventDefault();
              void getCurrentWebviewWindow().startDragging();
            }
          : undefined
      }
    >
      {positioning && (
        <div className="pointer-events-none absolute bottom-3 left-1/2 z-[200] -translate-x-1/2 rounded-full border border-white/15 bg-slate-950/80 px-3 py-1.5 text-[11px] text-white/70 backdrop-blur">
          Drag anywhere to move · {HOTKEY_TOGGLE_LOCK} to lock
        </div>
      )}

      {showKpsMeter && (
        <div className="pointer-events-none absolute right-3 top-3 z-[100]">
          <KpsMeter />
        </div>
      )}

      <div className={positioning ? "" : "pointer-events-none"}>
        <OverlayCanvas />
      </div>
    </div>
  );
}
