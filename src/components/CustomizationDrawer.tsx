import { Layers, Palette, Settings, Sparkles, Wand2, X } from "lucide-react";
import { useEffect, useState } from "react";
import type { DrawerTab, StyleScope } from "../types";
import { useOverlayStore } from "../store/useOverlayStore";
import { Segmented } from "./ui/controls";
import { AnimationsTab } from "./drawer/AnimationsTab";
import { LayoutsTab } from "./drawer/LayoutsTab";
import { SettingsTab } from "./drawer/SettingsTab";
import { ThemesTab } from "./drawer/ThemesTab";
import { VisualsTab } from "./drawer/VisualsTab";

const TABS: { id: DrawerTab; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { id: "visuals", label: "Visuals", icon: Palette },
  { id: "themes", label: "Themes", icon: Sparkles },
  { id: "animations", label: "Motion", icon: Wand2 },
  { id: "layouts", label: "Layouts", icon: Layers },
  { id: "settings", label: "Settings", icon: Settings },
];

const SCOPED_TABS: DrawerTab[] = ["visuals", "animations"];

/**
 * Docked right panel. Unlike a floating overlay it does not cover the canvas,
 * so keys stay reachable while you edit styles.
 */
export function CustomizationDrawer() {
  const tab = useOverlayStore((s) => s.drawerTab);
  const openDrawer = useOverlayStore((s) => s.openDrawer);
  const closeDrawer = useOverlayStore((s) => s.closeDrawer);
  const selectionCount = useOverlayStore((s) => s.selectedKeyIds.length);

  const [scope, setScope] = useState<StyleScope>("all");

  useEffect(() => {
    if (selectionCount > 0) setScope("selection");
  }, [selectionCount]);

  useEffect(() => {
    if (!tab) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") closeDrawer();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [tab, closeDrawer]);

  if (!tab) return null;

  const effectiveScope: StyleScope = selectionCount === 0 ? "all" : scope;

  return (
    <aside className="flex h-full w-[320px] shrink-0 flex-col border-l border-white/10 bg-slate-950/95 backdrop-blur-xl">
      <header className="flex items-center justify-between border-b border-white/10 px-4 py-3">
        <h2 className="font-display text-sm font-bold tracking-wider text-white">CUSTOMIZE</h2>
        <button
          type="button"
          onClick={closeDrawer}
          aria-label="Close customization drawer"
          className="text-white/40 transition hover:text-white"
        >
          <X className="h-4 w-4" />
        </button>
      </header>

      <nav className="flex border-b border-white/10">
        {TABS.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            type="button"
            onClick={() => openDrawer(id)}
            className={`flex flex-1 flex-col items-center gap-1 py-2.5 text-[10px] transition ${
              tab === id
                ? "border-b-2 border-cyan-400 text-cyan-300"
                : "border-b-2 border-transparent text-white/40 hover:text-white/70"
            }`}
          >
            <Icon className="h-3.5 w-3.5" />
            {label}
          </button>
        ))}
      </nav>

      {SCOPED_TABS.includes(tab) && (
        <div className="border-b border-white/10 px-4 py-3">
          <Segmented
            value={effectiveScope}
            onChange={setScope}
            options={[
              {
                value: "selection",
                label: selectionCount > 0 ? `Selected (${selectionCount})` : "Selected",
                disabled: selectionCount === 0,
              },
              { value: "all", label: "All keys" },
            ]}
          />
        </div>
      )}

      <div className="flex-1 overflow-y-auto px-4 py-4">
        {tab === "visuals" && <VisualsTab scope={effectiveScope} />}
        {tab === "themes" && <ThemesTab />}
        {tab === "animations" && <AnimationsTab scope={effectiveScope} />}
        {tab === "layouts" && <LayoutsTab />}
        {tab === "settings" && <SettingsTab />}
      </div>
    </aside>
  );
}
