import { useEffect, useRef } from "react";
import { useOverlayStore } from "../store/useOverlayStore";

/**
 * Standard Gamepad API button index → overlay code / label.
 * Indices follow the W3C "Standard Gamepad" layout (Xbox-style).
 */
export const GAMEPAD_BUTTONS: { index: number; code: string; label: string }[] = [
  { index: 0, code: "PadA", label: "A" },
  { index: 1, code: "PadB", label: "B" },
  { index: 2, code: "PadX", label: "X" },
  { index: 3, code: "PadY", label: "Y" },
  { index: 4, code: "PadLB", label: "LB" },
  { index: 5, code: "PadRB", label: "RB" },
  { index: 6, code: "PadLT", label: "LT" },
  { index: 7, code: "PadRT", label: "RT" },
  { index: 8, code: "PadBack", label: "Back" },
  { index: 9, code: "PadStart", label: "Start" },
  { index: 10, code: "PadL3", label: "L3" },
  { index: 11, code: "PadR3", label: "R3" },
  { index: 12, code: "PadUp", label: "↑" },
  { index: 13, code: "PadDown", label: "↓" },
  { index: 14, code: "PadLeft", label: "←" },
  { index: 15, code: "PadRight", label: "→" },
];

/** Discrete 4-way bindings from each stick (useful as separate keys). */
const AXIS_DIRS: {
  axis: number;
  stick: "PadLS" | "PadRS";
  neg: { code: string; label: string };
  pos: { code: string; label: string };
}[] = [
  {
    axis: 0,
    stick: "PadLS",
    neg: { code: "PadLSLeft", label: "LS←" },
    pos: { code: "PadLSRight", label: "LS→" },
  },
  {
    axis: 1,
    stick: "PadLS",
    neg: { code: "PadLSUp", label: "LS↑" },
    pos: { code: "PadLSDown", label: "LS↓" },
  },
  {
    axis: 2,
    stick: "PadRS",
    neg: { code: "PadRSLeft", label: "RS←" },
    pos: { code: "PadRSRight", label: "RS→" },
  },
  {
    axis: 3,
    stick: "PadRS",
    neg: { code: "PadRSUp", label: "RS↑" },
    pos: { code: "PadRSDown", label: "RS↓" },
  },
];

const AXIS_THRESHOLD = 0.55;
const STICK_ACTIVE = 0.18;

/**
 * Polls connected gamepads: buttons, discrete stick directions, and live
 * analog stick positions for the joystick widgets.
 */
export function useGamepadInput() {
  const handleInputEvent = useOverlayStore((s) => s.handleInputEvent);
  const setStickAxes = useOverlayStore((s) => s.setStickAxes);
  const prevButtons = useRef<Map<string, boolean>>(new Map());
  const prevAxes = useRef<Map<string, boolean>>(new Map());
  const prevStickActive = useRef<Map<string, boolean>>(new Map());
  const raf = useRef(0);

  useEffect(() => {
    const tick = () => {
      const pads = navigator.getGamepads?.() ?? [];
      for (const pad of pads) {
        if (!pad) continue;

        for (const { index, code, label } of GAMEPAD_BUTTONS) {
          const pressed = !!pad.buttons[index]?.pressed;
          const key = `${pad.index}:${code}`;
          const was = prevButtons.current.get(key) ?? false;
          if (pressed && !was) handleInputEvent(code, "down", label);
          if (!pressed && was) handleInputEvent(code, "up", label);
          prevButtons.current.set(key, pressed);
        }

        const lx = pad.axes[0] ?? 0;
        const ly = pad.axes[1] ?? 0;
        const rx = pad.axes[2] ?? 0;
        const ry = pad.axes[3] ?? 0;
        setStickAxes("PadLS", lx, ly);
        setStickAxes("PadRS", rx, ry);

        for (const [code, x, y, label] of [
          ["PadLS", lx, ly, "LS"] as const,
          ["PadRS", rx, ry, "RS"] as const,
        ]) {
          const active = Math.hypot(x, y) >= STICK_ACTIVE;
          const key = `${pad.index}:${code}`;
          const was = prevStickActive.current.get(key) ?? false;
          if (active && !was) handleInputEvent(code, "down", label);
          if (!active && was) handleInputEvent(code, "up", label);
          prevStickActive.current.set(key, active);
        }

        for (const { axis, neg, pos } of AXIS_DIRS) {
          const value = pad.axes[axis] ?? 0;
          for (const dir of [
            { active: value < -AXIS_THRESHOLD, ...neg },
            { active: value > AXIS_THRESHOLD, ...pos },
          ]) {
            const key = `${pad.index}:${dir.code}`;
            const was = prevAxes.current.get(key) ?? false;
            if (dir.active && !was) handleInputEvent(dir.code, "down", dir.label);
            if (!dir.active && was) handleInputEvent(dir.code, "up", dir.label);
            prevAxes.current.set(key, dir.active);
          }
        }
      }
      raf.current = requestAnimationFrame(tick);
    };

    raf.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf.current);
  }, [handleInputEvent, setStickAxes]);
}
