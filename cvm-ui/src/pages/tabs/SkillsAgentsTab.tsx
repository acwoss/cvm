import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { BotIcon, FileCodeIcon, SpinnerIcon, TrashIcon, ZapIcon } from "../../components/Icons";
import { PluginVisibilityToggle } from "../../components/PluginVisibilityToggle";
import type { SkillOrAgentInfo } from "../../lib/commands";
import {
  commandErrorMessage,
  createAgent,
  createSkill,
  deleteAgent,
  deleteSkill,
} from "../../lib/commands";

type Kind = "skill" | "agent";

interface Props {
  envName: string;
  skills: SkillOrAgentInfo[];
  agents: SkillOrAgentInfo[];
  onEdit: (kind: Kind, id: string) => void;
}

export function SkillsAgentsTab({ envName, skills, agents, onEdit }: Props) {
  const queryClient = useQueryClient();
  const [showNew, setShowNew] = useState(false);
  const [showPlugins, setShowPlugins] = useState(true);
  const [newKind, setNewKind] = useState<Kind>("skill");
  const [newId, setNewId] = useState("");
  const [newDescription, setNewDescription] = useState("");

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["environment-detail", envName] });
  }

  const createMutation = useMutation({
    mutationFn: () => {
      const id = newId.trim();
      return newKind === "skill"
        ? createSkill(envName, id, id, newDescription)
        : createAgent(envName, id, id, newDescription);
    },
    onSuccess: () => {
      invalidate();
      const createdKind = newKind;
      const createdId = newId.trim();
      setShowNew(false);
      setNewId("");
      setNewDescription("");
      onEdit(createdKind, createdId);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (v: { kind: Kind; id: string }) =>
      v.kind === "skill" ? deleteSkill(envName, v.id) : deleteAgent(envName, v.id),
    onSuccess: invalidate,
  });

  function handleDelete(kind: Kind, id: string) {
    if (window.confirm(`Remove ${kind === "skill" ? "the skill" : "the agent"} '${id}'? This cannot be undone.`)) {
      deleteMutation.mutate({ kind, id });
    }
  }

  const combined = [
    ...skills.map((item) => ({ kind: "skill" as Kind, item })),
    ...agents.map((item) => ({ kind: "agent" as Kind, item })),
  ].filter(({ item }) => showPlugins || item.source.kind === "native");

  function renderList() {
    if (combined.length === 0) {
      return <p className="text-sm text-neutral-500">None.</p>;
    }
    return (
      <div className="flex flex-col gap-2">
        {combined.map(({ kind, item }) => {
          const isDeleting =
            deleteMutation.isPending &&
            deleteMutation.variables?.kind === kind &&
            deleteMutation.variables?.id === item.id;
          const originKey = item.source.kind === "native" ? "native" : `${item.source.marketplace}/${item.source.plugin}`;
          return (
          <div
            key={`${kind}-${originKey}-${item.id}`}
            className="flex items-center justify-between gap-3 rounded-lg border border-neutral-800 bg-neutral-900/60 px-3.5 py-2.5"
          >
            <div className="flex min-w-0 items-center gap-3">
              <span
                className={`flex h-7 w-7 flex-shrink-0 items-center justify-center rounded ${
                  kind === "skill" ? "bg-orange-500/10 text-orange-400" : "bg-neutral-800 text-neutral-400"
                }`}
              >
                {kind === "skill" ? <ZapIcon /> : <BotIcon />}
              </span>
              <div className="min-w-0">
                <div className="flex items-center gap-1.5">
                  <span className="truncate text-sm font-medium text-neutral-100">{item.name}</span>
                  <span className="rounded border border-neutral-800 px-1.5 py-0.5 font-mono text-[10px] text-neutral-500">
                    {kind}
                  </span>
                  {item.builtIn && (
                    <span
                      className="rounded border border-neutral-800 px-1.5 py-0.5 font-mono text-[10px] text-neutral-600"
                      title="Inherited from the global environment — edit it from the source environment"
                    >
                      built-in
                    </span>
                  )}
                  {item.source.kind === "plugin" && (
                    <span
                      className="rounded border border-neutral-800 px-1.5 py-0.5 font-mono text-[10px] text-neutral-600"
                      title={`Provided by the plugin '${item.source.plugin}' (${item.source.marketplace})`}
                    >
                      plugin: {item.source.plugin}
                    </span>
                  )}
                </div>
                <p className="truncate text-xs text-neutral-500">{item.description}</p>
              </div>
            </div>
            {!item.builtIn && item.source.kind !== "plugin" && (
              <div className="flex flex-shrink-0 items-center gap-3">
                <button
                  onClick={() => onEdit(kind, item.id)}
                  className="flex items-center gap-1 rounded border border-neutral-700 px-2 py-1 text-xs text-neutral-300 hover:border-neutral-500 hover:text-neutral-100"
                >
                  <FileCodeIcon /> Edit
                </button>
                <button
                  onClick={() => handleDelete(kind, item.id)}
                  disabled={isDeleting}
                  className="text-neutral-600 hover:text-red-400 disabled:opacity-50"
                >
                  {isDeleting ? <SpinnerIcon size={13} /> : <TrashIcon />}
                </button>
              </div>
            )}
          </div>
          );
        })}
      </div>
    );
  }

  return (
    <div className="space-y-4 p-7">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-neutral-500">
          {combined.length} skills & agents
        </h3>
        <div className="flex items-center gap-4">
          <PluginVisibilityToggle checked={showPlugins} onChange={setShowPlugins} />
          <button
            onClick={() => setShowNew(true)}
            className="rounded border border-orange-500/40 bg-orange-500/10 px-3 py-1.5 text-xs font-semibold text-orange-400 hover:bg-orange-500/20"
          >
            + New skill/agent
          </button>
        </div>
      </div>

      {showNew && (
        <div className="rounded border border-neutral-700 bg-neutral-900 p-4 text-sm">
          <div className="mb-3 grid grid-cols-2 gap-3">
            <div>
              <label className="mb-1 block text-xs text-neutral-500">Name</label>
              <input
                value={newId}
                onChange={(e) => setNewId(e.target.value)}
                placeholder="my-skill"
                className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500"
              />
            </div>
            <div>
              <label className="mb-1 block text-xs text-neutral-500">Type</label>
              <div className="flex gap-2">
                {(["skill", "agent"] as const).map((k) => (
                  <button
                    key={k}
                    onClick={() => setNewKind(k)}
                    className={`flex-1 rounded border px-2 py-1.5 text-xs ${
                      newKind === k
                        ? "border-orange-500/40 bg-orange-500/10 text-orange-400"
                        : "border-neutral-700 bg-neutral-950 text-neutral-400"
                    }`}
                  >
                    {k}
                  </button>
                ))}
              </div>
            </div>
          </div>
          <div className="mb-3">
            <label className="mb-1 block text-xs text-neutral-500">Description</label>
            <input
              value={newDescription}
              onChange={(e) => setNewDescription(e.target.value)}
              placeholder="What does this do?"
              className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500"
            />
          </div>
          {createMutation.error && (
            <p className="mb-2 text-xs text-red-400">Error: {commandErrorMessage(createMutation.error)}</p>
          )}
          <div className="flex justify-end gap-2">
            <button
              onClick={() => setShowNew(false)}
              className="rounded px-3 py-1.5 text-xs text-neutral-400 hover:text-neutral-200"
            >
              Cancel
            </button>
            <button
              onClick={() => createMutation.mutate()}
              disabled={newId.trim().length === 0 || createMutation.isPending}
              className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white disabled:opacity-50"
            >
              {createMutation.isPending ? "Creating…" : "Create and edit"}
            </button>
          </div>
        </div>
      )}

      {deleteMutation.error && (
        <p className="text-xs text-red-400">Error removing: {commandErrorMessage(deleteMutation.error)}</p>
      )}

      {renderList()}
    </div>
  );
}
