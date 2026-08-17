import { useMutation, useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { commandErrorMessage, getEnvironmentDetail, openInClaude } from "../lib/commands";
import { AccountTab } from "./tabs/AccountTab";
import { ConfigTab } from "./tabs/ConfigTab";
import { EnvVarsTab } from "./tabs/EnvVarsTab";
import { MarketplacesTab } from "./tabs/MarketplacesTab";
import { SkillsAgentsTab } from "./tabs/SkillsAgentsTab";

const TABS = ["config", "envvars", "marketplaces", "skills", "account"] as const;
type Tab = (typeof TABS)[number];

const TAB_LABELS: Record<Tab, string> = {
  config: "Config",
  envvars: "Env Vars",
  marketplaces: "Marketplaces & Plugins",
  skills: "Skills & Agents",
  account: "Conta",
};

interface Props {
  name: string;
  onBack: () => void;
}

export function EnvironmentDetailPage({ name, onBack }: Props) {
  const [tab, setTab] = useState<Tab>("config");
  const { data, isPending, error } = useQuery({
    queryKey: ["environment-detail", name],
    queryFn: () => getEnvironmentDetail(name),
  });
  const openInClaudeMutation = useMutation({ mutationFn: () => openInClaude(name) });

  return (
    <div>
      <header className="flex items-center justify-between gap-3 border-b border-neutral-800 px-6 py-4">
        <div className="flex items-center gap-3">
          <button onClick={onBack} className="text-sm text-neutral-400 hover:text-neutral-200">
            ← Voltar
          </button>
          <h2 className="text-sm font-medium text-neutral-100">{name}</h2>
        </div>
        <div className="flex flex-col items-end gap-1">
          <button
            onClick={() => openInClaudeMutation.mutate()}
            className="rounded bg-neutral-100 px-3 py-1.5 text-xs font-medium text-neutral-950 hover:bg-white"
          >
            Abrir no Claude
          </button>
          {openInClaudeMutation.error && (
            <p className="text-xs text-red-400">Erro: {commandErrorMessage(openInClaudeMutation.error)}</p>
          )}
        </div>
      </header>

      {isPending && <p className="p-6 text-sm text-neutral-400">Carregando…</p>}
      {error && <p className="p-6 text-sm text-red-400">Erro ao carregar ambiente: {commandErrorMessage(error)}</p>}

      {data && (
        <>
          {data.warnings.length > 0 && (
            <div className="border-b border-amber-900/40 bg-amber-950/30 px-6 py-3 text-xs text-amber-400">
              {data.warnings.join(" · ")}
            </div>
          )}
          <nav className="flex gap-4 border-b border-neutral-800 px-6">
            {TABS.map((t) => (
              <button
                key={t}
                onClick={() => setTab(t)}
                className={`border-b-2 py-3 text-sm ${
                  tab === t ? "border-neutral-100 text-neutral-100" : "border-transparent text-neutral-500"
                }`}
              >
                {TAB_LABELS[t]}
              </button>
            ))}
          </nav>
          {tab === "config" && <ConfigTab config={data.config} envName={name} onRemoved={onBack} />}
          {tab === "envvars" && <EnvVarsTab envName={name} envVars={data.envVars} />}
          {tab === "marketplaces" && <MarketplacesTab marketplaces={data.marketplaces} />}
          {tab === "skills" && <SkillsAgentsTab skills={data.skills} agents={data.agents} />}
          {tab === "account" && <AccountTab account={data.account} />}
        </>
      )}
    </div>
  );
}
