import type { VisualTheme } from "../../types";
import { DEFAULT_KEY_STYLE } from "../../types";
import { VISUAL_THEMES } from "../../layouts/templates";
import { useOverlayStore } from "../../store/useOverlayStore";
import { SectionHeading } from "../ui/controls";

// "custom" is not a preset you can pick — it is the marker the store sets once
// colours have been hand-edited, so it is shown as status rather than a button.
const PRESETS = (Object.keys(VISUAL_THEMES) as VisualTheme[]).filter((id) => id !== "custom");

export function ThemesTab() {
  const globalTheme = useOverlayStore((s) => s.profile.globalTheme);
  const applyTheme = useOverlayStore((s) => s.applyTheme);

  return (
    <div className="space-y-4">
      <SectionHeading>One-click presets</SectionHeading>
      <div className="grid gap-2">
        {PRESETS.map((id) => {
          const theme = VISUAL_THEMES[id];
          const swatch = { ...DEFAULT_KEY_STYLE, ...theme.globalStyle };
          return (
            <button
              key={id}
              type="button"
              onClick={() => applyTheme(id)}
              className={`flex items-center gap-3 rounded-xl border p-3 text-left transition ${
                globalTheme === id
                  ? "border-cyan-500/50 bg-cyan-950/30"
                  : "border-white/10 bg-white/5 hover:border-white/20 hover:bg-white/10"
              }`}
            >
              <span
                className="h-9 w-9 shrink-0 border"
                style={{
                  backgroundColor: swatch.backgroundColor,
                  borderColor: swatch.borderColor,
                  borderRadius: swatch.borderRadius,
                  boxShadow: `0 0 10px 1px ${swatch.activeGlowColor}`,
                }}
              />
              <span className="min-w-0">
                <span className="block text-sm text-white">{theme.name}</span>
                <span className="block text-[10px] text-white/40">
                  {swatch.pressEffect.replace("-", " ")} on press
                </span>
              </span>
            </button>
          );
        })}
      </div>

      {globalTheme === "custom" && (
        <p className="rounded-lg border border-white/5 bg-black/20 p-3 text-xs text-white/40">
          Colours have been edited by hand, so no preset is active. Picking one above
          overwrites every key&apos;s style.
        </p>
      )}
    </div>
  );
}
