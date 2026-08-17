import { useQuery } from "@tanstack/react-query";
import { listEnvironments } from "../lib/commands";

interface Props {
  onSelect: (name: string) => void;
}

export function EnvironmentsListPage({ onSelect }: Props) {
  const { data, isPending, error } = useQuery({
    queryKey: ["environments"],
    queryFn: listEnvironments,
  });

  if (isPending) {
    return <p className="p-6 text-sm text-neutral-400">Carregando ambientes…</p>;
  }

  if (error) {
    return <p className="p-6 text-sm text-red-400">Erro ao listar ambientes: {String(error)}</p>;
  }

  if (data.length === 0) {
    return <p className="p-6 text-sm text-neutral-400">Nenhum ambiente encontrado. Crie um com `cvm create &lt;nome&gt;`.</p>;
  }

  return (
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
  );
}
