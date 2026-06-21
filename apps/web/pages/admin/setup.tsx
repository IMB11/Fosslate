import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { FormEvent } from "react";
import { useEffect, useState } from "react";

import { Text } from "@/components/retroui/Text";
import { EmailStep } from "@/components/setup/EmailStep";
import { FosslateLogo } from "@/components/setup/FosslateLogo";
import { SecretStep } from "@/components/setup/SecretStep";
import { SetupCard } from "@/components/setup/SetupCard";
import { SetupProgress } from "@/components/setup/SetupProgress";
import { SsoStep } from "@/components/setup/SsoStep";
import {
  activeSetupStep,
  descriptionForStep,
  errorMessage,
  SECRET_STORAGE_KEY,
  titleForStep,
} from "@/components/setup/setup-utils";
import {
  completeSetup,
  getSetupStatus,
  SetupClientError,
  verifySetupSecret,
} from "@/lib/setup-client";
import type { SetupStatus } from "@/lib/setup-types";

export default function AdminSetupPage() {
  const queryClient = useQueryClient();
  const [secret, setSecret] = useState("");
  const [verifiedSecret, setVerifiedSecret] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const storedSecret = window.sessionStorage.getItem(SECRET_STORAGE_KEY);
    if (storedSecret) {
      setSecret(storedSecret);
      setVerifiedSecret(storedSecret);
    }
  }, []);

  const statusQuery = useQuery({
    queryKey: ["setupStatus", verifiedSecret],
    queryFn: () => getSetupStatus(verifiedSecret),
    enabled: Boolean(verifiedSecret),
  });

  useEffect(() => {
    const error = statusQuery.error;
    if (!(error instanceof SetupClientError)) {
      return;
    }

    if (error.status === 401) {
      window.sessionStorage.removeItem(SECRET_STORAGE_KEY);
      setVerifiedSecret("");
      setError(
        "That setup code is no longer valid. Enter the current code from the API logs.",
      );
    }

    if (error.status === 409) {
      window.sessionStorage.removeItem(SECRET_STORAGE_KEY);
      window.location.assign("/");
    }
  }, [statusQuery.error]);

  const verifyMutation = useMutation({
    mutationFn: verifySetupSecret,
    onSuccess: (status, submittedSecret) => {
      window.sessionStorage.setItem(SECRET_STORAGE_KEY, submittedSecret);
      setVerifiedSecret(submittedSecret);
      setError(null);
      queryClient.setQueryData(["setupStatus", submittedSecret], status);
    },
    onError: (error) => setError(errorMessage(error)),
  });

  const completeMutation = useMutation({
    mutationFn: () => completeSetup(verifiedSecret),
    onSuccess: ({ next }) => {
      window.sessionStorage.removeItem(SECRET_STORAGE_KEY);
      window.location.assign(next || "/");
    },
    onError: (error) => setError(errorMessage(error)),
  });

  const status = statusQuery.data;
  const activeStep = activeSetupStep(status);

  function updateStatus(status: SetupStatus) {
    setError(null);
    queryClient.setQueryData(["setupStatus", verifiedSecret], status);
  }

  function handleVerify(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmedSecret = secret.trim();
    if (!trimmedSecret) {
      setError("Enter the setup code from the API logs.");
      return;
    }

    verifyMutation.mutate(trimmedSecret);
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-accent px-4 py-10 text-foreground">
      <div className="flex w-full max-w-[380px] flex-col items-center gap-4">
        <FosslateLogo />

        {!verifiedSecret ? (
          <SetupCard
            title="First time setup"
            description="Enter the secret code printed in the API logs to unlock setup for this Fosslate instance."
            error={error}
          >
            <SecretStep
              onSecretChange={setSecret}
              onSubmit={handleVerify}
              secret={secret}
              submitting={verifyMutation.isPending}
            />
          </SetupCard>
        ) : (
          <SetupCard
            title={titleForStep(activeStep)}
            description={descriptionForStep(activeStep)}
            error={error}
            footer={<SetupProgress activeStep={activeStep} status={status} />}
          >
            {statusQuery.isLoading ? (
              <Text className="text-sm">Loading setup status...</Text>
            ) : null}

            {status && activeStep === "github" ? (
              <SsoStep
                onError={setError}
                onSuccess={updateStatus}
                provider="github"
                secret={verifiedSecret}
                status={status}
              />
            ) : null}

            {status && activeStep === "gitlab" ? (
              <SsoStep
                onError={setError}
                onSuccess={updateStatus}
                provider="gitlab"
                secret={verifiedSecret}
                status={status}
              />
            ) : null}

            {status && activeStep === "email" ? (
              <EmailStep
                onComplete={() => completeMutation.mutate()}
                onError={setError}
                onSuccess={updateStatus}
                secret={verifiedSecret}
                status={status}
                submittingComplete={completeMutation.isPending}
              />
            ) : null}
          </SetupCard>
        )}
      </div>
    </main>
  );
}
