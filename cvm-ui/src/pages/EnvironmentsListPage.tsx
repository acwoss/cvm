import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { CreateEnvironmentDialog } from "../components/CreateEnvironmentDialog";
import { commandErrorMessage, listEnvironments } from "../lib/commands";

interface Props {
  onSelect: (name: string) => void;
}

export function EnvironmentsListPage({ onSelect }: Props) {
  const [showCreate, setShowCreate] = useState(false);
  const { data, isPending, error } = useQuery({
    queryKey: ["environments"],
    queryFn: listEnvironments,
  });

  return (
    <div>
      <div className="flex items-center justify-between px-6 py-4">
        <h1 className="text-sm font-medium text-neutral-100">Ambientes</h1>
        <button
          onClick={() => setShowCreate(true)}
          className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white"
        >
          + Novo ambiente
        </button>
      </div>

      {isPending && <p className="p-6 text-sm text-neutral-400">Carregando ambientes…</p>}
      {error && <p className="p-6 text-sm text-red-400">Erro ao listar ambientes: {commandErrorMessage(error)}</p>}
      {data && data.length === 0 && (
        <p className="p-6 text-sm text-neutral-400">Nenhum ambiente encontrado. Crie um com o botão acima.</p>
      )}
      {data && data.length > 0 && (
        <ul className="divide-y divide-neutral-800">
          {data.map((env) => (
            <li key={env.name}>
              <button
                onClick={() => onSelect(env.name)}
                className="flex w-full items-center justify-between px-6 py-4 text-left hover:bg-neutral-900"
              >
                <div>
                  <p className="text-sm font-medium text-neutral-100">{env.name}</p>
                  <p className="text-xs text-neutral-500">{env.path}</p>
                </div>
                {env.active && (
                  <span className="rounded-full bg-emerald-500/10 px-2 py-1 text-xs text-emerald-400">
                    ativo
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      )}

      {showCreate && <CreateEnvironmentDialog onClose={() => setShowCreate(false)} />}
    </div>
  );
}
