import { useCallback, useEffect, useRef } from "react";
import { useOverlayStore } from "../store/useOverlayStore";
import { KeyElement } from "./KeyElement";

const MIN_KEY_SIZE = 20;

interface DragSession {
  kind: "drag";
  pointerId: number;
  startX: number;
  startY: number;
  /** Position of the key under the cursor, used as the snap anchor. */
  anchorX: number;
  anchorY: number;
  dx: number;
  dy: number;
}

interface ResizeSession {
  kind: "resize";
  pointerId: number;
  handle: string;
  keyId: string;
  el: HTMLElement;
  scale: number;
  startX: number;
  startY: number;
  origin: { x: number; y: number; width: number; height: number };
  next: { x: number; y: number; width: number; height: number };
}

type Session = DragSession | ResizeSession;

/**
 * Renders the key configs and owns the edit-mode pointer interactions.
 *
 * Deliberately does not subscribe to `activeKeys`, `pressCounts` or
 * `selectedKeyIds` — each KeyElement reads those itself, so a keypress never
 * re-renders the canvas.
 *
 * Drag and resize are handled here by event delegation rather than per-key
 * handlers, and their in-flight geometry lives in a ref plus a pair of CSS
 * custom properties. Nothing is written to the store until pointerup, so a
 * drag costs zero React renders and zero debounced disk writes.
 */
