import type { AccountInfo } from "../../lib/commands";

export function AccountTab({ account }: { account: AccountInfo | null }) {
  if (!account) {
    return <p className="p-6 text-sm text-neutral-400">Ambiente anônimo (sem conta autenticada).</p>;
  }
  return (
    <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 p-6 text-sm">
      <dt className="text-neutral-500">E-mail</dt>
      <dd className="text-neutral-200">{account.email ?? "—"}</dd>
      <dt className="text-neutral-500">Nome</dt>
      <dd className="text-neutral-200">{account.displayName ?? "—"}</dd>
      <dt className="text-neutral-500">Organização</dt>
      <dd className="text-neutral-200">{account.organizationName ?? "—"}</dd>
      <dt className="text-neutral-500">Plano</dt>
      <dd className="text-neutral-200">{account.seatTier ?? "—"}</dd>
    </dl>
  );
}
