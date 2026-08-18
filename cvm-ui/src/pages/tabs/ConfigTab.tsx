import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { commandErrorMessage, removeEnvironment, writeConfigSection } from "../../lib/commands";
import type { ConfigSection } from "../../lib/commands";

interface Props {
  config: ConfigSection;
  envName: string;
  onRemoved: () => void;
}

export function ConfigTab({ config, envName, onRemoved }: Props) {
  const queryClient = useQueryClient();
  const [allowedTools, setAllowedTools] = useState(config.allowedTools.join("\n"));
  const [deniedTools, setDeniedTools] = useState(config.deniedTools.join("\n"));

  const saveMutation = useMutation({
    mutationFn: () =>
      writeConfigSection(
        envName,
        allowedTools
          .split("\n")
          .map((t) => t.trim())
          .filter(Boolean),
        deniedTools
          .split("\n")
          .map((t) => t.trim())
          .filter(Boolean),
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["environment-detail", envName] });
    },
  });

  const removeMutation = useMutation({
    mutationFn: () => removeEnvironment(envName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["environments"] });
      onRemoved();
    },
  });

  function handleRemove() {
    if (window.confirm(`Remover o ambiente '${envName}'? Isso não pode ser desfeito.`)) {
      removeMutation.mutate();
    }
  }

  return (
    <div className="space-y-4 p-6 text-sm">
      <section>
        <h3 className="mb-1 font-medium text-neutral-200">Allowed tools</h3>
        <p className="mb-2 text-xs text-neutral-500">Uma ferramenta por linha.</p>
        <textarea
          value={allowedTools}
          onChange={(e) => setAllowedTools(e.target.value)}
          rows={4}
          className="w-full rounded border border-neutral-700 bg-neutral-950 px-3 py-2 font-mono text-xs text-neutral-100 outline-none focus:border-neutral-500"
        />
      </section>
      <section>
        <h3 className="mb-1 font-medium text-neutral-200">Denied tools</h3>
        <p className="mb-2 text-xs text-neutral-500">Uma ferramenta por linha.</p>
        <textarea
          value={deniedTools}
          onChange={(e) => setDeniedTools(e.target.value)}
          rows={4}
          className="w-full rounded border border-neutral-700 bg-neutral-950 px-3 py-2 font-mono text-xs text-neutral-100 outline-none focus:border-neutral-500"
        />
      </section>
      <div className="flex items-center gap-3">
        <button
          onClick={() => saveMutation.mutate()}
          disabled={saveMutation.isPending}
          className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white disabled:opacity-50"
        >
          {saveMutation.isPending ? "Salvando…" : "Salvar"}
        </button>
        {saveMutation.isSuccess && <span className="text-xs text-emerald-400">Salvo.</span>}
        {saveMutation.error && (
          <span className="text-xs text-red-400">Erro: {commandErrorMessage(saveMutation.error)}</span>
        )}
      </div>
      <section>
        <h3 className="mb-1 font-medium text-neutral-200">Outras chaves (settings.json)</h3>
        <pre className="overflow-x-auto whitespace-pre-wrap break-words rounded bg-neutral-900 p-3 text-xs text-neutral-400">
          {JSON.stringify(config.other, null, 2)}
        </pre>
      </section>
      <section className="rounded border border-red-900/40 bg-red-950/10 p-4">
        <h3 className="mb-1 font-medium text-red-400">Zona de risco</h3>
        <p className="mb-3 text-xs text-neutral-500">
          Remove permanentemente este ambiente e todos os seus dados.
        </p>
        <button
          onClick={handleRemove}
          disabled={removeMutation.isPending}
          className="rounded border border-red-900/50 bg-red-950/30 px-3 py-1.5 text-xs font-medium text-red-400 hover:bg-red-950/50 disabled:opacity-50"
        >
          {removeMutation.isPending ? "Removendo…" : "Remover ambiente"}
        </button>
        {removeMutation.error && (
          <p className="mt-2 text-xs text-red-400">Erro: {commandErrorMessage(removeMutation.error)}</p>
        )}
      </section>
    </div>
  );
}
