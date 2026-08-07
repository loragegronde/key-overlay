import { Crosshair } from "lucide-react";
import type { KeyShape, StyleScope } from "../../types";
import { useOverlayStore } from "../../store/useOverlayStore";
import { ColorField, Field, SectionHeading, Slider, Toggle } from "../ui/controls";

export function VisualsTab({ scope }: { scope: StyleScope }) {
  const keys = useOverlayStore((s) => s.profile.keys);
  const selectedKeyIds = useOverlayStore((s) => s.selectedKeyIds);
  const capturingKeyId = useOverlayStore((s) => s.capturingKeyId);
  const updateAllKeyStyles = useOverlayStore((s) => s.updateAllKeyStyles);
  const updateSelectedKeyStyles = useOverlayStore((s) => s.updateSelectedKeyStyles);
  const updateSelectedKeys = useOverlayStore((s) => s.updateSelectedKeys);
  const updateKey = useOverlayStore((s) => s.updateKey);
  const startCapture = useOverlayStore((s) => s.startCapture);
  const cancelCapture = useOverlayStore((s) => s.cancelCapture);

  const apply = scope === "all" ? updateAllKeyStyles : updateSelectedKeyStyles;

  // The controls need one key's values to show as their current state. For a
  // multi-key edit that is the first one in the group.
  const reference =
    scope === "selection" ? keys.find((k) => selectedKeyIds.includes(k.id)) : keys[0];

  if (!reference) {
    return (
      <p className="text-sm text-white/40">
        {scope === "selection"
          ? "Select a key on the canvas to edit it."
          : "Add a key before editing global styles."}
      </p>
    );
  }

  const style = reference.style;
  const single = scope === "selection" && selectedKeyIds.length === 1;

  return (
    <div className="space-y-6">
      {single && (
        <section className="space-y-3">
          <SectionHeading>Key</SectionHeading>
          <Field label="Display name">
            <input
              type="text"
              value={reference.label}
              onChange={(e) => updateKey(reference.id, { label: e.target.value })}
              placeholder="Label shown on the key"
              className="w-full rounded-lg border border-white/10 bg-black/40 px-3 py-2 text-sm text-white outline-none focus:border-cyan-500/50"
            />
          </Field>
          <Field label="Binding">
            <div className="flex items-center gap-2 rounded-lg border border-white/10 bg-black/30 p-2">
              <code className="flex-1 truncate font-mono text-xs text-white/70">
                {reference.code}
              </code>
              <button
                type="button"
                onClick={() =>
                  capturingKeyId === reference.id ? cancelCapture() : startCapture(reference.id)
                }
                className={`flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs transition ${
                  capturingKeyId === reference.id
                    ? "bg-amber-500/20 text-amber-300"
                    : "bg-white/5 text-white/70 hover:bg-white/10"
                }`}
              >
                <Crosshair className="h-3 w-3" />
                {capturingKeyId === reference.id ? "Press…" : "Rebind"}
              </button>
            </div>
          </Field>
          <p className="text-[10px] leading-snug text-white/35">
            The display name is what you see on the key. Rebind changes which physical
            key/button lights it up — your custom name is kept.
          </p>
        </section>
      )}

      <section className="space-y-3">
        <SectionHeading>Colours</SectionHeading>
        <ColorField
          label="Background"
          value={style.backgroundColor}
          onChange={(v) => apply({ backgroundColor: v })}
        />
        <ColorField
          label="Border"
          value={style.borderColor}
          onChange={(v) => apply({ borderColor: v })}
        />
        <ColorField
          label="Active glow"
          value={style.activeGlowColor}
          onChange={(v) => apply({ activeGlowColor: v })}
        />
        <ColorField
          label="Text"
          value={style.textColor}
          onChange={(v) => apply({ textColor: v })}
          withAlphaSlider={false}
        />
      </section>

      <section className="space-y-3">
        <SectionHeading>Shape</SectionHeading>
        {scope === "selection" && (
          <Field label="Outline">
            <select
              value={reference.shape}
              onChange={(e) => updateSelectedKeys({ shape: e.target.value as KeyShape })}
              className="w-full rounded-lg border border-white/10 bg-black/40 px-3 py-2 text-sm text-white"
            >
              <option value="rectangle">Rectangle</option>
              <option value="circle">Circle</option>
              <option value="stick">Joystick</option>
            </select>
          </Field>
        )}
        <Slider
          label="Border radius"
          value={style.borderRadius}
          min={0}
          max={32}
          onChange={(v) => apply({ borderRadius: v })}
          format={(v) => `${v}px`}
        />
        <Slider
          label="Opacity"
          value={style.opacity}
          min={0.1}
          max={1}
          step={0.05}
          onChange={(v) => apply({ opacity: v })}
          format={(v) => `${Math.round(v * 100)}%`}
        />
        <Slider
          label="Font size"
          value={style.fontSize}
          min={8}
          max={32}
          onChange={(v) => apply({ fontSize: v })}
          format={(v) => `${v}px`}
        />
      </section>

      <section className="space-y-3">
        <SectionHeading>Readouts</SectionHeading>
        <Toggle
          label="Key label"
          checked={style.showLabel}
          onChange={(v) => apply({ showLabel: v })}
        />
        <Toggle
          label="Press count"
          checked={style.showPressCount}
          onChange={(v) => apply({ showPressCount: v })}
        />
        <Toggle
          label="Hold duration"
          checked={style.showDuration}
          onChange={(v) => apply({ showDuration: v })}
        />
      </section>
    </div>
  );
}
