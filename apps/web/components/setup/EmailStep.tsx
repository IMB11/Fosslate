import { useMutation } from "@tanstack/react-query";
import type { FormEvent } from "react";
import { useState } from "react";

import { Alert } from "@/components/retroui/Alert";
import { Button } from "@/components/retroui/Button";
import { Input } from "@/components/retroui/Input";
import { SetupField } from "@/components/setup/SetupField";
import { errorMessage } from "@/components/setup/setup-utils";
import { testEmailDeliverySetup } from "@/lib/setup-client";
import type { SetupStatus, TestEmailDeliveryRequest } from "@/lib/setup-types";

export function EmailStep({
  secret,
  status,
  submittingComplete,
  onComplete,
  onError,
  onSuccess,
}: {
  secret: string;
  status: SetupStatus;
  submittingComplete: boolean;
  onComplete: () => void;
  onError: (message: string | null) => void;
  onSuccess: (status: SetupStatus) => void;
}) {
  const [resendApiKey, setResendApiKey] = useState("");
  const [fromName, setFromName] = useState(status.email.from_name ?? "Fosslate");
  const [fromEmail, setFromEmail] = useState(status.email.from_email ?? "");
  const [testRecipient, setTestRecipient] = useState(
    status.email.last_test_recipient ?? "",
  );

  const testMutation = useMutation({
    mutationFn: (body: TestEmailDeliveryRequest) =>
      testEmailDeliverySetup(secret, body),
    onSuccess,
    onError: (error) => onError(errorMessage(error)),
  });

  const canTest =
    resendApiKey.trim().length > 0 &&
    fromName.trim().length > 0 &&
    fromEmail.trim().length > 0 &&
    testRecipient.trim().length > 0;

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canTest) {
      onError("Enter the Resend key, sender, and test recipient before sending a test email.");
      return;
    }

    testMutation.mutate({
      resend_api_key: resendApiKey.trim(),
      from_name: fromName.trim(),
      from_email: fromEmail.trim(),
      test_recipient: testRecipient.trim(),
    });
  }

  return (
    <form className="space-y-5" onSubmit={handleSubmit}>
      {status.email.configured ? (
        <Alert status="success">
          <Alert.Title className="text-base font-bold leading-5 text-green-900">
            Email test passed
          </Alert.Title>
          <Alert.Description className="mt-2 text-sm leading-5 text-green-900">
            Fosslate can send setup email through Resend.
          </Alert.Description>
        </Alert>
      ) : null}

      <SetupField label="Resend API key">
        <Input
          autoComplete="off"
          className="h-10 bg-background"
          onChange={(event) => setResendApiKey(event.target.value)}
          placeholder="re_..."
          type="password"
          value={resendApiKey}
        />
      </SetupField>

      <SetupField label="From name">
        <Input
          className="h-10 bg-background"
          onChange={(event) => setFromName(event.target.value)}
          placeholder="Fosslate"
          value={fromName}
        />
      </SetupField>

      <SetupField label="From email">
        <Input
          className="h-10 bg-background"
          onChange={(event) => setFromEmail(event.target.value)}
          placeholder="hello@example.com"
          type="email"
          value={fromEmail}
        />
      </SetupField>

      <SetupField label="Test recipient">
        <Input
          className="h-10 bg-background"
          onChange={(event) => setTestRecipient(event.target.value)}
          placeholder="admin@example.com"
          type="email"
          value={testRecipient}
        />
      </SetupField>

      <div className="grid gap-3">
        <Button
          className="h-10 w-full bg-background"
          disabled={testMutation.isPending}
          type="submit"
          variant="outline"
        >
          {testMutation.isPending ? "Sending..." : "Send test email"}
        </Button>
        <Button
          className="h-10 w-full bg-primary"
          disabled={!status.email.configured || submittingComplete}
          onClick={onComplete}
          type="button"
        >
          {submittingComplete ? "Completing..." : "Complete setup"}
        </Button>
      </div>
    </form>
  );
}
