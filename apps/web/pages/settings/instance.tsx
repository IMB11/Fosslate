import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { KeyRound, Lock, Mail, Save, ShieldCheck } from "lucide-react";
import type { FormEvent, ReactNode } from "react";
import { useEffect, useState } from "react";

import { Alert } from "@/components/retroui/Alert";
import { Badge } from "@/components/retroui/Badge";
import { Button } from "@/components/retroui/Button";
import { Input } from "@/components/retroui/Input";
import { Text } from "@/components/retroui/Text";
import { SettingsLayout } from "@/components/settings/SettingsLayout";
import {
  getOptionalAuthSession,
  type AuthUser,
} from "@/lib/auth-client";
import {
  claimInstanceAdmin,
  getInstanceSettings,
  saveInstanceSsoProvider,
  settingsErrorMessage,
  testInstanceEmailDelivery,
  type InstanceSettings,
} from "@/lib/settings-client";
import type {
  AuthProviderSetupStatus,
  EmailDeliverySetupStatus,
} from "@/lib/setup-types";

type InstanceSettingsPageProps = {
  initialAuthUser?: AuthUser | null;
};

type Provider = "github" | "gitlab";

export default function InstanceSettingsPage({
  initialAuthUser,
}: InstanceSettingsPageProps) {
  const queryClient = useQueryClient();
  const [adminSecret, setAdminSecret] = useState("");
  const [claimError, setClaimError] = useState<string | null>(null);

  const sessionQuery = useQuery({
    queryKey: ["authSession"],
    queryFn: getOptionalAuthSession,
    initialData: initialAuthUser,
  });
  const user = sessionQuery.data ?? null;
  const isAdmin = Boolean(user?.is_admin);

  const settingsQuery = useQuery({
    queryKey: ["instanceSettings"],
    queryFn: getInstanceSettings,
    enabled: isAdmin,
  });

  const claimMutation = useMutation({
    mutationFn: claimInstanceAdmin,
    onSuccess: (user) => {
      setAdminSecret("");
      setClaimError(null);
      queryClient.setQueryData(["authSession"], user);
      void queryClient.invalidateQueries({ queryKey: ["instanceSettings"] });
    },
    onError: (error) => setClaimError(settingsErrorMessage(error)),
  });

  function handleClaim(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmedSecret = adminSecret.trim();
    if (!trimmedSecret) {
      setClaimError("Enter the current admin code from the backend logs.");
      return;
    }

    claimMutation.mutate(trimmedSecret);
  }

  return (
    <SettingsLayout>
      <div className="max-w-4xl space-y-6">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="space-y-2">
            <Text as="h1" className="text-3xl">
              Instance
            </Text>
          </div>
          {isAdmin ? (
            <Badge className="w-fit gap-2" variant="surface">
              <ShieldCheck aria-hidden="true" className="size-4" />
              Admin
            </Badge>
          ) : null}
        </div>

        {!user ? (
          <Alert status="warning">
            <Alert.Description className="text-yellow-900">
              Sign in before changing instance settings.
            </Alert.Description>
          </Alert>
        ) : null}

        {user && !isAdmin ? (
          <section className="border-2 border-border bg-card p-5 shadow-md">
            <div className="mb-5 flex items-start gap-3">
              <div className="flex size-10 shrink-0 items-center justify-center border-2 border-border bg-primary text-primary-foreground">
                <Lock aria-hidden="true" className="size-5" />
              </div>
              <div className="space-y-1">
                <h2 className="font-head text-xl font-bold">Locked</h2>
                <p className="text-sm text-muted-foreground">
                  Enter the current backend log code to claim admin access.
                </p>
              </div>
            </div>

            {claimError ? (
              <Alert className="mb-5" status="error">
                <Alert.Description className="text-red-900">
                  {claimError}
                </Alert.Description>
              </Alert>
            ) : null}

            <form className="grid gap-3 sm:grid-cols-[1fr_auto]" onSubmit={handleClaim}>
              <Input
                autoComplete="off"
                className="h-11 bg-background"
                onChange={(event) => setAdminSecret(event.target.value)}
                placeholder="fs_setup_..."
                type="password"
                value={adminSecret}
              />
              <Button
                className="h-11 gap-2"
                disabled={claimMutation.isPending}
                type="submit"
              >
                <KeyRound aria-hidden="true" className="size-4" />
                {claimMutation.isPending ? "Checking..." : "Unlock"}
              </Button>
            </form>
          </section>
        ) : null}

        {isAdmin ? (
          <AdminInstanceSettings
            isLoading={settingsQuery.isLoading}
            settings={settingsQuery.data}
          />
        ) : null}
      </div>
    </SettingsLayout>
  );
}