export function OverlayCanvas() {
  const keys = useOverlayStore((s) => s.profile.keys);
  const globalTheme = useOverlayStore((s) => s.profile.globalTheme);
  const windowOpacity = useOverlayStore((s) => s.profile.windowOpacity);
  const snapToGrid = useOverlayStore((s) => s.profile.snapToGrid);
  const gridSize = useOverlayStore((s) => s.profile.gridSize);
  const mode = useOverlayStore((s) => s.mode);

  const rootRef = useRef<HTMLDivElement>(null);
  const session = useRef<Session | null>(null);

  const editMode = mode === "EDIT";

  // Keep the store aware of the canvas box so new keys spawn in-view and
  // drags cannot push keys under the docked chrome (which is outside this node).
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    const report = () => {
      const { clientWidth: width, clientHeight: height } = root;
      if (width > 0 && height > 0) {
        useOverlayStore.getState().setCanvasSize(width, height);
      }
    };
    report();
    const ro = new ResizeObserver(report);
    ro.observe(root);
    return () => ro.disconnect();
  }, []);

  const endSession = useCallback((pointerId: number) => {
    const s = session.current;
    if (!s || s.pointerId !== pointerId) return;
    session.current = null;

    const root = rootRef.current;
    const store = useOverlayStore.getState();

    if (s.kind === "drag") {
      root?.style.removeProperty("--drag-x");
      root?.style.removeProperty("--drag-y");
      if (s.dx !== 0 || s.dy !== 0) store.nudgeSelectedKeys(s.dx, s.dy);
      else store.clampKeysToCanvas();
      return;
    }

    s.el.style.removeProperty("--resize-x");
    s.el.style.removeProperty("--resize-y");
    const { x, y, width, height } = s.next;
    const o = s.origin;
    if (x !== o.x || y !== o.y || width !== o.width || height !== o.height) {
      store.updateKey(s.keyId, { x, y, width, height });
      store.clampKeysToCanvas();
    }
  }, []);

  const onPointerDown = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    const root = rootRef.current;
    if (!root || useOverlayStore.getState().mode !== "EDIT") return;

    const target = e.target as HTMLElement;
    const keyEl = target.closest<HTMLElement>("[data-key-id]");
    if (!keyEl?.dataset.keyId) {
      useOverlayStore.getState().clearSelection();
      return;
    }

    const keyId = keyEl.dataset.keyId;
    const store = useOverlayStore.getState();
    const key = store.profile.keys.find((k) => k.id === keyId);
    if (!key) return;

    const handle = target.closest<HTMLElement>("[data-resize-handle]")?.dataset.resizeHandle;

    e.preventDefault();
    root.setPointerCapture(e.pointerId);

    if (handle) {
      // Resizing is a single-key operation; grabbing a handle implies that key.
      if (!store.selectedKeyIds.includes(keyId)) store.selectKey(keyId);
      const origin = { x: key.x, y: key.y, width: key.width, height: key.height };
      session.current = {
        kind: "resize",
        pointerId: e.pointerId,
        handle,
        keyId,
        el: keyEl,
        scale: key.scale,
        startX: e.clientX,
        startY: e.clientY,
        origin,
        next: { ...origin },
      };
      return;
    }

    if (e.shiftKey) {
      store.selectKey(keyId, true);
    } else if (!store.selectedKeyIds.includes(keyId)) {
      store.selectKey(keyId);
    }

    // A shift+click that removed the key from the selection must not then drag it.
    if (!useOverlayStore.getState().selectedKeyIds.includes(keyId)) return;

    session.current = {
      kind: "drag",
      pointerId: e.pointerId,
      startX: e.clientX,
      startY: e.clientY,
      anchorX: key.x,
      anchorY: key.y,
      dx: 0,
      dy: 0,
    };
  }, []);

  const onPointerMove = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    const s = session.current;
    if (!s || s.pointerId !== e.pointerId) return;

    const profile = useOverlayStore.getState().profile;
    const grid = profile.snapToGrid ? profile.gridSize : 0;
    const quantize = (v: number) => (grid > 0 ? Math.round(v / grid) * grid : Math.round(v));

    if (s.kind === "drag") {
      // Snap the grabbed key to the grid, then shift the whole selection by
      // that same delta so relative spacing inside the selection survives.
      s.dx = quantize(s.anchorX + (e.clientX - s.startX)) - s.anchorX;
      s.dy = quantize(s.anchorY + (e.clientY - s.startY)) - s.anchorY;
      const root = rootRef.current;
      root?.style.setProperty("--drag-x", `${s.dx}px`);
      root?.style.setProperty("--drag-y", `${s.dy}px`);
      return;
    }

    const o = s.origin;
    const rawDx = e.clientX - s.startX;
    const rawDy = e.clientY - s.startY;
    let { x, y, width, height } = o;

    if (s.handle.includes("e")) width = quantize(o.x + o.width + rawDx) - o.x;
    if (s.handle.includes("w")) {
      x = quantize(o.x + rawDx);
      width = o.width + (o.x - x);
    }
    if (s.handle.includes("s")) height = quantize(o.y + o.height + rawDy) - o.y;
    if (s.handle.includes("n")) {
      y = quantize(o.y + rawDy);
      height = o.height + (o.y - y);
    }

    width = Math.max(MIN_KEY_SIZE, width);
    height = Math.max(MIN_KEY_SIZE, height);
    // Re-anchor after clamping so the opposite edge stays put.
    if (s.handle.includes("w")) x = o.x + o.width - width;
    if (s.handle.includes("n")) y = o.y + o.height - height;

    s.next = { x, y, width, height };
    s.el.style.width = `${width * s.scale}px`;
    s.el.style.height = `${height * s.scale}px`;
    s.el.style.setProperty("--resize-x", `${x - o.x}px`);
    s.el.style.setProperty("--resize-y", `${y - o.y}px`);
  }, []);

  const onPointerUp = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => endSession(e.pointerId),
    [endSession],
  );

  // Undo / redo, keyboard nudge and delete for the current selection.
  useEffect(() => {
    if (!editMode) return;

    const onKeyDown = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      if (el && /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName)) return;

      const store = useOverlayStore.getState();
      const mod = e.ctrlKey || e.metaKey;

      if (mod && !e.altKey && e.key.toLowerCase() === "z") {
        e.preventDefault();
        if (e.shiftKey) store.redo();
        else store.undo();
        return;
      }
      if (mod && !e.altKey && e.key.toLowerCase() === "y") {
        e.preventDefault();
        store.redo();
        return;
      }

      if (e.key === "Escape") {
        // Escapes out of a rebind even when the drawer that started it is closed.
        if (store.capturingKeyId) store.cancelCapture();
        else store.clearSelection();
        return;
      }

      // While rebinding (or just after — rdev clears capture before DOM keydown),
      // arrows / delete must assign the binding, not move or remove keys.
      if (
        store.capturingKeyId ||
        performance.now() < store.suppressEditorShortcutsUntil
      ) {
        return;
      }

      if (store.selectedKeyIds.length === 0) return;

      if (e.key === "Delete" || e.key === "Backspace") {
        e.preventDefault();
        store.removeSelectedKeys();
        return;
      }

      const step = e.shiftKey ? 10 : store.profile.snapToGrid ? store.profile.gridSize : 1;
      const delta: Record<string, [number, number]> = {
        ArrowLeft: [-step, 0],
        ArrowRight: [step, 0],
        ArrowUp: [0, -step],
        ArrowDown: [0, step],
      };
      const move = delta[e.key];
      if (!move) return;
      e.preventDefault();
      store.nudgeSelectedKeys(move[0], move[1]);
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [editMode]);

  const themeBg =
    globalTheme === "rgb-wave"
      ? "bg-gradient-to-br from-purple-900/20 via-pink-900/20 to-cyan-900/20"
      : "";

  return (
    <div
      ref={rootRef}
      className={`relative h-full w-full ${themeBg}`}
      style={{ opacity: windowOpacity, touchAction: "none" }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
    >
      {editMode && snapToGrid && (
        <div
          className="pointer-events-none absolute inset-0"
          style={{
            backgroundImage:
              "linear-gradient(to right, rgba(255,255,255,0.07) 1px, transparent 1px)," +
              "linear-gradient(to bottom, rgba(255,255,255,0.07) 1px, transparent 1px)",
            backgroundSize: `${gridSize}px ${gridSize}px`,
          }}
        />
      )}

      {keys.map((key) => (
        <KeyElement key={key.id} keyData={key} editMode={editMode} />
      ))}

      {editMode && keys.length === 0 && (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <p className="rounded-lg border border-dashed border-white/20 bg-black/30 px-6 py-4 text-sm text-white/50 backdrop-blur">
            Add a key or load a layout preset to get started
          </p>
        </div>
      )}
    </div>
  );
}
