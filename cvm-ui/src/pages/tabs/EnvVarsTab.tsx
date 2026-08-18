import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { SpinnerIcon } from "../../components/Icons";
import type { EnvVarSource, EnvVarSummary } from "../../lib/commands";
import { commandErrorMessage, removeEnvVar, revealEnvVar, writeEnvVar } from "../../lib/commands";

interface Props {
  envName: string;
  envVars: EnvVarSummary[];
}

const SOURCE_LABELS: Record<EnvVarSource, string> = {
  dotenv: ".env",
  settings: "settings.json",
};

export function EnvVarsTab({ envName, envVars }: Props) {
  const queryClient = useQueryClient();
  const [revealed, setRevealed] = useState<Record<string, string>>({});
  const [editing, setEditing] = useState<{ source: EnvVarSource; key: string } | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [formKey, setFormKey] = useState("");
  const [formValue, setFormValue] = useState("");
  const [formSource, setFormSource] = useState<EnvVarSource>("dotenv");
  const [loadingEdit, setLoadingEdit] = useState(false);

  const revealMutation = useMutation({
    mutationFn: ({ source, key }: { source: EnvVarSource; key: string }) =>
      revealEnvVar(envName, source, key),
  });

  const saveMutation = useMutation({
    mutationFn: () => writeEnvVar(envName, formSource, formKey, formValue),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["environment-detail", envName] });
      closeForm();
    },
  });

  const removeMutation = useMutation({
    mutationFn: (v: { source: EnvVarSource; key: string }) => removeEnvVar(envName, v.source, v.key),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["environment-detail", envName] });
    },
  });

  function closeForm() {
    setShowForm(false);
    setEditing(null);
    setFormKey("");
    setFormValue("");
    setFormSource("dotenv");
    setLoadingEdit(false);
  }

  function openAddForm() {
    closeForm();
    setShowForm(true);
  }

  async function openEditForm(v: EnvVarSummary) {
    setEditing({ source: v.source, key: v.key });
    setFormKey(v.key);
    setFormValue("");
    setFormSource(v.source);
    setShowForm(true);
    setLoadingEdit(true);
    try {
      const value = await revealEnvVar(envName, v.source, v.key);
      setFormValue(value);
    } catch {
      // deixa o campo vazio; o usuário ainda pode digitar um valor novo,
      // mas o botão Salvar continua bloqueado até ele fazer isso (ver
      // disabled={... || (editing !== null && loadingEdit)} abaixo) - não
      // sobrescrevemos silenciosamente o valor existente com "" se a
      // releitura falhar.
    } finally {
      setLoadingEdit(false);
    }
  }

  function handleRemove(v: EnvVarSummary) {
    if (window.confirm(`Remove the variable '${v.key}'? This cannot be undone.`)) {
      removeMutation.mutate({ source: v.source, key: v.key });
    }
  }

  async function reveal(source: EnvVarSource, key: string) {
    try {
      const value = await revealMutation.mutateAsync({ source, key });
      setRevealed((prev) => ({ ...prev, [`${source}:${key}`]: value }));
    } catch {
      // error surfaced below via revealMutation.error
    }
  }

  return (
    <div className="p-6">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-medium text-neutral-200">Environment variables</h3>
        <button
          onClick={openAddForm}
          className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white"
        >
          + Add variable
        </button>
      </div>

      <p className="mb-4 rounded border border-amber-900/40 bg-amber-950/30 px-3 py-2 text-xs text-amber-400">
        These variables are loaded into the Claude Code process, but an MCP server only receives one
        of them if it's explicitly referenced in that server's <code>env</code> config block (MCP
        Servers tab). Setting the variable here does not guarantee an MCP server will receive it.
      </p>

      {showForm && (
        <div className="mb-4 rounded border border-neutral-700 bg-neutral-900 p-4">
          <div className="mb-3 grid grid-cols-2 gap-3">
            <div>
              <label className="mb-1 block text-xs text-neutral-500">Key</label>
              <input
                value={formKey}
                onChange={(e) => setFormKey(e.target.value)}
                disabled={editing !== null}
                className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500 disabled:opacity-50"
              />
            </div>
            <div>
              <label className="mb-1 block text-xs text-neutral-500">Value</label>
              <input
                value={formValue}
                onChange={(e) => setFormValue(e.target.value)}
                disabled={loadingEdit}
                placeholder={loadingEdit ? "Loading current value…" : undefined}
                className="w-full rounded border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500 disabled:opacity-50"
              />
            </div>
          </div>
          <div className="mb-3 flex items-center gap-2 text-xs text-neutral-400">
            <span>Source:</span>
            <select
              value={formSource}
              onChange={(e) => setFormSource(e.target.value as EnvVarSource)}
              disabled={editing !== null}
              className="rounded border border-neutral-700 bg-neutral-950 px-2 py-1 text-xs text-neutral-100 disabled:opacity-50"
            >
              <option value="dotenv">.env</option>
              <option value="settings">settings.json</option>
            </select>
          </div>
          {saveMutation.error && (
            <p className="mb-2 text-xs text-red-400">Error: {commandErrorMessage(saveMutation.error)}</p>
          )}
          <div className="flex justify-end gap-2">
            <button
              onClick={closeForm}
              className="rounded px-3 py-1.5 text-xs text-neutral-400 hover:text-neutral-200"
            >
              Cancel
            </button>
            <button
              onClick={() => saveMutation.mutate()}
              disabled={formKey.trim().length === 0 || saveMutation.isPending || loadingEdit}
              className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white disabled:opacity-50"
            >
              {saveMutation.isPending ? "Saving…" : "Save"}
            </button>
          </div>
        </div>
      )}

      {revealMutation.error && (
        <p className="mb-2 text-xs text-red-400">
          Error revealing: {commandErrorMessage(revealMutation.error)}
        </p>
      )}
      {removeMutation.error && (
        <p className="mb-2 text-xs text-red-400">
          Error removing: {commandErrorMessage(removeMutation.error)}
        </p>
      )}

      {envVars.length === 0 ? (
        <p className="text-sm text-neutral-400">No environment variables.</p>
      ) : (
        <ul className="divide-y divide-neutral-800 rounded border border-neutral-800 text-sm">
          {envVars.map((v) => {
            const id = `${v.source}:${v.key}`;
            const isRevealing =
              revealMutation.isPending &&
              revealMutation.variables?.source === v.source &&
              revealMutation.variables?.key === v.key;
            const isRemoving =
              removeMutation.isPending &&
              removeMutation.variables?.source === v.source &&
              removeMutation.variables?.key === v.key;
            const isEditingRow =
              loadingEdit && editing?.source === v.source && editing?.key === v.key;
            return (
              <li key={id} className="flex items-center justify-between px-4 py-3">
                <div>
                  <p className="text-neutral-200">{v.key}</p>
                  {revealed[id] !== undefined && (
                    <p className="font-mono text-xs text-amber-400">{revealed[id]}</p>
                  )}
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-xs text-neutral-500">{SOURCE_LABELS[v.source]}</span>
                  {revealed[id] === undefined && (
                    <button
                      onClick={() => reveal(v.source, v.key)}
                      disabled={isRevealing}
                      className="flex items-center gap-1.5 text-xs text-neutral-400 underline hover:text-neutral-200 disabled:opacity-50"
                    >
                      {isRevealing && <SpinnerIcon size={11} />}
                      reveal
                    </button>
                  )}
                  <button
                    onClick={() => openEditForm(v)}
                    disabled={isEditingRow}
                    className="flex items-center gap-1.5 text-xs text-neutral-400 underline hover:text-neutral-200 disabled:opacity-50"
                  >
                    {isEditingRow && <SpinnerIcon size={11} />}
                    edit
                  </button>
                  <button
                    onClick={() => handleRemove(v)}
                    disabled={isRemoving}
                    className="flex items-center gap-1.5 text-xs text-red-400 underline hover:text-red-300 disabled:opacity-50"
                  >
                    {isRemoving && <SpinnerIcon size={11} />}
                    remove
                  </button>
                </div>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