function AdminInstanceSettings({
  isLoading,
  settings,
}: {
  isLoading: boolean;
  settings?: InstanceSettings;
}) {
  const queryClient = useQueryClient();

  function updateSettings(settings: InstanceSettings) {
    queryClient.setQueryData(["instanceSettings"], settings);
  }

  if (isLoading) {
    return <p className="text-sm text-muted-foreground">Loading settings...</p>;
  }

  if (!settings) {
    return null;
  }

  return (
    <div className="space-y-5">
      <SsoSettingsForm
        onSaved={updateSettings}
        provider="github"
        status={settings.github}
      />
      <SsoSettingsForm
        onSaved={updateSettings}
        provider="gitlab"
        status={settings.gitlab}
      />
      <EmailSettingsForm onSaved={updateSettings} status={settings.email} />
    </div>
  );
}

function SsoSettingsForm({
  provider,
  status,
  onSaved,
}: {
  provider: Provider;
  status: AuthProviderSetupStatus;
  onSaved: (settings: InstanceSettings) => void;
}) {
  const providerName = provider === "github" ? "GitHub" : "GitLab";
  const [enabled, setEnabled] = useState(status.enabled);
  const [clientId, setClientId] = useState(status.client_id ?? "");
  const [clientSecret, setClientSecret] = useState("");
  const [baseUrl, setBaseUrl] = useState(status.base_url ?? "https://gitlab.com");
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setEnabled(status.enabled);
    setClientId(status.client_id ?? "");
    setClientSecret("");
    setBaseUrl(status.base_url ?? "https://gitlab.com");
  }, [status]);

  const saveMutation = useMutation({
    mutationFn: () =>
      saveInstanceSsoProvider(provider, {
        enabled,
        ...(enabled
          ? {
            client_id: clientId.trim(),
            ...(clientSecret.trim()
              ? { client_secret: clientSecret.trim() }
              : {}),
            ...(provider === "gitlab" ? { base_url: baseUrl.trim() } : {}),
          }
          : {}),
      }),
    onSuccess: (settings) => {
      setError(null);
      setMessage(`${providerName} settings saved.`);
      onSaved(settings);
    },
    onError: (error) => {
      setMessage(null);
      setError(settingsErrorMessage(error));
    },
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (enabled && (!clientId.trim() || (!clientSecret.trim() && !status.has_client_secret))) {
      setMessage(null);
      setError(`Enter the ${providerName} client ID and client secret.`);
      return;
    }

    saveMutation.mutate();
  }

  return (
    <section className="border-2 border-border bg-card p-5 shadow-md">
      <form className="space-y-5" onSubmit={handleSubmit}>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="space-y-1">
            <h2 className="font-head text-xl font-bold">{providerName} OAuth</h2>
            <p className="text-sm text-muted-foreground">
              Callback URL:{" "}
              <code className="break-all rounded border-2 border-border bg-background px-1 py-0.5 text-xs">
                {status.callback_url}
              </code>
            </p>
          </div>
          <label className="flex w-fit items-center gap-2 font-head text-sm font-bold">
            <input
              checked={enabled}
              className="size-5 accent-primary"
              onChange={(event) => setEnabled(event.target.checked)}
              type="checkbox"
            />
            Enabled
          </label>
        </div>

        {message ? (
          <Alert status="success">
            <Alert.Description className="text-green-900">
              {message}
            </Alert.Description>
          </Alert>
        ) : null}

        {error ? (
          <Alert status="error">
            <Alert.Description className="text-red-900">{error}</Alert.Description>
          </Alert>
        ) : null}

        {enabled ? (
          <div className="grid gap-4">
            {provider === "gitlab" ? (
              <SettingsField label="GitLab base URL">
                <Input
                  className="h-10 bg-background"
                  onChange={(event) => setBaseUrl(event.target.value)}
                  placeholder="https://gitlab.com"
                  value={baseUrl}
                />
              </SettingsField>
            ) : null}

            <SettingsField label="Client ID">
              <Input
                autoComplete="off"
                className="h-10 bg-background"
                onChange={(event) => setClientId(event.target.value)}
                placeholder={`Enter the ${providerName} client ID`}
                value={clientId}
              />
            </SettingsField>

            <SettingsField label="Client secret">
              <Input
                autoComplete="off"
                className="h-10 bg-background"
                onChange={(event) => setClientSecret(event.target.value)}
                placeholder={
                  status.has_client_secret ? "Saved secret unchanged" : "Enter the client secret"
                }
                type="password"
                value={clientSecret}
              />
            </SettingsField>
          </div>
        ) : null}

        <Button
          className="h-10 gap-2"
          disabled={saveMutation.isPending}
          type="submit"
        >
          <Save aria-hidden="true" className="size-4" />
          {saveMutation.isPending ? "Saving..." : "Save"}
        </Button>
      </form>
    </section>
  );
}

