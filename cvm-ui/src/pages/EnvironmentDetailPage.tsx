import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { commandErrorMessage, getEnvironmentDetail } from "../lib/commands";
import { OpenInClaudeDialog } from "../components/OpenInClaudeDialog";
import { SkillEditor } from "../components/SkillEditor";
import { AccountTab } from "./tabs/AccountTab";
import { ClaudeMdTab } from "./tabs/ClaudeMdTab";
import { ConfigTab } from "./tabs/ConfigTab";
import { EnvVarsTab } from "./tabs/EnvVarsTab";
import { MarketplacesTab } from "./tabs/MarketplacesTab";
import { McpServersTab } from "./tabs/McpServersTab";
import { SkillsAgentsTab } from "./tabs/SkillsAgentsTab";

const TABS = ["overview", "claudeMd", "marketplaces", "mcp", "envvars", "skills", "config", "account"] as const;
type Tab = (typeof TABS)[number];

interface Props {
  name: string;
  onBack: () => void;
}

export function EnvironmentDetailPage({ name, onBack }: Props) {
  const [tab, setTab] = useState<Tab>("overview");
  const [editing, setEditing] = useState<{ kind: "skill" | "agent"; id: string } | null>(null);
  const [openingInClaude, setOpeningInClaude] = useState(false);
  const { data, isPending, error } = useQuery({
    queryKey: ["environment-detail", name],
    queryFn: () => getEnvironmentDetail(name),
  });

  if (editing) {
    return (
      <SkillEditor
        envName={name}
        kind={editing.kind}
        id={editing.id}
        onClose={() => setEditing(null)}
      />
    );
  }

  const pluginCount = data?.marketplaces.flatMap((m) => m.plugins).filter((p) => p.installed).length ?? 0;
  const mcpServersCount = data?.mcpServers.length ?? 0;
  const skillsAgentsCount = (data?.skills.length ?? 0) + (data?.agents.length ?? 0);
  const envVarsCount = data?.envVars.length ?? 0;

  const tabLabels: Record<Tab, string> = {
    overview: "Overview",
    claudeMd: "CLAUDE.md",
    marketplaces: `Marketplaces & Plugins (${pluginCount})`,
    mcp: `MCP Servers (${mcpServersCount})`,
    envvars: `Env Vars (${envVarsCount})`,
    skills: `Skills & Agents (${skillsAgentsCount})`,
    config: "Configuration",
    account: "Account",
  };

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <header className="flex-shrink-0 border-b border-neutral-800 px-7 pb-4 pt-5">
        <button onClick={onBack} className="mb-2 text-xs text-neutral-500 hover:text-neutral-300">
          ← Environments <span className="text-neutral-700">/</span> <span className="text-neutral-300">{name}</span>
        </button>
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-center gap-3">
            <div className="h-8 w-[3px] rounded bg-orange-500" />
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-xl font-bold tracking-tight text-neutral-100">{name}</h1>
                {data?.active && (
                  <span className="rounded px-1.5 py-0.5 font-mono text-[10px] font-semibold tracking-wide text-emerald-400">
                    ACTIVE
                  </span>
                )}
              </div>
              {data && <p className="font-mono text-xs text-neutral-500">{data.path}</p>}
            </div>
          </div>
          <div className="flex flex-col items-end gap-1">
            <button
              onClick={() => setOpeningInClaude(true)}
              className="rounded border border-orange-500/40 bg-orange-500/10 px-3.5 py-1.5 text-xs font-semibold text-orange-400 hover:bg-orange-500/20"
            >
              Open in Claude
            </button>
          </div>
        </div>
      </header>

      {isPending && <p className="p-7 text-sm text-neutral-400">Loading…</p>}
      {error && <p className="p-7 text-sm text-red-400">Error loading environment: {commandErrorMessage(error)}</p>}

      {data && (
        <div className="flex flex-1 flex-col overflow-hidden">
          {data.warnings.length > 0 && (
            <div className="flex-shrink-0 border-b border-amber-900/40 bg-amber-950/30 px-7 py-3 text-xs text-amber-400">
              {data.warnings.join(" · ")}
            </div>
          )}
          <nav className="flex flex-shrink-0 gap-5 border-b border-neutral-800 px-7">
            {TABS.map((t) => (
              <button
                key={t}
                onClick={() => setTab(t)}
                className={`border-b-2 py-3 text-sm ${
                  tab === t ? "border-orange-500 text-neutral-100" : "border-transparent text-neutral-500 hover:text-neutral-300"
                }`}
              >
                {tabLabels[t]}
              </button>
            ))}
          </nav>
          <div className="flex-1 overflow-y-auto">
            {tab === "overview" && (
              <div className="grid grid-cols-1 gap-3 p-7 sm:grid-cols-3">
                <div className="rounded-lg border border-neutral-800 bg-neutral-900/60 p-4">
                  <p className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-neutral-600">
                    Active plugins
                  </p>
                  <p className="text-2xl font-bold text-neutral-100">{pluginCount}</p>
                </div>
                <div className="rounded-lg border border-neutral-800 bg-neutral-900/60 p-4">
                  <p className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-neutral-600">
                    Skills & Agents
                  </p>
                  <p className="text-2xl font-bold text-neutral-100">{skillsAgentsCount}</p>
                </div>
                <div className="rounded-lg border border-neutral-800 bg-neutral-900/60 p-4">
                  <p className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-neutral-600">
                    Env Variables
                  </p>
                  <p className="text-2xl font-bold text-neutral-100">{envVarsCount}</p>
                </div>
              </div>
            )}
            {tab === "claudeMd" && <ClaudeMdTab envName={name} />}
            {tab === "config" && <ConfigTab config={data.config} envName={name} onRemoved={onBack} />}
            {tab === "envvars" && <EnvVarsTab envName={name} envVars={data.envVars} />}
            {tab === "marketplaces" && <MarketplacesTab envName={name} marketplaces={data.marketplaces} />}
            {tab === "mcp" && <McpServersTab mcpServers={data.mcpServers} />}
            {tab === "skills" && (
              <SkillsAgentsTab
                envName={name}
                skills={data.skills}
                agents={data.agents}
                onEdit={(kind, id) => setEditing({ kind, id })}
              />
            )}
            {tab === "account" && <AccountTab envName={name} account={data.account} />}
          </div>
        </div>
      )}
      {openingInClaude && <OpenInClaudeDialog envName={name} onClose={() => setOpeningInClaude(false)} />}
    </div>
  );
}
