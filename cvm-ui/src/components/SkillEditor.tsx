import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { marked } from "marked";
import { useEffect, useState } from "react";
import type { SkillContent } from "../lib/commands";
import {
  commandErrorMessage,
  getAgentContent,
  getSkillContent,
  writeAgentContent,
  writeSkillContent,
} from "../lib/commands";

interface Props {
  envName: string;
  kind: "skill" | "agent";
  id: string;
  onClose: () => void;
}

export function SkillEditor({ envName, kind, id, onClose }: Props) {
  const queryClient = useQueryClient();
  const [preview, setPreview] = useState(false);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [body, setBody] = useState("");

  const { data, isPending, error } = useQuery({
    queryKey: [kind === "skill" ? "skill-content" : "agent-content", envName, id],
    queryFn: () => (kind === "skill" ? getSkillContent(envName, id) : getAgentContent(envName, id)),
  });

  useEffect(() => {
    if (data) {
      setName(data.name);
      setDescription(data.description);
      setBody(data.body);
    }
  }, [data]);

  const saveMutation = useMutation({
    mutationFn: () => {
      const content: SkillContent = { name, description, body };
      return kind === "skill"
        ? writeSkillContent(envName, id, content)
        : writeAgentContent(envName, id, content);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["environment-detail", envName] });
    },
  });

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-neutral-950">
      <div className="flex h-12 flex-shrink-0 items-center gap-3 border-b border-neutral-800 px-5">
        <button onClick={onClose} className="text-sm text-neutral-400 hover:text-neutral-200">
          ← Voltar
        </button>
        <div className="h-5 w-px bg-neutral-800" />
        <span className="font-mono text-xs font-semibold text-neutral-100">{id}</span>
        <span className="rounded border border-neutral-800 bg-neutral-900 px-1.5 py-0.5 text-[10px] text-neutral-500">
          {kind}
        </span>
        <div className="flex-1" />
        <button
          onClick={() => setPreview((p) => !p)}
          className={`rounded border px-2.5 py-1 font-mono text-xs ${
            preview
              ? "border-orange-500/40 bg-orange-500/10 text-orange-400"
              : "border-neutral-800 text-neutral-400"
          }`}
        >
          {preview ? "Editar" : "Preview"}
        </button>
        <button
          onClick={() => saveMutation.mutate()}
          disabled={saveMutation.isPending}
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

      {data && (
        <>
          <div className="grid flex-shrink-0 grid-cols-2 gap-4 border-b border-neutral-800 p-4">
            <div>
              <label className="mb-1 block text-xs text-neutral-500">Nome</label>
              <input
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full rounded border border-neutral-700 bg-neutral-900 px-2 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500"
              />
            </div>
            <div>
              <label className="mb-1 block text-xs text-neutral-500">Descrição</label>
              <input
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="w-full rounded border border-neutral-700 bg-neutral-900 px-2 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500"
              />
            </div>
          </div>

          <div className="flex-1 overflow-y-auto">
            {!preview ? (
              <textarea
                value={body}
                onChange={(e) => setBody(e.target.value)}
                spellCheck={false}
                className="h-full w-full resize-none bg-neutral-950 p-6 font-mono text-sm text-neutral-100 outline-none"
              />
            ) : (
              <div
                className="markdown-preview mx-auto max-w-3xl p-6 text-sm text-neutral-200"
                dangerouslySetInnerHTML={{ __html: marked.parse(body) as string }}
              />
            )}
          </div>
        </>
      )}
    </div>
  );
}
