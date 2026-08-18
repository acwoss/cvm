import { useMutation } from "@tanstack/react-query";
import { checkAuthStatus, commandErrorMessage } from "../../lib/commands";
import type { AccountInfo } from "../../lib/commands";

interface Props {
  envName: string;
  account: AccountInfo | null;
}

function authMethodLabel(account: AccountInfo): string {
  return account.authMethod === "oauth" ? "Anthropic Account (OAuth)" : "Authenticated via API key";
}

export function AccountTab({ envName, account }: Props) {
  const statusMutation = useMutation({
    mutationFn: () => checkAuthStatus(envName),
  });

  return (
    <div className="space-y-6 p-7 text-sm">
      {!account ? (
        <p className="text-neutral-400">Anonymous environment (no authenticated account).</p>
      ) : (
        <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-2">
          <dt className="text-neutral-500">Authentication</dt>
          <dd className="text-neutral-200">{authMethodLabel(account)}</dd>
          {account.authMethod === "oauth" && (
            <>
              <dt className="text-neutral-500">Email</dt>
              <dd className="text-neutral-200">{account.email ?? "—"}</dd>
              <dt className="text-neutral-500">Name</dt>
              <dd className="text-neutral-200">{account.displayName ?? "—"}</dd>
              <dt className="text-neutral-500">Organization</dt>
              <dd className="text-neutral-200">{account.organizationName ?? "—"}</dd>
              <dt className="text-neutral-500">Plan</dt>
              <dd className="text-neutral-200">{account.seatTier ?? "—"}</dd>
            </>
          )}
          {account.authMethod === "apiKey" && (
            <>
              <dt className="text-neutral-500">Source</dt>
              <dd className="text-neutral-200">ANTHROPIC_API_KEY environment variable</dd>
            </>
          )}
        </dl>
      )}

      <div className="border-t border-neutral-800 pt-4">
        <button
          onClick={() => statusMutation.mutate()}
          disabled={statusMutation.isPending}
          className="rounded border border-neutral-700 px-3 py-1.5 text-xs text-neutral-300 hover:border-neutral-500 hover:text-neutral-100 disabled:opacity-50"
        >
          {statusMutation.isPending ? "Checking…" : "Check status"}
        </button>

        {statusMutation.error && (
          <p className="mt-2 text-xs text-red-400">
            Error: {commandErrorMessage(statusMutation.error)}
          </p>
        )}

        {statusMutation.data && (
          <dl className="mt-3 grid grid-cols-[auto_1fr] gap-x-6 gap-y-1.5 rounded-lg border border-neutral-800 bg-neutral-900/60 p-4 font-mono text-xs">
            <dt className="text-neutral-500">loggedIn</dt>
            <dd className="text-neutral-200">{String(statusMutation.data.loggedIn)}</dd>
            <dt className="text-neutral-500">authMethod</dt>
            <dd className="text-neutral-200">{statusMutation.data.authMethod}</dd>
            {statusMutation.data.apiProvider && (
              <>
                <dt className="text-neutral-500">apiProvider</dt>
                <dd className="text-neutral-200">{statusMutation.data.apiProvider}</dd>
              </>
            )}
            {statusMutation.data.email && (
              <>
                <dt className="text-neutral-500">email</dt>
                <dd className="text-neutral-200">{statusMutation.data.email}</dd>
              </>
            )}
            {statusMutation.data.orgName && (
              <>
                <dt className="text-neutral-500">orgName</dt>
                <dd className="text-neutral-200">{statusMutation.data.orgName}</dd>
              </>
            )}
            {statusMutation.data.orgId && (
              <>
                <dt className="text-neutral-500">orgId</dt>
                <dd className="text-neutral-200">{statusMutation.data.orgId}</dd>
              </>
            )}
            {statusMutation.data.subscriptionType && (
              <>
                <dt className="text-neutral-500">subscriptionType</dt>
                <dd className="text-neutral-200">{statusMutation.data.subscriptionType}</dd>
              </>
            )}
            {statusMutation.data.apiKeySource && (
              <>
                <dt className="text-neutral-500">apiKeySource</dt>
                <dd className="text-neutral-200">{statusMutation.data.apiKeySource}</dd>
              </>
            )}
          </dl>
        )}
      </div>
    </div>
  );
}
