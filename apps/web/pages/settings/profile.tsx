import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { KeyRound, Link as LinkIcon, Trash2 } from "lucide-react";
import type { FormEvent, ReactNode } from "react";
import { useEffect, useState } from "react";
import { useRouter } from "next/router";

import { BrandIcon } from "@/components/auth/BrandIcon";
import { PasswordPolicy } from "@/components/auth/AuthLayout";
import { Alert } from "@/components/retroui/Alert";
import { Badge } from "@/components/retroui/Badge";
import { Button } from "@/components/retroui/Button";
import { Input } from "@/components/retroui/Input";
import { Text } from "@/components/retroui/Text";
import { SettingsLayout } from "@/components/settings/SettingsLayout";
import {
  getAuthProviders,
  getOptionalAuthSession,
  type AuthUser,
} from "@/lib/auth-client";
import {
  validatePasswordConfirmation,
  validatePasswordPolicy,
} from "@/lib/auth-validation";
import {
  getAccountSecurity,
  removeAccountSso,
  settingsErrorMessage,
  updateAccountPassword,
  type AccountSecurity,
  type AccountSsoProvider,
} from "@/lib/settings-client";

type ProfileSettingsPageProps = {
  initialAuthUser?: AuthUser | null;
};

export default function ProfileSettingsPage({
  initialAuthUser,
}: ProfileSettingsPageProps) {
  const router = useRouter();
  const queryClient = useQueryClient();
  const [notice, setNotice] = useState<string | null>(null);
  const sessionQuery = useQuery({
    queryKey: ["authSession"],
    queryFn: getOptionalAuthSession,
    initialData: initialAuthUser,
  });
  const securityQuery = useQuery({
    queryKey: ["accountSecurity"],
    queryFn: getAccountSecurity,
    enabled: sessionQuery.data !== null,
  });
  const providersQuery = useQuery({
    queryKey: ["authProviders"],
    queryFn: getAuthProviders,
  });
  const user = sessionQuery.data ?? null;
  const security = securityQuery.data;

  useEffect(() => {
    const linkedProvider =
      typeof router.query.sso_linked === "string" ? router.query.sso_linked : null;
    if (linkedProvider === "github" || linkedProvider === "gitlab") {
      setNotice(`${providerLabel(linkedProvider)} SSO connected.`);
      queryClient.invalidateQueries({ queryKey: ["accountSecurity"] });
      router.replace("/settings/profile", undefined, { shallow: true });
    }
  }, [queryClient, router]);

  return (
    <SettingsLayout>
      <div className="max-w-3xl space-y-6">
        <div className="space-y-2">
          <Text as="h1" className="text-3xl">
            Profile
          </Text>
          <p className="text-sm text-muted-foreground">Your account details.</p>
        </div>

        <div className="overflow-hidden border-2 border-border bg-card shadow-md">
          <dl className="divide-y-2 divide-border">
            <ProfileRow label="Username" value={user?.username ?? "Not signed in"} />
            <ProfileRow label="Email" value={user?.email ?? "Not signed in"} />
            <div className="grid gap-2 p-4 sm:grid-cols-[160px_minmax(0,1fr)]">
              <dt className="font-head text-sm font-bold">Admin</dt>
              <dd className="flex flex-wrap gap-2">
                {user?.is_admin ? (
                  <Badge size="sm" variant="surface">
                    Admin
                  </Badge>
                ) : (
                  <span className="text-sm text-muted-foreground">No</span>
                )}
              </dd>
            </div>
          </dl>
        </div>

        {notice ? (
          <Alert status="success">
            <Alert.Description className="text-sm">{notice}</Alert.Description>
          </Alert>
        ) : null}

        <PasswordPanel
          onMessage={setNotice}
          onUpdated={(nextSecurity) => {
            queryClient.setQueryData(["accountSecurity"], nextSecurity);
          }}
          security={security}
        />

        <SsoPanel
          providers={providersQuery.data}
          security={security}
          onMessage={setNotice}
          onUpdated={(nextSecurity) => {
            queryClient.setQueryData(["accountSecurity"], nextSecurity);
          }}
        />
      </div>
    </SettingsLayout>
  );
}

function ProfileRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid gap-2 p-4 sm:grid-cols-[160px_minmax(0,1fr)]">
      <dt className="font-head text-sm font-bold">{label}</dt>
      <dd className="min-w-0 break-words text-sm">{value}</dd>
    </div>
  );
}

