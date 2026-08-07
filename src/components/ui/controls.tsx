import type { ReactNode } from "react";
import { parseColor, toHex, withAlpha, withHex } from "../../lib/color";

export function SectionHeading({ children }: { children: ReactNode }) {
  return (
    <h3 className="mb-2 text-[10px] font-semibold uppercase tracking-[0.12em] text-white/40">
      {children}
    </h3>
  );
}

export function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="block">
      <span className="text-xs text-white/50">{label}</span>
      <div className="mt-1">{children}</div>
    </label>
  );
}

export function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-sm text-white/70">{label}</span>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        onClick={() => onChange(!checked)}
        className={`relative h-6 w-11 shrink-0 rounded-full transition ${
          checked ? "bg-cyan-600" : "bg-white/10"
        }`}
      >
        <span
          className={`absolute top-0.5 h-5 w-5 rounded-full bg-white transition-all ${
            checked ? "left-[22px]" : "left-0.5"
          }`}
        />
      </button>
    </div>
  );
}

export function Slider({
  label,
  value,
  min,
  max,
  step = 1,
  format,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  format?: (value: number) => string;
  onChange: (value: number) => void;
}) {
  return (
    <label className="block">
      <span className="flex items-center justify-between text-xs text-white/50">
        {label}
        <span className="font-mono text-white/70">{format ? format(value) : value}</span>
      </span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="mt-1 w-full accent-cyan-500"
      />
    </label>
  );
}

/**
 * A real swatch picker rather than the raw rgba() text field this replaced.
 *
 * Hue and alpha are edited separately because `<input type="color">` only
 * understands `#rrggbb`, while the stored value keeps its alpha so translucent
 * key backgrounds survive a hue change.
 */
export function ColorField({
  label,
  value,
  onChange,
  withAlphaSlider = true,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  withAlphaSlider?: boolean;
}) {
  const color = parseColor(value);

  return (
    <div className="flex items-center gap-3">
      <span className="w-20 shrink-0 text-xs text-white/50">{label}</span>
      <input
        type="color"
        aria-label={`${label} colour`}
        value={toHex(color)}
        onChange={(e) => onChange(withHex(value, e.target.value))}
        className="h-7 w-9 shrink-0 cursor-pointer rounded border border-white/10 bg-transparent p-0.5"
      />
      {withAlphaSlider && (
        <input
          type="range"
          aria-label={`${label} opacity`}
          min={0}
          max={1}
          step={0.01}
          value={color.a}
          onChange={(e) => onChange(withAlpha(value, Number(e.target.value)))}
          className="w-full accent-cyan-500"
        />
      )}
    </div>
  );
}

export function Segmented<T extends string>({
  options,
  value,
  onChange,
}: {
  options: { value: T; label: string; disabled?: boolean }[];
  value: T;
  onChange: (value: T) => void;
}) {
  return (
    <div className="flex rounded-lg border border-white/10 bg-black/30 p-0.5">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          disabled={option.disabled}
          onClick={() => onChange(option.value)}
          className={`flex-1 rounded-md px-2 py-1.5 text-xs transition disabled:cursor-not-allowed disabled:opacity-40 ${
            value === option.value
              ? "bg-cyan-500/20 text-cyan-300"
              : "text-white/60 hover:text-white"
          }`}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
