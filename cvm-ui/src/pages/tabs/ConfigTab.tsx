import { useMutation, useQueryClient } from "@tanstack/react-query";
import { commandErrorMessage, removeEnvironment } from "../../lib/commands";
import type { ConfigSection } from "../../lib/commands";

interface Props {
  config: ConfigSection;
  envName: string;
  onRemoved: () => void;
}

export function ConfigTab({ config, envName, onRemoved }: Props) {
  const queryClient = useQueryClient();
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
        <p className="text-neutral-400">{config.allowedTools.join(", ") || "—"}</p>
      </section>
      <section>
        <h3 className="mb-1 font-medium text-neutral-200">Denied tools</h3>
        <p className="text-neutral-400">{config.deniedTools.join(", ") || "—"}</p>
      </section>
      <section>
        <h3 className="mb-1 font-medium text-neutral-200">Outras chaves (settings.json)</h3>
        <pre className="overflow-x-auto rounded bg-neutral-900 p-3 text-xs text-neutral-400">
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
