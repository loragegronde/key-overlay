import { Copy, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import type { LayoutTemplateId } from "../../types";
import { LAYOUT_TEMPLATES } from "../../layouts/templates";
import { useOverlayStore } from "../../store/useOverlayStore";
import { SectionHeading } from "../ui/controls";

const TEMPLATE_IDS = Object.keys(LAYOUT_TEMPLATES) as LayoutTemplateId[];

export function LayoutsTab() {
  const library = useOverlayStore((s) => s.library);
  const activeId = useOverlayStore((s) => s.profile.id);
  const currentTemplate = useOverlayStore((s) => s.profile.templateId);
  const createLayout = useOverlayStore((s) => s.createLayout);
  const duplicateLayout = useOverlayStore((s) => s.duplicateLayout);
  const deleteLayout = useOverlayStore((s) => s.deleteLayout);
  const switchLayout = useOverlayStore((s) => s.switchLayout);
  const renameLayout = useOverlayStore((s) => s.renameLayout);
  const loadTemplate = useOverlayStore((s) => s.loadTemplate);

  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [draftName, setDraftName] = useState("");

  const commitRename = (id: string) => {
    renameLayout(id, draftName);
    setRenamingId(null);
  };

  return (
    <div className="space-y-6">
      <section className="space-y-3">
        <div className="flex items-center justify-between gap-2">
          <SectionHeading>Saved layouts</SectionHeading>
          <button
            type="button"
            onClick={() => createLayout()}
            className="flex items-center gap-1 rounded-lg border border-cyan-500/40 bg-cyan-950/40 px-2 py-1 text-[11px] text-cyan-300 transition hover:bg-cyan-950/70"
          >
            <Plus className="h-3 w-3" />
            New
          </button>
        </div>

        <div className="grid gap-2">
          {library.profiles.map((entry) => {
            const active = entry.id === activeId;
            const renaming = renamingId === entry.id;
            return (
              <div
                key={entry.id}
                className={`rounded-xl border p-3 transition ${
                  active
                    ? "border-cyan-500/50 bg-cyan-950/30"
                    : "border-white/10 bg-white/5 hover:border-white/20"
                }`}
              >
                <div className="flex items-start gap-2">
                  <button
                    type="button"
                    onClick={() => switchLayout(entry.id)}
                    className="min-w-0 flex-1 text-left"
                  >
                    {renaming ? (
                      <input
                        autoFocus
                        value={draftName}
                        onChange={(e) => setDraftName(e.target.value)}
                        onBlur={() => commitRename(entry.id)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") commitRename(entry.id);
                          if (e.key === "Escape") setRenamingId(null);
                          e.stopPropagation();
                        }}
                        onClick={(e) => e.stopPropagation()}
                        className="w-full rounded-md border border-white/20 bg-black/40 px-2 py-1 text-sm text-white outline-none focus:border-cyan-500/50"
                      />
                    ) : (
                      <span
                        className="block truncate text-sm text-white"
                        onDoubleClick={(e) => {
                          e.stopPropagation();
                          setRenamingId(entry.id);
                          setDraftName(entry.name);
                        }}
                      >
                        {entry.name}
                      </span>
                    )}
                    <span className="mt-0.5 block font-mono text-[10px] text-white/35">
                      {entry.keys.length} keys
                      {active ? " · active" : ""}
                    </span>
                  </button>

                  <button
                    type="button"
                    title="Duplicate"
                    aria-label="Duplicate layout"
                    onClick={() => duplicateLayout(entry.id)}
                    className="rounded-md p-1.5 text-white/50 transition hover:bg-white/10 hover:text-white"
                  >
                    <Copy className="h-3.5 w-3.5" />
                  </button>
                  <button
                    type="button"
                    title="Delete"
                    aria-label="Delete layout"
                    disabled={library.profiles.length <= 1}
                    onClick={() => {
                      if (library.profiles.length <= 1) return;
                      if (window.confirm(`Delete “${entry.name}”?`)) {
                        deleteLayout(entry.id);
                      }
                    }}
                    className="rounded-md p-1.5 text-white/50 transition hover:bg-red-950/50 hover:text-red-300 disabled:opacity-30"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </div>
                {!renaming && (
                  <button
                    type="button"
                    onClick={() => {
                      setRenamingId(entry.id);
                      setDraftName(entry.name);
                    }}
                    className="mt-2 text-[10px] text-white/35 underline-offset-2 hover:text-white/60 hover:underline"
                  >
                    Rename
                  </button>
                )}
              </div>
            );
          })}
        </div>
        <p className="text-[10px] text-white/30">
          Layouts stay saved on disk. Double-click a name to rename, or use Duplicate to copy
          one before experimenting.
        </p>
      </section>

      <section className="space-y-3">
        <SectionHeading>Presets</SectionHeading>
        <div className="grid gap-2">
          {TEMPLATE_IDS.map((id) => {
            const template = LAYOUT_TEMPLATES[id];
            return (
              <button
                key={id}
                type="button"
                onClick={() => loadTemplate(id)}
                className={`rounded-xl border p-3 text-left transition ${
                  currentTemplate === id
                    ? "border-cyan-500/50 bg-cyan-950/30"
                    : "border-white/10 bg-white/5 hover:border-white/20 hover:bg-white/10"
                }`}
              >
                <span className="flex items-baseline justify-between gap-2">
                  <span className="text-sm text-white">{template.name}</span>
                  <span className="shrink-0 font-mono text-[10px] text-white/30">
                    {template.keys.length} keys
                  </span>
                </span>
                <span className="mt-0.5 block text-[10px] text-white/40">
                  {template.description}
                </span>
              </button>
            );
          })}
        </div>
        <p className="text-[10px] text-white/30">
          Loading a preset replaces every key on the active layout. Theme, opacity and grid
          settings are kept.
        </p>
      </section>
    </div>
  );
}
