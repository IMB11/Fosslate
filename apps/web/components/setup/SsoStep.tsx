import { useMutation } from "@tanstack/react-query";
import type { FormEvent } from "react";
import { useState } from "react";

import { Button } from "@/components/retroui/Button";
import { Input } from "@/components/retroui/Input";
import { CallbackUrl } from "@/components/setup/CallbackUrl";
import { SetupField } from "@/components/setup/SetupField";
import { errorMessage, type Provider } from "@/components/setup/setup-utils";
import {
  saveGithubSsoSetup,
  saveGitlabSsoSetup,
} from "@/lib/setup-client";
import type { SaveSsoProviderRequest, SetupStatus } from "@/lib/setup-types";

export function SsoStep({
  provider,
  secret,
  status,
  onError,
  onSuccess,
}: {
  provider: Provider;
  secret: string;
  status: SetupStatus;
  onError: (message: string | null) => void;
  onSuccess: (status: SetupStatus) => void;
}) {
  const providerStatus = status[provider];
  const [clientId, setClientId] = useState(providerStatus.client_id ?? "");
  const [clientSecret, setClientSecret] = useState("");
  const [baseUrl, setBaseUrl] = useState(providerStatus.base_url ?? "https://gitlab.com");

  const saveMutation = useMutation({
    mutationFn: (body: SaveSsoProviderRequest) =>
      provider === "github"
        ? saveGithubSsoSetup(secret, body)
        : saveGitlabSsoSetup(secret, body),
    onSuccess,
    onError: (error) => onError(errorMessage(error)),
  });

  const providerName = provider === "github" ? "GitHub" : "GitLab";
  const canContinue =
    clientId.trim().length > 0 &&
    clientSecret.trim().length > 0 &&
    (provider === "github" || baseUrl.trim().length > 0);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canContinue) {
      onError(`Enter the ${providerName} OAuth credentials or skip this step.`);
      return;
    }

    saveMutation.mutate({
      enabled: true,
      client_id: clientId.trim(),
      client_secret: clientSecret.trim(),
      ...(provider === "gitlab" ? { base_url: baseUrl.trim() } : {}),
    });
  }

  return (
    <form className="space-y-5" onSubmit={handleSubmit}>
      <CallbackUrl value={providerStatus.callback_url} />

      {provider === "gitlab" ? (
        <SetupField label="GitLab base URL">
          <Input
            className="h-10 bg-background"
            onChange={(event) => setBaseUrl(event.target.value)}
            placeholder="https://gitlab.com"
            value={baseUrl}
          />
        </SetupField>
      ) : null}

      <SetupField label="Client ID">
        <Input
          autoComplete="off"
          className="h-10 bg-background"
          onChange={(event) => setClientId(event.target.value)}
          placeholder={`Enter the ${providerName} client ID`}
          value={clientId}
        />
      </SetupField>

      <SetupField label="Client secret">
        <Input
          autoComplete="off"
          className="h-10 bg-background"
          onChange={(event) => setClientSecret(event.target.value)}
          placeholder={
            providerStatus.has_client_secret ? "Saved secret unchanged" : "Enter the client secret"
          }
          type="password"
          value={clientSecret}
        />
      </SetupField>

      <div className="grid gap-3">
        <Button
          className="h-10 w-full bg-primary"
          disabled={saveMutation.isPending}
          type="submit"
        >
          {saveMutation.isPending ? "Saving..." : "Continue"}
        </Button>
        <Button
          className="h-10 w-full bg-background"
          disabled={saveMutation.isPending}
          onClick={() => saveMutation.mutate({ enabled: false })}
          type="button"
          variant="outline"
        >
          Skip
        </Button>
      </div>
    </form>
  );
}

