import type { ConfigSection } from "../../lib/commands";

export function ConfigTab({ config }: { config: ConfigSection }) {
  return (
    <div className="space-y-4 p-6 text-sm">
      <section>
        <h3 className="mb-1 font-medium text-neutral-200">Allowed tools</h3>
        <p className="text-neutral-400">{config.allowedTools.join(", ") || "—"}</p>
      </section>
      <section>
        <h3 className="mb-1 font-medium text-neutral-200">Denied tools</h3>
        <p className="text-neutral-400">{config.deniedTools.join(", ") || "—"}</p>
      </section>
      <section>
        <h3 className="mb-1 font-medium text-neutral-200">Outras chaves (settings.json)</h3>
        <pre className="overflow-x-auto rounded bg-neutral-900 p-3 text-xs text-neutral-400">
          {JSON.stringify(config.other, null, 2)}
        </pre>
      </section>
    </div>
  );
}
