import { useState } from "react";
import type { EnvVarSource, EnvVarSummary } from "../../lib/commands";
import { revealEnvVar } from "../../lib/commands";

interface Props {
  envName: string;
  envVars: EnvVarSummary[];
}

export function EnvVarsTab({ envName, envVars }: Props) {
  const [revealed, setRevealed] = useState<Record<string, string>>({});

  async function reveal(source: EnvVarSource, key: string) {
    const value = await revealEnvVar(envName, source, key);
    setRevealed((prev) => ({ ...prev, [`${source}:${key}`]: value }));
  }

  if (envVars.length === 0) {
    return <p className="p-6 text-sm text-neutral-400">Nenhuma variável de ambiente.</p>;
  }

  return (
    <ul className="divide-y divide-neutral-800 text-sm">
      {envVars.map((v) => {
        const id = `${v.source}:${v.key}`;
        return (
          <li key={id} className="flex items-center justify-between px-6 py-3">
            <div>
              <p className="text-neutral-200">{v.key}</p>
              {revealed[id] !== undefined && (
                <p className="font-mono text-xs text-amber-400">{revealed[id]}</p>
              )}
            </div>
            <div className="flex items-center gap-3">
              <span className="text-xs text-neutral-500">{v.source === "dotenv" ? ".env" : "settings.json"}</span>
              {revealed[id] === undefined && (
                <button
                  onClick={() => reveal(v.source, v.key)}
                  className="text-xs text-neutral-400 underline hover:text-neutral-200"
                >
                  revelar
                </button>
              )}
            </div>
          </li>
        );
      })}
    </ul>
  );
}
