import { useState } from "react";
import { SearchIcon } from "../../components/Icons";
import { PluginVisibilityToggle } from "../../components/PluginVisibilityToggle";
import type { McpServerInfo } from "../../lib/commands";

interface Props {
  mcpServers: McpServerInfo[];
}

function originLabel(server: McpServerInfo): string {
  return server.source.kind === "native" ? "native" : server.source.plugin;
}

function originKey(source: McpServerInfo["source"]): string {
  return source.kind === "native" ? "native" : `${source.marketplace}/${source.plugin}`;
}

export function McpServersTab({ mcpServers }: Props) {
  const [search, setSearch] = useState("");
  const [showPlugins, setShowPlugins] = useState(true);

  const query = search.trim().toLowerCase();
  const visible = mcpServers
    .filter((s) => showPlugins || s.source.kind === "native")
    .filter((s) => !query || s.name.toLowerCase().includes(query));

  return (
    <div className="space-y-4 p-7">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-semibold uppercase tracking-wider text-neutral-500">
          {mcpServers.length} MCP servers
        </h3>
        <PluginVisibilityToggle checked={showPlugins} onChange={setShowPlugins} />
      </div>

      {mcpServers.length > 0 && (
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

      {visible.length === 0 ? (
        <p className="text-sm text-neutral-500">
          {mcpServers.length === 0 ? "No MCP servers configured." : "No MCP servers match your filters."}
        </p>
      ) : (
        <div className="flex flex-col gap-2">
          {visible.map((server) => (
            <div
              key={`${originKey(server.source)}-${server.name}`}
              className="rounded-lg border border-neutral-800 bg-neutral-900/60 px-3.5 py-2.5"
            >
              <div className="flex items-center justify-between gap-3">
                <span className="truncate font-mono text-xs font-medium text-neutral-100">{server.name}</span>
                <span
                  className="flex-shrink-0 rounded border border-neutral-800 px-1.5 py-0.5 font-mono text-[10px] text-neutral-500"
                  title={
                    server.source.kind === "native"
                      ? "Configured directly in this environment"
                      : `Provided by the plugin '${server.source.plugin}' (${server.source.marketplace})`
                  }
                >
                  {originLabel(server)}
                </span>
              </div>
              <p className="mt-1 truncate font-mono text-xs text-neutral-500">
                {server.command} {server.args.join(" ")}
              </p>
              {server.envKeys.length > 0 && (
                <p className="mt-1 font-mono text-[10px] text-neutral-600">env: {server.envKeys.join(", ")}</p>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
