import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { commandErrorMessage, getClaudeMd, writeClaudeMd } from "../../lib/commands";

interface Props {
  envName: string;
}

export function ClaudeMdTab({ envName }: Props) {
  const queryClient = useQueryClient();
  const [content, setContent] = useState("");

  const { data, isPending, error } = useQuery({
    queryKey: ["claude-md", envName],
    queryFn: () => getClaudeMd(envName),
  });

  useEffect(() => {
    if (data !== undefined) setContent(data);
  }, [data]);

  const saveMutation = useMutation({
    mutationFn: () => writeClaudeMd(envName, content),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["claude-md", envName] });
    },
  });

  return (
    <div className="flex h-full flex-col p-6 text-sm">
      <div className="mb-2 flex items-center justify-between">
        <div>
          <h3 className="font-medium text-neutral-200">CLAUDE.md</h3>
          <p className="text-xs text-neutral-500">
            Instruções persistentes lidas pelo Claude Code neste ambiente.
          </p>
        </div>
        <div className="flex items-center gap-3">
          {saveMutation.isSuccess && <span className="text-xs text-emerald-400">Saved.</span>}
          {saveMutation.error && (
            <span className="text-xs text-red-400">Error: {commandErrorMessage(saveMutation.error)}</span>
          )}
          <button
            onClick={() => saveMutation.mutate()}
            disabled={isPending || saveMutation.isPending}
            className="rounded border border-orange-500/40 bg-orange-500/10 px-3.5 py-1 text-xs font-semibold text-orange-400 hover:bg-orange-500/20 disabled:opacity-50"
          >
            {saveMutation.isPending ? "Saving…" : "Save"}
          </button>
        </div>
      </div>

      {isPending && <p className="text-sm text-neutral-400">Loading…</p>}
      {error && <p className="text-sm text-red-400">Error loading CLAUDE.md: {commandErrorMessage(error)}</p>}

      {!isPending && !error && (
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          spellCheck={false}
          placeholder="# CLAUDE.md&#10;&#10;Instruções para o Claude Code neste ambiente…"
          className="mt-2 flex-1 resize-none rounded border border-neutral-700 bg-neutral-950 p-3 font-mono text-xs text-neutral-100 outline-none focus:border-neutral-500"
        />
      )}
    </div>
  );
}
