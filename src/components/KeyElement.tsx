import { motion, type TargetAndTransition, type Transition } from "framer-motion";
import { memo, useEffect, useRef } from "react";
import type { KeyConfig, PressEffect } from "../types";
import { pressedFill, withAlpha } from "../lib/color";
import { useOverlayStore } from "../store/useOverlayStore";

interface KeyElementProps {
  keyData: KeyConfig;
  editMode: boolean;
}

const RESIZE_HANDLES = [
  { corner: "nw", className: "-left-1 -top-1 cursor-nwse-resize" },
  { corner: "ne", className: "-right-1 -top-1 cursor-nesw-resize" },
  { corner: "sw", className: "-bottom-1 -left-1 cursor-nesw-resize" },
  { corner: "se", className: "-bottom-1 -right-1 cursor-nwse-resize" },
] as const;

const SPRING: Transition = { type: "spring", stiffness: 500, damping: 30 };
const TRANSPARENT_SHADOW = "0 0 0 0 rgba(0, 0, 0, 0)";

const REST: TargetAndTransition = { scale: 1, y: 0, boxShadow: TRANSPARENT_SHADOW };

/**
 * Press animations are described here as framer-motion targets rather than as
 * Tailwind utility classes. Building class names at runtime (`animate-${effect}`)
 * put them beyond the reach of Tailwind's scanner, so they were purged from the
 * stylesheet and half the effects silently did nothing.
 */
function pressTarget(
  effect: PressEffect,
  glow: string,
): { target: TargetAndTransition; transition: Transition } {
  switch (effect) {
    case "glow":
      return {
        target: { scale: 0.96, y: 0, boxShadow: `0 0 24px 4px ${glow}` },
        transition: SPRING,
      };
    case "glow-pulse":
      return {
        target: {
          scale: 0.96,
          y: 0,
          boxShadow: [
            `0 0 0 0 ${withAlpha(glow, 0.7)}`,
            `0 0 20px 10px ${withAlpha(glow, 0)}`,
            `0 0 0 0 ${withAlpha(glow, 0)}`,
          ],
        },
        transition: {
          default: SPRING,
          boxShadow: { duration: 0.45, times: [0, 0.7, 1], ease: "easeOut" },
        },
      };
    case "key-drop":
      return {
        target: { scale: 0.88, y: 4, boxShadow: TRANSPARENT_SHADOW },
        transition: { type: "spring", stiffness: 700, damping: 22 },
      };
    case "border-ripple":
      // The ring itself is a separate element; the key only dips.
      return { target: { scale: 0.96, y: 0, boxShadow: TRANSPARENT_SHADOW }, transition: SPRING };
    case "none":
      return { target: REST, transition: SPRING };
  }
}

/**
 * Live press-duration readout.
 *
 * Mounts when the key goes down and unmounts when it comes up, and updates the
 * text through a ref inside a rAF loop. That keeps a ticking millisecond
 * counter off React's render path entirely — no state, no re-renders.
 */
function PressDuration() {
  const ref = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    const startedAt = performance.now();
    let frame = 0;

    const tick = () => {
      const node = ref.current;
      if (node) {
        node.textContent = `${((performance.now() - startedAt) / 1000).toFixed(2)}s`;
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);

    return () => cancelAnimationFrame(frame);
  }, []);

  return (
    <span ref={ref} className="mt-0.5 text-[10px] opacity-70">
      0.00s
    </span>
  );
}

