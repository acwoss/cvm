import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { commandErrorMessage, openInClaude } from "../lib/commands";

interface Props {
  envName: string;
  onClose: () => void;
}

export function OpenInClaudeDialog({ envName, onClose }: Props) {
  const [directory, setDirectory] = useState("");

  const mutation = useMutation({
    mutationFn: () => openInClaude(envName, directory.trim() || undefined),
    onSuccess: () => {
      onClose();
    },
  });

  async function browse() {
    const selected = await openDialog({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setDirectory(selected);
    }
  }

  return (
    <div className="fixed inset-0 flex items-center justify-center bg-black/60">
      <div className="w-96 rounded-lg border border-neutral-800 bg-neutral-900 p-5">
        <h2 className="mb-1 text-sm font-medium text-neutral-100">Open in Claude</h2>
        <p className="mb-4 text-xs text-neutral-500">
          Choose which directory <span className="font-mono">claude</span> should run in for{" "}
          <span className="text-neutral-300">{envName}</span>.
        </p>
        <div className="mb-4 flex gap-2">
          <input
            autoFocus
            value={directory}
            onChange={(e) => setDirectory(e.target.value)}
            placeholder="/path/to/project (leave empty for default)"
            className="w-full rounded border border-neutral-700 bg-neutral-950 px-3 py-2 text-xs text-neutral-100 outline-none focus:border-neutral-500"
          />
          <button
            onClick={browse}
            className="flex-shrink-0 rounded border border-neutral-700 px-3 py-2 text-xs text-neutral-300 hover:bg-neutral-800"
          >
            Browse…
          </button>
        </div>
        {mutation.error && (
          <p className="mb-3 text-xs text-red-400">Error: {commandErrorMessage(mutation.error)}</p>
        )}
        <div className="flex justify-end gap-2">
          <button onClick={onClose} className="rounded px-3 py-1.5 text-xs text-neutral-400 hover:text-neutral-200">
            Cancel
          </button>
          <button
            onClick={() => mutation.mutate()}
            disabled={mutation.isPending}
            className="rounded bg-orange-500 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-orange-400 disabled:opacity-50"
          >
            {mutation.isPending ? "Opening…" : "Open"}
          </button>
        </div>
      </div>
    </div>
  );
}