function PasswordPanel({
  onMessage,
  onUpdated,
  security,
}: {
  onMessage: (message: string | null) => void;
  onUpdated: (security: AccountSecurity) => void;
  security: AccountSecurity | undefined;
}) {
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [error, setError] = useState<string | null>(null);

  const mutation = useMutation({
    mutationFn: updateAccountPassword,
    onSuccess: (nextSecurity) => {
      onUpdated(nextSecurity);
      setPassword("");
      setConfirmation("");
      setError(null);
      onMessage("Password updated.");
    },
    onError: (error) => setError(settingsErrorMessage(error)),
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const validation =
      validatePasswordPolicy(password) ??
      validatePasswordConfirmation(password, confirmation);

    if (validation) {
      setError(validation);
      onMessage(null);
      return;
    }

    mutation.mutate({
      password,
      password_confirmation: confirmation,
    });
  }

  return (
    <section className="space-y-6 border-2 border-border bg-card p-6 shadow-md">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="space-y-1">
          <h2 className="font-head text-3xl! font-bold">
            Password
          </h2>
          <p className="text-sm leading-5 text-muted-foreground">
            {security?.password_enabled
              ? "Update the password for this account."
              : "Add a password so this account can sign in without SSO."}
          </p>
        </div>
        <Badge size="sm" variant="surface">
          {security?.password_enabled ? "Enabled" : "Not set"}
        </Badge>
      </div>

      {error ? (
        <Alert status="error">
          <Alert.Description className="text-sm">{error}</Alert.Description>
        </Alert>
      ) : null}

      <form className="space-y-4" onSubmit={handleSubmit}>
        <SettingsField label="New password">
          <Input
            autoComplete="new-password"
            onChange={(event) => {
              setPassword(event.target.value);
              setError(null);
              onMessage(null);
            }}
            placeholder="Enter a password"
            type="password"
            value={password}
          />
        </SettingsField>

        <SettingsField label="Repeat password">
          <Input
            autoComplete="new-password"
            onChange={(event) => {
              setConfirmation(event.target.value);
              setError(null);
              onMessage(null);
            }}
            placeholder="Repeat your password"
            type="password"
            value={confirmation}
          />
        </SettingsField>

        <PasswordPolicy />

        <Button disabled={mutation.isPending} type="submit">
          <KeyRound className="mr-2 size-4" />
          {mutation.isPending ? "Saving..." : "Save password"}
        </Button>
      </form>
    </section>
  );
}

function SsoPanel({
  onMessage,
  onUpdated,
  providers,
  security,
}: {
  onMessage: (message: string | null) => void;
  onUpdated: (security: AccountSecurity) => void;
  providers:
  | Awaited<ReturnType<typeof getAuthProviders>>
  | undefined;
  security: AccountSecurity | undefined;
}) {
  return (
    <section className="space-y-6 border-2 border-border bg-card p-6 shadow-md">
      <div className="space-y-1">
        <h2 className="font-head text-3xl! font-bold">
          Single sign-on
        </h2>
        <p className="text-sm leading-5 text-muted-foreground">
          Connect or remove GitHub and GitLab sign-in for this account.
        </p>
      </div>

      <div className="divide-y-2 divide-border border-2 border-border">
        <SsoProviderRow
          enabled={providers?.sso.github.enabled ?? false}
          provider="github"
          security={security}
          onMessage={onMessage}
          onUpdated={onUpdated}
        />
        <SsoProviderRow
          enabled={providers?.sso.gitlab.enabled ?? false}
          provider="gitlab"
          security={security}
          onMessage={onMessage}
          onUpdated={onUpdated}
        />
      </div>
    </section>
  );
}

function SsoProviderRow({
  enabled,
  onMessage,
  onUpdated,
  provider,
  security,
}: {
  enabled: boolean;
  onMessage: (message: string | null) => void;
  onUpdated: (security: AccountSecurity) => void;
  provider: AccountSsoProvider;
  security: AccountSecurity | undefined;
}) {
  const [error, setError] = useState<string | null>(null);
  const identity = security?.identities.find((item) => item.provider === provider);
  const linked = Boolean(identity);
  const mutation = useMutation({
    mutationFn: removeAccountSso,
    onSuccess: (nextSecurity) => {
      onUpdated(nextSecurity);
      onMessage(`${providerLabel(provider)} SSO removed.`);
      setError(null);
    },
    onError: (error) => {
      setError(settingsErrorMessage(error));
      onMessage(null);
    },
  });

  return (
    <div className="grid gap-4 p-4 sm:grid-cols-[1fr_auto] sm:items-center">
      <div className="min-w-0 space-y-1">
        <div className="flex flex-wrap items-center gap-2">
          <BrandIcon brand={provider} className="size-4" />
          <span className="font-head text-sm font-bold">{providerLabel(provider)}</span>
          <Badge size="sm" variant="surface">
            {linked ? "Connected" : enabled ? "Available" : "Unavailable"}
          </Badge>
        </div>
        <p className="break-words text-sm text-muted-foreground">
          {linked
            ? identity?.email ?? identity?.username ?? "Connected to this account."
            : enabled
              ? `Connect ${providerLabel(provider)} sign-in.`
              : `${providerLabel(provider)} SSO is not configured for this instance.`}
        </p>
        {error ? <p className="text-sm text-destructive">{error}</p> : null}
      </div>

      {linked ? (
        <Button
          disabled={mutation.isPending}
          onClick={() => mutation.mutate(provider)}
          type="button"
          variant="outline"
        >
          <Trash2 className="mr-2 size-4" />
          {mutation.isPending ? "Removing..." : "Remove"}
        </Button>
      ) : (
        <Button
          disabled={!enabled}
          onClick={() => {
            window.location.href = `/api/v1/settings/profile/sso/${provider}/start`;
          }}
          type="button"
          variant="secondary"
        >
          <LinkIcon className="mr-2 size-4" />
          Connect
        </Button>
      )}
    </div>
  );
}

function SettingsField({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  return (
    <label className="block space-y-2">
      <span className="font-head text-sm font-bold">{label}</span>
      {children}
    </label>
  );
}

function providerLabel(provider: AccountSsoProvider): string {
  return provider === "github" ? "GitHub" : "GitLab";
}
