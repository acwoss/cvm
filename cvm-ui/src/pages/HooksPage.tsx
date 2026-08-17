import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { HookEditor } from "../components/HookEditor";
import { commandErrorMessage, deleteHook, listHooks } from "../lib/commands";

interface Props {
  onBack: () => void;
}

export function HooksPage({ onBack }: Props) {
  const queryClient = useQueryClient();
  const [editingEvent, setEditingEvent] = useState<string | null>(null);

  const { data, isPending, error } = useQuery({
    queryKey: ["hooks"],
    queryFn: listHooks,
  });

  const deleteMutation = useMutation({
    mutationFn: (event: string) => deleteHook(event),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["hooks"] }),
  });

  function handleDelete(event: string) {
    if (window.confirm(`Remover o hook '${event}'? Isso não pode ser desfeito.`)) {
      deleteMutation.mutate(event);
    }
  }

  if (editingEvent) {
    return <HookEditor event={editingEvent} onClose={() => setEditingEvent(null)} />;
  }

  return (
    <div>
      <header className="flex items-center gap-3 border-b border-neutral-800 px-6 py-4">
        <button onClick={onBack} className="text-sm text-neutral-400 hover:text-neutral-200">
          ← Voltar
        </button>
        <h2 className="text-sm font-medium text-neutral-100">Hooks</h2>
      </header>

      {isPending && <p className="p-6 text-sm text-neutral-400">Carregando hooks…</p>}
      {error && <p className="p-6 text-sm text-red-400">Erro ao listar hooks: {commandErrorMessage(error)}</p>}
      {deleteMutation.error && (
        <p className="px-6 pt-3 text-xs text-red-400">Erro ao remover: {commandErrorMessage(deleteMutation.error)}</p>
      )}

      {data && (
        <ul className="divide-y divide-neutral-800">
          {data.map((hook) => (
            <li key={hook.event} className="flex items-center justify-between px-6 py-4">
              <button
                onClick={() => setEditingEvent(hook.event)}
                className="flex-1 text-left"
              >
                <p className="font-mono text-sm text-neutral-100">{hook.event}</p>
                <p className="text-xs text-neutral-500">
                  {hook.configured ? (hook.preview ?? "(script vazio)") : "não configurado"}
                </p>
              </button>
              {hook.configured && (
                <button
                  onClick={() => handleDelete(hook.event)}
                  className="text-xs text-red-400 underline hover:text-red-300"
                >
                  remover
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
