import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
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
    if (window.confirm(`Remover ${kind === "skill" ? "a skill" : "o agent"} '${id}'? Isso não pode ser desfeito.`)) {
      deleteMutation.mutate({ kind, id });
    }
  }

  function renderList(kind: Kind, items: SkillOrAgentInfo[]) {
    if (items.length === 0) {
      return <p className="text-sm text-neutral-500">Nenhum.</p>;
    }
    return (
      <ul className="divide-y divide-neutral-800 text-sm">
        {items.map((item) => (
          <li key={item.id} className="flex items-center justify-between py-2">
            <div>
              <p className="text-neutral-200">{item.name}</p>
              <p className="text-xs text-neutral-500">{item.description}</p>
            </div>
            <div className="flex items-center gap-3 text-xs">
              {item.builtIn ? (
                <span className="text-neutral-500" title="Herdado do ambiente global — edite pelo ambiente de origem">
                  herdado
                </span>
              ) : (
                <>
                  <button
                    onClick={() => onEdit(kind, item.id)}
                    className="text-neutral-400 underline hover:text-neutral-200"
                  >
                    editar
                  </button>
                  <button
                    onClick={() => handleDelete(kind, item.id)}
                    className="text-red-400 underline hover:text-red-300"
                  >
                    remover
                  </button>
                </>
              )}
            </div>
          </li>
        ))}
      </ul>
    );
  }

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-neutral-200">Skills & Agents</h3>
        <button
          onClick={() => setShowNew(true)}
          className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white"
        >
          + Nova skill/agent
        </button>
      </div>

      {showNew && (
        <div className="rounded border border-neutral-700 bg-neutral-900 p-4 text-sm">
          <div className="mb-3 grid grid-cols-2 gap-3">
            <div>
              <label className="mb-1 block text-xs text-neutral-500">Nome</label>
              <input
                value={newId}
                onChange={(e) => setNewId(e.target.value)}
                placeholder="minha-skill"
                className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500"
              />
            </div>
            <div>
              <label className="mb-1 block text-xs text-neutral-500">Tipo</label>
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
            <label className="mb-1 block text-xs text-neutral-500">Descrição</label>
            <input
              value={newDescription}
              onChange={(e) => setNewDescription(e.target.value)}
              placeholder="O que isso faz?"
              className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500"
            />
          </div>
          {createMutation.error && (
            <p className="mb-2 text-xs text-red-400">Erro: {commandErrorMessage(createMutation.error)}</p>
          )}
          <div className="flex justify-end gap-2">
            <button
              onClick={() => setShowNew(false)}
              className="rounded px-3 py-1.5 text-xs text-neutral-400 hover:text-neutral-200"
            >
              Cancelar
            </button>
            <button
              onClick={() => createMutation.mutate()}
              disabled={newId.trim().length === 0 || createMutation.isPending}
              className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white disabled:opacity-50"
            >
              {createMutation.isPending ? "Criando…" : "Criar e editar"}
            </button>
          </div>
        </div>
      )}

      {deleteMutation.error && (
        <p className="text-xs text-red-400">Erro ao remover: {commandErrorMessage(deleteMutation.error)}</p>
      )}

      <section>
        <h4 className="mb-2 text-xs font-medium uppercase tracking-wide text-neutral-500">Skills</h4>
        {renderList("skill", skills)}
      </section>
      <section>
        <h4 className="mb-2 text-xs font-medium uppercase tracking-wide text-neutral-500">Agents</h4>
        {renderList("agent", agents)}
      </section>
    </div>
  );
}