function KeyElementImpl({ keyData, editMode }: KeyElementProps) {
  const { id, code, style } = keyData;
  const isStick = keyData.shape === "stick";

  // Each key subscribes to its own slice. When one key is pressed every
  // selector re-runs (an O(1) Set lookup) but only the key whose boolean
  // actually flipped re-renders.
  const active = useOverlayStore((s) => s.activeKeys.has(code));
  const pressCount = useOverlayStore((s) => s.pressCounts[code] ?? 0);
  const stickAxes = useOverlayStore((s) => (isStick ? s.stickAxes[code] : undefined));
  const isSelected = useOverlayStore((s) => s.selectedKeyIds.includes(id));
  const isCapturing = useOverlayStore((s) => s.capturingKeyId === id);

  const borderRadius =
    keyData.shape === "circle" || isStick ? "50%" : style.borderRadius;
  const { target, transition } = pressTarget(
    isStick ? "none" : style.pressEffect,
    style.activeGlowColor,
  );
  const stickX = stickAxes?.x ?? 0;
  const stickY = stickAxes?.y ?? 0;
  // Knob travel ≈ 28% of the well so it stays inside the rim.
  const knobTravel = 0.28;

  // A selected key reads the drag/resize offsets that OverlayCanvas writes as
  // CSS custom properties, which is how a whole selection can follow the
  // pointer without a single React render or store write.
  const transform = isSelected
    ? `translate3d(calc(${keyData.x}px + var(--drag-x, 0px) + var(--resize-x, 0px)), calc(${keyData.y}px + var(--drag-y, 0px) + var(--resize-y, 0px)), 0) rotate(${keyData.rotation}deg)`
    : `translate3d(${keyData.x}px, ${keyData.y}px, 0) rotate(${keyData.rotation}deg)`;

  return (
    <div
      data-key-id={id}
      // Position and rotation ride on a single composited transform rather
      // than left/top, so moving a key never triggers layout.
      className={`absolute left-0 top-0 will-change-transform select-none ${
        editMode ? "cursor-grab active:cursor-grabbing" : "pointer-events-none"
      }`}
      style={{
        width: keyData.width * keyData.scale,
        height: keyData.height * keyData.scale,
        transform,
        zIndex: isSelected ? 50 : active ? 40 : 10,
      }}
    >
      <motion.div
        className={`h-full w-full ${
          isCapturing
            ? "ring-2 ring-amber-400"
            : isSelected
              ? "ring-2 ring-cyan-400 ring-offset-1 ring-offset-transparent"
              : ""
        }`}
        style={{ borderRadius }}
        // Only transform and box-shadow animate: both are composited, neither
        // triggers layout.
        animate={active ? target : REST}
        transition={active ? transition : SPRING}
      >
        <div
          className="relative flex h-full w-full flex-col items-center justify-center overflow-hidden border backdrop-blur-sm transition-colors"
          style={{
            backgroundColor: active
              ? pressedFill(style.backgroundColor, style.activeGlowColor)
              : style.backgroundColor,
            borderColor: active ? style.activeGlowColor : style.borderColor,
            borderRadius,
            opacity: style.opacity,
            color: style.textColor,
            fontSize: style.fontSize,
            fontFamily: style.fontFamily,
          }}
        >
          {isStick ? (
            <>
              <span
                className="pointer-events-none absolute inset-[18%] rounded-full border"
                style={{
                  borderColor: withAlpha(style.borderColor, 0.35),
                  backgroundColor: withAlpha(style.backgroundColor, 0.35),
                }}
              />
              <span
                className="pointer-events-none absolute left-1/2 top-1/2 h-[42%] w-[42%] -translate-x-1/2 -translate-y-1/2 rounded-full border shadow-md transition-transform duration-75"
                style={{
                  transform: `translate(calc(-50% + ${stickX * knobTravel * 100}%), calc(-50% + ${stickY * knobTravel * 100}%))`,
                  backgroundColor: active
                    ? withAlpha(style.activeGlowColor, 0.85)
                    : withAlpha(style.activeGlowColor, 0.45),
                  borderColor: style.activeGlowColor,
                  boxShadow: active
                    ? `0 0 14px 2px ${withAlpha(style.activeGlowColor, 0.55)}`
                    : "none",
                }}
              />
              {style.showLabel && (
                <span className="pointer-events-none absolute bottom-1.5 left-0 right-0 text-center font-mono text-[10px] font-bold leading-none opacity-80">
                  {isCapturing ? "…" : keyData.label}
                </span>
              )}
            </>
          ) : (
            <>
              {active && style.pressEffect === "border-ripple" && (
                // Re-keyed per press so a rapid double tap restarts the ring
                // instead of leaving the first one mid-flight.
                <motion.span
                  key={pressCount}
                  className="pointer-events-none absolute inset-0 border-2"
                  style={{ borderColor: style.activeGlowColor, borderRadius }}
                  initial={{ scale: 0.85, opacity: 1 }}
                  animate={{ scale: 2, opacity: 0 }}
                  transition={{ duration: 0.6, ease: "easeOut" }}
                />
              )}
              {style.showLabel && (
                <span className="font-mono font-bold leading-none">
                  {isCapturing ? "…" : keyData.label}
                </span>
              )}
              {style.showPressCount && pressCount > 0 && (
                <span className="mt-0.5 text-[10px] opacity-70">{pressCount}</span>
              )}
              {style.showDuration && active && <PressDuration />}
            </>
          )}
        </div>
      </motion.div>

      {editMode && isSelected && (
        <>
          {RESIZE_HANDLES.map(({ corner, className }) => (
            <span
              key={corner}
              data-resize-handle={corner}
              className={`absolute h-2.5 w-2.5 rounded-full border border-slate-900 bg-cyan-400 ${className}`}
            />
          ))}
        </>
      )}
    </div>
  );
}

/**
 * Memoised so that a re-render of the canvas (adding a key, changing the grid)
 * only re-renders the keys whose config object actually changed.
 */
export const KeyElement = memo(KeyElementImpl);
