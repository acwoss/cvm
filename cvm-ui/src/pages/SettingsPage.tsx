import { getVersion } from "@tauri-apps/api/app";
import { useMutation, useQuery } from "@tanstack/react-query";
import { applyUiUpdate, checkUiUpdate, commandErrorMessage } from "../lib/commands";

export function SettingsPage({ onBack: _onBack }: { onBack: () => void }) {
  const { data: version } = useQuery({
    queryKey: ["app-version"],
    queryFn: getVersion,
  });

  const checkMutation = useMutation({
    mutationFn: checkUiUpdate,
  });

  const updateMutation = useMutation({
    mutationFn: applyUiUpdate,
  });

  return (
    <div className="p-7">
      <h1 className="mb-5 text-lg font-bold tracking-tight text-neutral-100">Settings</h1>

      <div className="max-w-xl divide-y divide-neutral-800 rounded-lg border border-neutral-800">
        <div className="flex items-center justify-between px-4 py-3.5">
          <div>
            <p className="text-sm text-neutral-100">cvm-ui</p>
            <p className="text-xs text-neutral-500">App version</p>
          </div>
          <p className="font-mono text-xs text-neutral-400">{version ?? "…"}</p>
        </div>
        <div className="flex items-center justify-between px-4 py-3.5">
          <div>
            <p className="text-sm text-neutral-100">Environments directory</p>
            <p className="text-xs text-neutral-500">Where each cvm environment lives on disk</p>
          </div>
          <p className="font-mono text-xs text-neutral-400">~/.cvm/envs/</p>
        </div>
        <div className="flex items-center justify-between px-4 py-3.5">
          <div>
            <p className="text-sm text-neutral-100">Global hooks</p>
            <p className="text-xs text-neutral-500">Managed in the "Hooks" sidebar tab</p>
          </div>
          <p className="font-mono text-xs text-neutral-400">~/.cvm/hooks/</p>
        </div>
        <div className="px-4 py-3.5">
          <div className="mb-2 flex items-center justify-between">
            <div>
              <p className="text-sm text-neutral-100">Updates</p>
              <p className="text-xs text-neutral-500">Check GitHub Releases for a newer cvm-ui</p>
            </div>
            <button
              onClick={() => checkMutation.mutate()}
              disabled={checkMutation.isPending}
              className="rounded border border-neutral-700 px-3 py-1.5 text-xs text-neutral-300 hover:border-neutral-500 hover:text-neutral-100 disabled:opacity-50"
            >
              {checkMutation.isPending ? "Checking…" : "Check for updates"}
            </button>
          </div>

          {checkMutation.error && (
            <p className="text-xs text-red-400">Error: {commandErrorMessage(checkMutation.error)}</p>
          )}

          {checkMutation.isSuccess && checkMutation.data === null && (
            <p className="text-xs text-emerald-400">You're up to date.</p>
          )}

          {checkMutation.isSuccess && checkMutation.data && !updateMutation.isSuccess && (
            <div className="flex items-center justify-between rounded border border-orange-500/30 bg-orange-500/10 px-3 py-2">
              <p className="text-xs text-orange-400">
                Version {checkMutation.data.latest} is available (current: {checkMutation.data.current})
              </p>
              <button
                onClick={() => updateMutation.mutate()}
                disabled={updateMutation.isPending}
                className="rounded border border-orange-500/40 bg-orange-500/10 px-3 py-1 text-xs font-semibold text-orange-400 hover:bg-orange-500/20 disabled:opacity-50"
              >
                {updateMutation.isPending ? "Updating…" : "Update now"}
              </button>
            </div>
          )}

          {updateMutation.error && (
            <p className="mt-2 text-xs text-red-400">Error: {commandErrorMessage(updateMutation.error)}</p>
          )}

          {updateMutation.isSuccess && (
            <p className="text-xs text-emerald-400">Updated — restart the app to use the new version.</p>
          )}
        </div>
      </div>
    </div>
  );
}
