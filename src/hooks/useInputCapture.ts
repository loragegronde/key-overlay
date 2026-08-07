import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { InputEventPayload } from "../types";
import { useOverlayStore } from "../store/useOverlayStore";

const INPUT_EVENT = "input-event";

/** Map a DOM KeyboardEvent to the same code/label style as the Rust listener. */
function bindingFromDomKey(e: KeyboardEvent): { code: string; label: string } {
  const code = e.code || e.key;

  // Space's `key` is a single character (" "), so the generic length===1 path
  // would store a blank label and look like the bind failed.
  if (code === "Space" || e.key === " ") {
    return { code: "Space", label: "Space" };
  }

  if (e.key.length === 1) {
    return { code, label: e.key.toUpperCase() };
  }

  const label =
    code.replace(/^Key/, "").replace(/^Digit/, "").replace(/^Arrow/, "") || e.key;
  return { code, label };
}

/**
 * Feeds global key/mouse events into the store so the editor can finish a
 * rebind ("Press any key…") and preview presses on the canvas.
 *
 * Keyboard rebind also has a DOM fallback: when the editor window is focused,
 * Tauri/WebView2 can swallow key events before rdev sees them (mouse is fine).
 * While `capturingKeyId` is set we therefore also listen to `window.keydown`.
 */
export function useInputCapture() {
  const handleInputEvent = useOverlayStore((s) => s.handleInputEvent);

  useEffect(() => {
    invoke("start_input_listener").catch(console.error);

    const unlisten = listen<InputEventPayload>(INPUT_EVENT, (e) => {
      handleInputEvent(e.payload.code, e.payload.action, e.payload.label);
    });

    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [handleInputEvent]);

  // DOM fallback for key binding while the editor has focus.
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const capturingKeyId = useOverlayStore.getState().capturingKeyId;
      if (!capturingKeyId) return;

      // Let Escape cancel capture (handled elsewhere); don't bind it by accident.
      if (e.key === "Escape") return;

      // Ignore bare modifiers — wait for the actual key.
      if (e.key === "Shift" || e.key === "Control" || e.key === "Alt" || e.key === "Meta") {
        return;
      }

      e.preventDefault();
      e.stopPropagation();
      // Stop other window listeners (nudge/delete) on this same keydown.
      e.stopImmediatePropagation();

      const { code, label } = bindingFromDomKey(e);
      handleInputEvent(code, "down", label);
    };

    // Space activates a focused <button> on keyup — block that while capturing
    // (and briefly after, same window as the canvas nudge suppress).
    const onKeyUp = (e: KeyboardEvent) => {
      const { capturingKeyId, suppressEditorShortcutsUntil } = useOverlayStore.getState();
      if (!capturingKeyId && performance.now() >= suppressEditorShortcutsUntil) return;
      if (e.code === "Space" || e.key === " ") {
        e.preventDefault();
        e.stopImmediatePropagation();
      }
    };

    // Capture phase so we win over the canvas nudge/delete handlers.
    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
    };
  }, [handleInputEvent]);
}
