import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { commandErrorMessage, createEnvironment } from "../lib/commands";

interface Props {
  onClose: () => void;
}

export function CreateEnvironmentDialog({ onClose }: Props) {
  const [name, setName] = useState("");
  const [anonymous, setAnonymous] = useState(false);
  const [inherit, setInherit] = useState(false);
  const [open, setOpen] = useState(false);
  const queryClient = useQueryClient();

  const mutation = useMutation({
    mutationFn: () => createEnvironment(name, anonymous, inherit, open),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["environments"] });
      onClose();
    },
  });

  return (
    <div className="fixed inset-0 flex items-center justify-center bg-black/60">
      <div className="w-80 rounded-lg border border-neutral-800 bg-neutral-900 p-5">
        <h2 className="mb-4 text-sm font-medium text-neutral-100">Novo ambiente</h2>
        <input
          autoFocus
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="nome-do-ambiente"
          className="mb-4 w-full rounded border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm text-neutral-100 outline-none focus:border-neutral-500"
        />
        <label className="mb-2 flex items-center gap-2 text-xs text-neutral-300">
          <input type="checkbox" checked={anonymous} onChange={(e) => setAnonymous(e.target.checked)} />
          Anônimo (não copiar credenciais globais)
        </label>
        <label className="mb-2 flex items-center gap-2 text-xs text-neutral-300">
          <input type="checkbox" checked={inherit} onChange={(e) => setInherit(e.target.checked)} />
          Herdar configurações globais
        </label>
        <label className="mb-4 flex items-center gap-2 text-xs text-neutral-300">
          <input type="checkbox" checked={open} onChange={(e) => setOpen(e.target.checked)} />
          Abrir no Claude após criar
        </label>
        {mutation.error && (
          <p className="mb-3 text-xs text-red-400">Erro: {commandErrorMessage(mutation.error)}</p>
        )}
        <div className="flex justify-end gap-2">
          <button onClick={onClose} className="rounded px-3 py-1.5 text-xs text-neutral-400 hover:text-neutral-200">
            Cancelar
          </button>
          <button
            onClick={() => mutation.mutate()}
            disabled={name.trim().length === 0 || mutation.isPending}
            className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white disabled:opacity-50"
          >
            {mutation.isPending ? "Criando…" : "Criar"}
          </button>
        </div>
      </div>
    </div>
  );
}
