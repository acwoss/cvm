import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { SearchIcon, SpinnerIcon, TrashIcon } from "../../components/Icons";
import type { MarketplaceInfo, PluginInfo } from "../../lib/commands";
import {
  addMarketplace,
  commandErrorMessage,
  disablePlugin,
  enablePlugin,
  installPlugin,
  removeMarketplace,
  uninstallPlugin,
} from "../../lib/commands";

interface Props {
  envName: string;
  marketplaces: MarketplaceInfo[];
}

function pluginId(marketplaceId: string, plugin: PluginInfo): string {
  return `${plugin.name}@${marketplaceId}`;
}

export function MarketplacesTab({ envName, marketplaces }: Props) {
  const queryClient = useQueryClient();
  const [newSource, setNewSource] = useState("");
  const [search, setSearch] = useState("");

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["environment-detail", envName] });
  }

  const addMarketplaceMutation = useMutation({
    mutationFn: () => addMarketplace(envName, newSource),
    onSuccess: () => {
      setNewSource("");
      invalidate();
    },
  });

  const removeMarketplaceMutation = useMutation({
    mutationFn: (marketplace: string) => removeMarketplace(envName, marketplace),
    onSuccess: invalidate,
  });

  const installMutation = useMutation({
    mutationFn: (plugin: string) => installPlugin(envName, plugin),
    onSuccess: invalidate,
  });

  const uninstallMutation = useMutation({
    mutationFn: (plugin: string) => uninstallPlugin(envName, plugin),
    onSuccess: invalidate,
  });

  const enableMutation = useMutation({
    mutationFn: (plugin: string) => enablePlugin(envName, plugin),
    onSuccess: invalidate,
  });

  const disableMutation = useMutation({
    mutationFn: (plugin: string) => disablePlugin(envName, plugin),
    onSuccess: invalidate,
  });

  function handleRemoveMarketplace(id: string) {
    if (window.confirm(`Remove the marketplace '${id}'? This cannot be undone.`)) {
      removeMarketplaceMutation.mutate(id);
    }
  }

  const query = search.trim().toLowerCase();
  const filteredMarketplaces = query
    ? marketplaces
        .map((m) => ({ ...m, plugins: m.plugins.filter((p) => p.name.toLowerCase().includes(query)) }))
        .filter((m) => m.plugins.length > 0)
    : marketplaces;

  return (
    <div className="space-y-6 p-6 text-sm">
      <section className="rounded border border-neutral-800 bg-neutral-900 p-4">
        <h3 className="mb-2 font-medium text-neutral-200">Add marketplace</h3>
        <div className="flex gap-2">
          <input
            value={newSource}
            onChange={(e) => setNewSource(e.target.value)}
            placeholder="URL, path, or GitHub repository"
            className="flex-1 rounded border border-neutral-700 bg-neutral-950 px-3 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500"
          />
          <button
            onClick={() => addMarketplaceMutation.mutate()}
            disabled={newSource.trim().length === 0 || addMarketplaceMutation.isPending}
            className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white disabled:opacity-50"
          >
            {addMarketplaceMutation.isPending ? "Adding…" : "Add"}
          </button>
        </div>
        {addMarketplaceMutation.error && (
          <p className="mt-2 text-xs text-red-400">
            Error: {commandErrorMessage(addMarketplaceMutation.error)}
          </p>
        )}
      </section>

      {marketplaces.length > 0 && (
        <div className="relative max-w-sm">
          <span className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-neutral-600">
            <SearchIcon />
          </span>
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search MCP servers…"
            className="w-full rounded border border-neutral-800 bg-neutral-900 py-1.5 pl-8 pr-3 text-xs text-neutral-100 outline-none focus:border-neutral-600"
          />
        </div>
      )}

      {marketplaces.length === 0 ? (
        <p className="text-neutral-400">No marketplaces installed.</p>
      ) : filteredMarketplaces.length === 0 ? (
        <p className="text-neutral-400">No MCP servers match your search.</p>
      ) : (
        filteredMarketplaces.map((m) => (
          <section key={m.id}>
            <div className="mb-2 flex items-center justify-between">
              <h3 className="font-mono text-xs font-semibold uppercase tracking-wide text-neutral-500">
                {m.id} {m.repo && <span className="text-neutral-600">({m.repo})</span>}
              </h3>
              <button
                onClick={() => handleRemoveMarketplace(m.id)}
                disabled={removeMarketplaceMutation.isPending && removeMarketplaceMutation.variables === m.id}
                className="flex items-center gap-1.5 text-xs text-red-400 underline hover:text-red-300 disabled:opacity-50"
              >
                {removeMarketplaceMutation.isPending && removeMarketplaceMutation.variables === m.id && (
                  <SpinnerIcon size={11} />
                )}
                remove marketplace
              </button>
            </div>
            {removeMarketplaceMutation.error && removeMarketplaceMutation.variables === m.id && (
              <p className="mb-2 text-xs text-red-400">
                Error: {commandErrorMessage(removeMarketplaceMutation.error)}
              </p>
            )}
            <div className="flex flex-col gap-2">
              {m.plugins.map((p) => {
                const id = pluginId(m.id, p);
                return (
                  <div
                    key={p.name}
                    className="rounded-lg border border-neutral-800 bg-neutral-900/60 px-3.5 py-2.5"
                  >
                    <div className="flex items-center justify-between gap-3">
                      <div className="flex min-w-0 items-center gap-2">
                        <span
                          className={`h-1.5 w-1.5 flex-shrink-0 rounded-full ${
                            p.installed && p.enabled ? "bg-emerald-400" : "bg-neutral-600"
                          }`}
                        />
                        <span className="truncate font-mono text-xs font-medium text-neutral-100">
                          {p.name}
                        </span>
                        {p.version && (
                          <span className="flex-shrink-0 font-mono text-[10px] text-neutral-600">
                            v{p.version}
                          </span>
                        )}
                      </div>
                      <div className="flex flex-shrink-0 items-center gap-3">
                        {p.installed ? (
                          <>
                            <button
                              onClick={() =>
                                p.enabled ? disableMutation.mutate(id) : enableMutation.mutate(id)
                              }
                              disabled={
                                (enableMutation.isPending && enableMutation.variables === id) ||
                                (disableMutation.isPending && disableMutation.variables === id)
                              }
                              className="relative flex h-[18px] w-8 items-center justify-center rounded-full transition-colors disabled:opacity-60"
                              style={{ background: p.enabled ? "rgba(0,229,255,0.18)" : "#282B33" }}
                            >
                              {(enableMutation.isPending && enableMutation.variables === id) ||
                              (disableMutation.isPending && disableMutation.variables === id) ? (
                                <SpinnerIcon size={11} />
                              ) : (
                                <span
                                  className="absolute top-0.5 h-3.5 w-3.5 rounded-full transition-all"
                                  style={{
                                    left: p.enabled ? 15 : 2,
                                    background: p.enabled ? "#00E5FF" : "#555B68",
                                  }}
                                />
                              )}
                            </button>
                            <button
                              onClick={() => uninstallMutation.mutate(id)}
                              disabled={uninstallMutation.isPending && uninstallMutation.variables === id}
                              className="text-neutral-600 hover:text-red-400 disabled:opacity-50"
                            >
                              {uninstallMutation.isPending && uninstallMutation.variables === id ? (
                                <SpinnerIcon size={13} />
                              ) : (
                                <TrashIcon />
                              )}
                            </button>
                          </>
                        ) : (
                          <button
                            onClick={() => installMutation.mutate(id)}
                            disabled={installMutation.isPending && installMutation.variables === id}
                            className="flex items-center gap-1.5 rounded border border-neutral-700 px-2.5 py-1 text-xs text-neutral-300 hover:border-neutral-500 hover:text-neutral-100 disabled:opacity-50"
                          >
                            {installMutation.isPending && installMutation.variables === id && (
                              <SpinnerIcon size={11} />
                            )}
                            {installMutation.isPending && installMutation.variables === id
                              ? "installing…"
                              : "install"}
                          </button>
                        )}
                      </div>
                    </div>
                    {p.description && <p className="mt-1 text-xs text-neutral-500">{p.description}</p>}
                    {installMutation.error && installMutation.variables === id && (
                      <p className="mt-1 text-xs text-red-400">Error: {commandErrorMessage(installMutation.error)}</p>
                    )}
                    {uninstallMutation.error && uninstallMutation.variables === id && (
                      <p className="mt-1 text-xs text-red-400">
                        Error: {commandErrorMessage(uninstallMutation.error)}
                      </p>
                    )}
                    {enableMutation.error && enableMutation.variables === id && (
                      <p className="mt-1 text-xs text-red-400">Error: {commandErrorMessage(enableMutation.error)}</p>
                    )}
                    {disableMutation.error && disableMutation.variables === id && (
                      <p className="mt-1 text-xs text-red-400">
                        Error: {commandErrorMessage(disableMutation.error)}
                      </p>
                    )}
                  </div>
                );
              })}
            </div>
          </section>
        ))
      )}
    </div>
  );
}
