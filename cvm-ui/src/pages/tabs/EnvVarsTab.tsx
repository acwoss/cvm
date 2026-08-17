import type { EnvVarSummary } from "../../lib/commands";

export function EnvVarsTab({ envVars }: { envVars: EnvVarSummary[] }) {
  if (envVars.length === 0) {
    return <p className="p-6 text-sm text-neutral-400">Nenhuma variável de ambiente.</p>;
  }
  return (
    <ul className="divide-y divide-neutral-800 text-sm">
      {envVars.map((v) => (
        <li key={`${v.source}:${v.key}`} className="flex items-center justify-between px-6 py-3">
          <span className="text-neutral-200">{v.key}</span>
          <span className="text-xs text-neutral-500">{v.source === "dotenv" ? ".env" : "settings.json"}</span>
        </li>
      ))}
    </ul>
  );
}
