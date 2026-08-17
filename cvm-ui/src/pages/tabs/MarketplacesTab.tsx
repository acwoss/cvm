import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
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
    if (window.confirm(`Remover o marketplace '${id}'? Isso não pode ser desfeito.`)) {
      removeMarketplaceMutation.mutate(id);
    }
  }

  return (
    <div className="space-y-6 p-6 text-sm">
      <section className="rounded border border-neutral-800 bg-neutral-900 p-4">
        <h3 className="mb-2 font-medium text-neutral-200">Adicionar marketplace</h3>
        <div className="flex gap-2">
          <input
            value={newSource}
            onChange={(e) => setNewSource(e.target.value)}
            placeholder="URL, caminho ou repositório do GitHub"
            className="flex-1 rounded border border-neutral-700 bg-neutral-950 px-3 py-1.5 text-xs text-neutral-100 outline-none focus:border-neutral-500"
          />
          <button
            onClick={() => addMarketplaceMutation.mutate()}
            disabled={newSource.trim().length === 0 || addMarketplaceMutation.isPending}
            className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white disabled:opacity-50"
          >
            {addMarketplaceMutation.isPending ? "Adicionando…" : "Adicionar"}
          </button>
        </div>
        {addMarketplaceMutation.error && (
          <p className="mt-2 text-xs text-red-400">
            Erro: {commandErrorMessage(addMarketplaceMutation.error)}
          </p>
        )}
      </section>

      {marketplaces.length === 0 ? (
        <p className="text-neutral-400">Nenhum marketplace instalado.</p>
      ) : (
        marketplaces.map((m) => (
          <section key={m.id}>
            <div className="flex items-center justify-between">
              <h3 className="font-medium text-neutral-200">
                {m.id} {m.repo && <span className="text-xs text-neutral-500">({m.repo})</span>}
              </h3>
              <button
                onClick={() => handleRemoveMarketplace(m.id)}
                className="text-xs text-red-400 underline hover:text-red-300"
              >
                remover marketplace
              </button>
            </div>
            {removeMarketplaceMutation.error && removeMarketplaceMutation.variables === m.id && (
              <p className="mt-1 text-xs text-red-400">
                Erro: {commandErrorMessage(removeMarketplaceMutation.error)}
              </p>
            )}
            <ul className="mt-2 divide-y divide-neutral-800">
              {m.plugins.map((p) => {
                const id = pluginId(m.id, p);
                return (
                  <li key={p.name} className="flex items-center justify-between py-2">
                    <div>
                      <p className="text-neutral-200">{p.name}</p>
                      {p.description && <p className="text-xs text-neutral-500">{p.description}</p>}
                    </div>
                    <div className="flex items-center gap-2 text-xs">
                      {p.version && <span className="text-neutral-500">v{p.version}</span>}
                      {p.installed ? (
                        <>
                          <button
                            onClick={() =>
                              p.enabled ? disableMutation.mutate(id) : enableMutation.mutate(id)
                            }
                            className={p.enabled ? "text-emerald-400 underline" : "text-neutral-500 underline"}
                          >
                            {p.enabled ? "habilitado" : "desabilitado"}
                          </button>
                          <button
                            onClick={() => uninstallMutation.mutate(id)}
                            className="text-red-400 underline hover:text-red-300"
                          >
                            desinstalar
                          </button>
                        </>
                      ) : (
                        <button
                          onClick={() => installMutation.mutate(id)}
                          className="text-neutral-100 underline hover:text-white"
                        >
                          instalar
                        </button>
                      )}
                    </div>
                  </li>
                );
              })}
            </ul>
          </section>
        ))
      )}
    </div>
  );
}
