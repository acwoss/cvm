import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { commandErrorMessage, getHook, writeHook } from "../lib/commands";

const DEFAULT_TEMPLATE_PREFIX = "#!/bin/sh\n\n# hook: ";

interface Props {
  event: string;
  onClose: () => void;
}

export function HookEditor({ event, onClose }: Props) {
  const queryClient = useQueryClient();
  const [content, setContent] = useState("");

  const { data, isPending, error } = useQuery({
    queryKey: ["hook-content", event],
    queryFn: () => getHook(event),
  });

  useEffect(() => {
    if (data !== undefined) {
      setContent(data ?? `${DEFAULT_TEMPLATE_PREFIX}${event}\n`);
    }
  }, [data, event]);

  const saveMutation = useMutation({
    mutationFn: () => writeHook(event, content),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["hooks"] });
      queryClient.invalidateQueries({ queryKey: ["hook-content", event] });
    },
  });

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-neutral-950">
      <div className="flex h-12 flex-shrink-0 items-center gap-3 border-b border-neutral-800 px-5">
        <button onClick={onClose} className="text-sm text-neutral-400 hover:text-neutral-200">
          ← Voltar
        </button>
        <div className="h-5 w-px bg-neutral-800" />
        <span className="font-mono text-xs font-semibold text-neutral-100">{event}</span>
        <div className="flex-1" />
        <button
          onClick={() => saveMutation.mutate()}
          disabled={data === undefined || saveMutation.isPending}
          className="rounded border border-orange-500/40 bg-orange-500/10 px-3.5 py-1 text-xs font-semibold text-orange-400 disabled:opacity-50"
        >
          {saveMutation.isPending ? "Salvando…" : "Salvar"}
        </button>
      </div>

      {isPending && <p className="p-6 text-sm text-neutral-400">Carregando…</p>}
      {error && <p className="p-6 text-sm text-red-400">Erro ao carregar: {commandErrorMessage(error)}</p>}
      {saveMutation.error && (
        <p className="border-b border-red-900/40 bg-red-950/20 px-5 py-2 text-xs text-red-400">
          Erro ao salvar: {commandErrorMessage(saveMutation.error)}
        </p>
      )}
      {saveMutation.isSuccess && (
        <p className="border-b border-emerald-900/40 bg-emerald-950/20 px-5 py-2 text-xs text-emerald-400">
          Salvo.
        </p>
      )}

      {data !== undefined && (
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          spellCheck={false}
          className="w-full flex-1 resize-none bg-neutral-950 p-6 font-mono text-sm text-neutral-100 outline-none"
        />
      )}
    </div>
  );
}
