import type { MarketplaceInfo } from "../../lib/commands";

export function MarketplacesTab({ marketplaces }: { marketplaces: MarketplaceInfo[] }) {
  if (marketplaces.length === 0) {
    return <p className="p-6 text-sm text-neutral-400">Nenhum marketplace instalado.</p>;
  }
  return (
    <div className="space-y-6 p-6 text-sm">
      {marketplaces.map((m) => (
        <section key={m.id}>
          <h3 className="font-medium text-neutral-200">
            {m.id} {m.repo && <span className="text-xs text-neutral-500">({m.repo})</span>}
          </h3>
          <ul className="mt-2 divide-y divide-neutral-800">
            {m.plugins.map((p) => (
              <li key={p.name} className="flex items-center justify-between py-2">
                <div>
                  <p className="text-neutral-200">{p.name}</p>
                  {p.description && <p className="text-xs text-neutral-500">{p.description}</p>}
                </div>
                <div className="flex items-center gap-2 text-xs">
                  {p.version && <span className="text-neutral-500">v{p.version}</span>}
                  <span className={p.enabled ? "text-emerald-400" : "text-neutral-500"}>
                    {p.enabled ? "habilitado" : "desabilitado"}
                  </span>
                </div>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
