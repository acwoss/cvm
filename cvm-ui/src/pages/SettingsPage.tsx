import { getVersion } from "@tauri-apps/api/app";
import { useQuery } from "@tanstack/react-query";

export function SettingsPage({ onBack: _onBack }: { onBack: () => void }) {
  const { data: version } = useQuery({
    queryKey: ["app-version"],
    queryFn: getVersion,
  });

  return (
    <div className="p-7">
      <h1 className="mb-5 text-lg font-bold tracking-tight text-neutral-100">Configurações</h1>

      <div className="max-w-xl divide-y divide-neutral-800 rounded-lg border border-neutral-800">
        <div className="flex items-center justify-between px-4 py-3.5">
          <div>
            <p className="text-sm text-neutral-100">cvm-ui</p>
            <p className="text-xs text-neutral-500">Versão do aplicativo</p>
          </div>
          <p className="font-mono text-xs text-neutral-400">{version ?? "…"}</p>
        </div>
        <div className="flex items-center justify-between px-4 py-3.5">
          <div>
            <p className="text-sm text-neutral-100">Diretório de ambientes</p>
            <p className="text-xs text-neutral-500">Onde cada ambiente do cvm vive no disco</p>
          </div>
          <p className="font-mono text-xs text-neutral-400">~/.cvm/envs/</p>
        </div>
        <div className="flex items-center justify-between px-4 py-3.5">
          <div>
            <p className="text-sm text-neutral-100">Hooks globais</p>
            <p className="text-xs text-neutral-500">Gerenciados na aba "Hooks" da barra lateral</p>
          </div>
          <p className="font-mono text-xs text-neutral-400">~/.cvm/hooks/</p>
        </div>
      </div>

      <p className="mt-4 max-w-xl text-xs text-neutral-600">
        Preferências configuráveis (shell padrão, notificações, canal de atualização) ainda não
        existem no cvm — esta tela mostra só informação real até essa camada existir.
      </p>
    </div>
  );
}
