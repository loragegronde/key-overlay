import type { PressEffect, StyleScope } from "../../types";
import { PRESS_EFFECT_LABELS } from "../../types";
import { useOverlayStore } from "../../store/useOverlayStore";
import { SectionHeading } from "../ui/controls";

const EFFECT_DESCRIPTIONS: Record<PressEffect, string> = {
  glow: "Steady halo in the active colour while held",
  "glow-pulse": "Halo expands and fades out on each press",
  "key-drop": "Key dips down like a physical switch",
  "border-ripple": "Ring expands outward from the key edge",
  none: "Colour change only, no motion",
};

const ORDER: PressEffect[] = ["glow-pulse", "key-drop", "border-ripple", "glow", "none"];

export function AnimationsTab({ scope }: { scope: StyleScope }) {
  const keys = useOverlayStore((s) => s.profile.keys);
  const selectedKeyIds = useOverlayStore((s) => s.selectedKeyIds);
  const updateAllKeyStyles = useOverlayStore((s) => s.updateAllKeyStyles);
  const updateSelectedKeyStyles = useOverlayStore((s) => s.updateSelectedKeyStyles);

  const apply = scope === "all" ? updateAllKeyStyles : updateSelectedKeyStyles;
  const reference =
    scope === "selection" ? keys.find((k) => selectedKeyIds.includes(k.id)) : keys[0];

  if (!reference) {
    return (
      <p className="text-sm text-white/40">
        {scope === "selection"
          ? "Select a key on the canvas to change its press animation."
          : "Add a key first."}
      </p>
    );
  }

  const current = reference.style.pressEffect;

  return (
    <div className="space-y-4">
      <SectionHeading>Press animation</SectionHeading>
      <div className="grid gap-2">
        {ORDER.map((effect) => (
          <button
            key={effect}
            type="button"
            onClick={() => apply({ pressEffect: effect })}
            className={`rounded-xl border p-3 text-left transition ${
              current === effect
                ? "border-cyan-500/50 bg-cyan-950/30"
                : "border-white/10 bg-white/5 hover:border-white/20 hover:bg-white/10"
            }`}
          >
            <span className="block text-sm text-white">{PRESS_EFFECT_LABELS[effect]}</span>
            <span className="mt-0.5 block text-[10px] text-white/40">
              {EFFECT_DESCRIPTIONS[effect]}
            </span>
          </button>
        ))}
      </div>
      <p className="text-[10px] text-white/30">
        Press a real key to preview — the overlay reacts to global input even while the
        drawer is open.
      </p>
    </div>
  );
}