function EmailSettingsForm({
  status,
  onSaved,
}: {
  status: EmailDeliverySetupStatus;
  onSaved: (settings: InstanceSettings) => void;
}) {
  const [resendApiKey, setResendApiKey] = useState("");
  const [fromName, setFromName] = useState(status.from_name ?? "Fosslate");
  const [fromEmail, setFromEmail] = useState(status.from_email ?? "");
  const [testRecipient, setTestRecipient] = useState(
    status.last_test_recipient ?? "",
  );
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setResendApiKey("");
    setFromName(status.from_name ?? "Fosslate");
    setFromEmail(status.from_email ?? "");
    setTestRecipient(status.last_test_recipient ?? "");
  }, [status]);

  const testMutation = useMutation({
    mutationFn: () =>
      testInstanceEmailDelivery({
        ...(resendApiKey.trim() ? { resend_api_key: resendApiKey.trim() } : {}),
        from_name: fromName.trim(),
        from_email: fromEmail.trim(),
        test_recipient: testRecipient.trim(),
      }),
    onSuccess: (settings) => {
      setError(null);
      setMessage("Test email sent and settings saved.");
      onSaved(settings);
    },
    onError: (error) => {
      setMessage(null);
      setError(settingsErrorMessage(error));
    },
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (
      (!resendApiKey.trim() && !status.has_api_key) ||
      !fromName.trim() ||
      !fromEmail.trim() ||
      !testRecipient.trim()
    ) {
      setMessage(null);
      setError("Enter the Resend key, sender, and test recipient.");
      return;
    }

    testMutation.mutate();
  }

  return (
    <section className="border-2 border-border bg-card p-5 shadow-md">
      <form className="space-y-5" onSubmit={handleSubmit}>
        <div className="flex items-start gap-3">
          <div className="flex size-10 shrink-0 items-center justify-center border-2 border-border bg-primary text-primary-foreground">
            <Mail aria-hidden="true" className="size-5" />
          </div>
          <div className="space-y-1">
            <h2 className="font-head text-xl font-bold">Resend email</h2>
            <p className="text-sm text-muted-foreground">
              Send a test email before saving delivery settings.
            </p>
          </div>
        </div>

        {message ? (
          <Alert status="success">
            <Alert.Description className="text-green-900">
              {message}
            </Alert.Description>
          </Alert>
        ) : null}

        {error ? (
          <Alert status="error">
            <Alert.Description className="text-red-900">{error}</Alert.Description>
          </Alert>
        ) : null}

        <div className="grid gap-4">
          <SettingsField label="Resend API key">
            <Input
              autoComplete="off"
              className="h-10 bg-background"
              onChange={(event) => setResendApiKey(event.target.value)}
              placeholder={status.has_api_key ? "Saved key unchanged" : "re_..."}
              type="password"
              value={resendApiKey}
            />
          </SettingsField>

          <SettingsField label="From name">
            <Input
              className="h-10 bg-background"
              onChange={(event) => setFromName(event.target.value)}
              placeholder="Fosslate"
              value={fromName}
            />
          </SettingsField>

          <SettingsField label="From email">
            <Input
              className="h-10 bg-background"
              onChange={(event) => setFromEmail(event.target.value)}
              placeholder="hello@example.com"
              type="email"
              value={fromEmail}
            />
          </SettingsField>

          <SettingsField label="Test recipient">
            <Input
              className="h-10 bg-background"
              onChange={(event) => setTestRecipient(event.target.value)}
              placeholder="admin@example.com"
              type="email"
              value={testRecipient}
            />
          </SettingsField>
        </div>

        <Button
          className="h-10 gap-2"
          disabled={testMutation.isPending}
          type="submit"
        >
          <Mail aria-hidden="true" className="size-4" />
          {testMutation.isPending ? "Sending..." : "Send test and save"}
        </Button>
      </form>
    </section>
  );
}

function SettingsField({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <label className="grid gap-2">
      <span className="font-head text-sm font-bold">{label}</span>
      {children}
    </label>
  );
}
