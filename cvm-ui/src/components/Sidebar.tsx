import type React from "react";
import {
  CloudIcon,
  HomeIcon,
  LogoMark,
  PackageIcon,
  PlusIcon,
  ServerIcon,
  SettingsIcon,
  TerminalIcon,
} from "./Icons";

export type NavPage = "environments" | "hooks" | "settings";

interface Props {
  active: NavPage;
  onNav: (page: NavPage) => void;
  onQuickCreate: () => void;
}

const NAV_ITEMS: { id: NavPage; icon: React.ComponentType<{ size?: number }>; label: string }[] = [
  { id: "environments", icon: HomeIcon, label: "Environments" },
  { id: "hooks", icon: TerminalIcon, label: "Hooks" },
  { id: "settings", icon: SettingsIcon, label: "Settings" },
];

export function Sidebar({ active, onNav, onQuickCreate }: Props) {
  return (
    <nav className="flex w-[52px] min-w-[52px] flex-col items-center gap-0 border-r border-[#1E2028] bg-[#0D0F11] py-4">
      <div className="mb-6 text-orange-500/90">
        <LogoMark />
      </div>

      <div className="flex flex-1 flex-col gap-0.5">
        {NAV_ITEMS.map((item) => {
          const Icon = item.icon;
          const isActive = active === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onNav(item.id)}
              title={item.label}
              className={`flex h-9 w-9 items-center justify-center rounded-lg transition-colors ${
                isActive
                  ? "bg-orange-500/10 text-orange-500"
                  : "text-neutral-600 hover:text-neutral-400"
              }`}
            >
              <Icon />
            </button>
          );
        })}

        {/* Estes dois ícones não têm uma tela global própria hoje (marketplaces e
            plugins são sempre por-ambiente, geridos na aba do próprio ambiente) -
            ficam presentes só por fidelidade visual ao mockup, sem fingir uma
            funcionalidade que não existe, mesmo tratamento que o protótipo dá ao
            ícone de nuvem abaixo. */}
        <div className="mt-1 flex h-9 w-9 cursor-default items-center justify-center text-neutral-800" title="Coming soon">
          <PackageIcon />
        </div>
        <div className="flex h-9 w-9 cursor-default items-center justify-center text-neutral-800" title="Coming soon">
          <ServerIcon />
        </div>
      </div>

      <div className="mb-3 text-neutral-600">
        <CloudIcon />
      </div>
      <button
        onClick={onQuickCreate}
        title="New environment"
        className="flex h-8 w-8 items-center justify-center rounded-lg border border-neutral-700 text-neutral-400 transition-colors hover:border-orange-500/60 hover:bg-orange-500/10 hover:text-orange-500"
      >
        <PlusIcon />
      </button>
    </nav>
  );
}
