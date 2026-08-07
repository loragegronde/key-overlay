import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useProfilePersistence } from "../hooks/useProfilePersistence";
import { useInputCapture } from "../hooks/useInputCapture";
import { useGamepadInput } from "../hooks/useGamepadInput";
import { useOverlayStore } from "../store/useOverlayStore";
import { CustomizationDrawer } from "../components/CustomizationDrawer";
import { FloatingControlBar } from "../components/FloatingControlBar";
import { OverlayCanvas } from "../components/OverlayCanvas";
import { Toolbar } from "../components/Toolbar";

/**
 * Decorated window for building layouts.
 *
 * Layout: docked toolbar | canvas workspace | docked drawer.
 * Keys only live in the centre pane, so they cannot hide under settings UI.
 */
export function EditorApp() {
  useProfilePersistence();
  useInputCapture();
  useGamepadInput();

  const hydrated = useOverlayStore((s) => s.hydrated);
  const capturingKeyId = useOverlayStore((s) => s.capturingKeyId);
  const setMode = useOverlayStore((s) => s.setMode);
  const cancelCapture = useOverlayStore((s) => s.cancelCapture);
  const targetAppEnabled = useOverlayStore((s) => s.profile.targetAppEnabled);
  const targetAppMatch = useOverlayStore((s) => s.profile.targetAppMatch);

  useEffect(() => {
    setMode("EDIT");
  }, [setMode]);

  useEffect(() => {
    if (!hydrated) return;
    invoke("set_app_filter", {
      enabled: targetAppEnabled,
      matchText: targetAppMatch,
    }).catch(console.error);
  }, [hydrated, targetAppEnabled, targetAppMatch]);

  if (!hydrated) return null;

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-slate-950 text-white">
      <Toolbar />

      <main className="relative min-w-0 flex-1 overflow-hidden bg-[radial-gradient(ellipse_at_top,_rgba(34,211,238,0.06),_transparent_55%)]">
        <div className="absolute inset-0 border border-white/5">
          <OverlayCanvas />
        </div>

        {capturingKeyId && (
          <div className="pointer-events-none absolute left-1/2 top-4 z-[300] -translate-x-1/2">
            <div className="flex items-center gap-3 rounded-full border border-amber-400/50 bg-amber-950/90 px-4 py-2 text-xs text-amber-100 shadow-xl backdrop-blur">
              <span className="font-medium">Press a key, mouse button, or controller…</span>
              <button
                type="button"
                className="pointer-events-auto rounded-full bg-white/10 px-2.5 py-1 text-[11px] text-white/80 transition hover:bg-white/20"
                onClick={cancelCapture}
              >
                Cancel (Esc)
              </button>
            </div>
          </div>
        )}

        <FloatingControlBar />
      </main>

      <CustomizationDrawer />
    </div>
  );
}
