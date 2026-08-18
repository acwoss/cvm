import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { SearchIcon } from "../components/Icons";
import { commandErrorMessage, listEnvironments } from "../lib/commands";

interface Props {
  onSelect: (name: string) => void;
}

export function EnvironmentsListPage({ onSelect }: Props) {
  const [search, setSearch] = useState("");
  const { data, isPending, error } = useQuery({
    queryKey: ["environments"],
    queryFn: listEnvironments,
  });

  const activeCount = data?.filter((env) => env.active).length ?? 0;
  const filtered = data?.filter((env) => env.name.toLowerCase().includes(search.toLowerCase())) ?? [];

  return (
    <div className="p-7">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-lg font-bold tracking-tight text-neutral-100">Environments</h1>
        {data && data.length > 0 && (
          <p className="text-xs text-neutral-500">
            <span className="font-semibold text-orange-400">{activeCount}</span> of {data.length} active
          </p>
        )}
      </div>

      <div className="mb-5 flex items-center gap-2">
        <div className="relative flex-1 max-w-sm">
          <span className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-neutral-600">
            <SearchIcon />
          </span>
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search environments…"
            className="w-full rounded border border-neutral-800 bg-neutral-900 py-1.5 pl-8 pr-3 text-xs text-neutral-100 outline-none focus:border-neutral-600"
          />
        </div>
      </div>

      {isPending && <p className="text-sm text-neutral-400">Loading environments…</p>}
      {error && <p className="text-sm text-red-400">Error listing environments: {commandErrorMessage(error)}</p>}
      {data && data.length === 0 && (
        <p className="text-sm text-neutral-400">No environments found. Create one with the "+" button in the sidebar.</p>
      )}
      {data && data.length > 0 && filtered.length === 0 && (
        <p className="text-sm text-neutral-400">No environments match your search.</p>
      )}

      {filtered.length > 0 && (
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {filtered.map((env) => (
            <div
              key={env.name}
              onClick={() => onSelect(env.name)}
              className="cursor-pointer rounded-lg border border-neutral-800 bg-neutral-900/60 p-4 transition-colors hover:border-orange-500/60 hover:bg-orange-500/10"
            >
              <p className="mb-0.5 text-sm font-semibold text-neutral-100">{env.name}</p>
              <p className="mb-3 truncate font-mono text-xs text-neutral-500">{env.path}</p>
              <div className="flex justify-end">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onSelect(env.name);
                  }}
                  className="rounded border border-orange-500/40 px-3 py-1 text-xs font-medium text-orange-400 hover:bg-orange-500/10"
                >
                  Open
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
