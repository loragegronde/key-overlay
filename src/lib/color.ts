export interface Rgba {
  r: number;
  g: number;
  b: number;
  a: number;
}

const FALLBACK: Rgba = { r: 255, g: 255, b: 255, a: 1 };

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

/**
 * Accepts the CSS colour forms this app can produce: #rgb, #rrggbb, #rrggbbaa,
 * rgb(...) and rgba(...). Anything unparseable falls back to opaque white
 * rather than throwing, because these strings can come from a hand-edited
 * layout.json.
 */
export function parseColor(input: string): Rgba {
  const value = input.trim().toLowerCase();

  if (value.startsWith("#")) {
    const hex = value.slice(1);
    const expand = (c: string) => parseInt(c + c, 16);

    if (hex.length === 3) {
      return { r: expand(hex[0]), g: expand(hex[1]), b: expand(hex[2]), a: 1 };
    }
    if (hex.length === 6 || hex.length === 8) {
      const r = parseInt(hex.slice(0, 2), 16);
      const g = parseInt(hex.slice(2, 4), 16);
      const b = parseInt(hex.slice(4, 6), 16);
      const a = hex.length === 8 ? parseInt(hex.slice(6, 8), 16) / 255 : 1;
      if ([r, g, b].every(Number.isFinite)) return { r, g, b, a };
    }
    return { ...FALLBACK };
  }

  const match = value.match(/^rgba?\(([^)]+)\)$/);
  if (match) {
    const parts = match[1].split(/[,/\s]+/).filter(Boolean).map(Number);
    const [r, g, b, a] = parts;
    if ([r, g, b].every(Number.isFinite)) {
      return {
        r: clamp(Math.round(r), 0, 255),
        g: clamp(Math.round(g), 0, 255),
        b: clamp(Math.round(b), 0, 255),
        a: Number.isFinite(a) ? clamp(a, 0, 1) : 1,
      };
    }
  }

  return { ...FALLBACK };
}

export function toRgbaString({ r, g, b, a }: Rgba): string {
  return `rgba(${r}, ${g}, ${b}, ${Number(a.toFixed(3))})`;
}

/** `#rrggbb`, which is the only form `<input type="color">` understands. */
export function toHex({ r, g, b }: Rgba): string {
  const channel = (c: number) => clamp(Math.round(c), 0, 255).toString(16).padStart(2, "0");
  return `#${channel(r)}${channel(g)}${channel(b)}`;
}

/** Replaces a colour's alpha, preserving its hue. */
export function withAlpha(input: string, alpha: number): string {
  return toRgbaString({ ...parseColor(input), a: clamp(alpha, 0, 1) });
}

/**
 * Solid pressed fill: blends the glow into the key background without dropping
 * alpha (a translucent glow on a transparent overlay window looks invisible).
 */
export function pressedFill(background: string, glow: string, amount = 0.55): string {
  const base = parseColor(background);
  const accent = parseColor(glow);
  const t = clamp(amount, 0, 1);
  return toRgbaString({
    r: Math.round(base.r * (1 - t) + accent.r * t),
    g: Math.round(base.g * (1 - t) + accent.g * t),
    b: Math.round(base.b * (1 - t) + accent.b * t),
    a: Math.max(base.a, 0.9),
  });
}

/** Keeps the alpha from `input` but takes the hue from a `#rrggbb` picker value. */
export function withHex(input: string, hex: string): string {
  const { a } = parseColor(input);
  const { r, g, b } = parseColor(hex);
  return toRgbaString({ r, g, b, a });
}
