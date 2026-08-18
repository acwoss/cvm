import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { HookEditor } from "../components/HookEditor";
import { TrashIcon } from "../components/Icons";
import { commandErrorMessage, deleteHook, listHooks, setHookEnabled } from "../lib/commands";

const FILTERS = [
  "All",
  "post-create",
  "pre-activate",
  "post-activate",
  "pre-deactivate",
  "post-deactivate",
  "pre-remove",
  "post-remove",
] as const;

function eventColorClass(event: string): string {
  return event.startsWith("pre-") ? "text-orange-400 border-orange-500/30 bg-orange-500/10" : "text-cyan-400 border-cyan-500/30 bg-cyan-500/10";
}

export function HooksPage({ onBack: _onBack }: { onBack: () => void }) {
  const queryClient = useQueryClient();
  const [editingEvent, setEditingEvent] = useState<string | null>(null);
  const [filter, setFilter] = useState<(typeof FILTERS)[number]>("All");

  const { data, isPending, error } = useQuery({
    queryKey: ["hooks"],
    queryFn: listHooks,
  });

  const deleteMutation = useMutation({
    mutationFn: (event: string) => deleteHook(event),
    onSuccess: (_data, event) => {
      queryClient.invalidateQueries({ queryKey: ["hooks"] });
      queryClient.invalidateQueries({ queryKey: ["hook-content", event] });
    },
  });

  const toggleMutation = useMutation({
    mutationFn: (vars: { event: string; enabled: boolean }) => setHookEnabled(vars.event, vars.enabled),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["hooks"] }),
  });

  function handleDelete(event: string) {
    if (window.confirm(`Remove the hook '${event}'? This cannot be undone.`)) {
      deleteMutation.mutate(event);
    }
  }

  if (editingEvent) {
    return <HookEditor key={editingEvent} event={editingEvent} onClose={() => setEditingEvent(null)} />;
  }

  const filtered = data?.filter((hook) => filter === "All" || hook.event === filter) ?? [];

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="flex-shrink-0 border-b border-neutral-800 px-7 pb-4 pt-5">
        <div className="mb-3.5 flex items-center justify-between">
          <h1 className="text-lg font-bold tracking-tight text-neutral-100">Hooks</h1>
        </div>
        <div className="flex flex-wrap gap-1">
          {FILTERS.map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={`rounded px-2.5 py-1 font-mono text-[11px] ${
                filter === f
                  ? "border border-orange-500/40 bg-orange-500/10 text-orange-400"
                  : "border border-neutral-800 text-neutral-500 hover:text-neutral-300"
              }`}
            >
              {f}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-7 py-4">
        {isPending && <p className="text-sm text-neutral-400">Loading hooks…</p>}
        {error && <p className="text-sm text-red-400">Error listing hooks: {commandErrorMessage(error)}</p>}
        {deleteMutation.error && (
          <p className="mb-2 text-xs text-red-400">Error removing: {commandErrorMessage(deleteMutation.error)}</p>
        )}
        {toggleMutation.error && (
          <p className="mb-2 text-xs text-red-400">Error updating: {commandErrorMessage(toggleMutation.error)}</p>
        )}

        {data && (
          <>
            <div className="mb-1 grid grid-cols-[1fr_150px_80px_60px] gap-3 px-3.5 py-1.5">
              {["Event / Script", "Status", "", ""].map((h, i) => (
                <span key={i} className="text-[10px] font-semibold uppercase tracking-wider text-neutral-600">
                  {h}
                </span>
              ))}
            </div>
            <div className="flex flex-col gap-1">
              {filtered.map((hook) => (
                <div
                  key={hook.event}
                  onClick={() => setEditingEvent(hook.event)}
                  className={`grid cursor-pointer grid-cols-[1fr_150px_80px_60px] items-center gap-3 rounded-lg border border-neutral-800 bg-neutral-900/60 px-3.5 py-2.5 transition-colors hover:border-neutral-600 hover:bg-neutral-900 ${
                    hook.configured && !hook.enabled ? "opacity-50" : ""
                  }`}
                >
                  <div className="min-w-0">
                    <div className="mb-0.5 font-mono text-xs font-medium text-neutral-100">{hook.event}</div>
                    <div className="truncate font-mono text-[11px] text-neutral-600">
                      {hook.configured ? (hook.preview ?? "(empty script)") : "not configured"}
                    </div>
                  </div>
                  <span
                    className={`w-fit rounded px-1.5 py-0.5 font-mono text-[10px] ${
                      hook.configured
                        ? eventColorClass(hook.event)
                        : "border border-neutral-800 text-neutral-600"
                    }`}
                  >
                    {hook.configured ? (hook.enabled ? "active" : "disabled") : "empty"}
                  </span>
                  {hook.configured ? (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleMutation.mutate({ event: hook.event, enabled: !hook.enabled });
                      }}
                      disabled={toggleMutation.isPending}
                      className="relative h-[18px] w-8 flex-shrink-0 rounded-full transition-colors disabled:opacity-50"
                      style={{ background: hook.enabled ? "rgba(0,229,255,0.18)" : "#282B33" }}
                    >
                      <span
                        className="absolute top-0.5 h-3.5 w-3.5 rounded-full transition-all"
                        style={{
                          left: hook.enabled ? 15 : 2,
                          background: hook.enabled ? "#00E5FF" : "#555B68",
                        }}
                      />
                    </button>
                  ) : (
                    <span />
                  )}
                  {hook.configured ? (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(hook.event);
                      }}
                      disabled={deleteMutation.isPending}
                      className="justify-self-end text-neutral-600 hover:text-red-400 disabled:opacity-50"
                    >
                      <TrashIcon />
                    </button>
                  ) : (
                    <span />
                  )}
                </div>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
