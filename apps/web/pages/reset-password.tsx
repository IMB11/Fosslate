import { useMutation } from "@tanstack/react-query";
import type { FormEvent } from "react";
import { useState } from "react";
import { useRouter } from "next/router";

import {
  AuthCard,
  AuthField,
  AuthMessage,
  AuthPageFrame,
  PasswordPolicy,
} from "@/components/auth/AuthLayout";
import { Button } from "@/components/retroui/Button";
import { Input } from "@/components/retroui/Input";
import { resetPassword } from "@/lib/auth-client";
import {
  resetErrorMessage,
  validatePasswordConfirmation,
  validatePasswordPolicy,
} from "@/lib/auth-validation";

export default function ResetPasswordPage() {
  const router = useRouter();
  const token = typeof router.query.token === "string" ? router.query.token : "";
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [complete, setComplete] = useState(false);

  const resetMutation = useMutation({
    mutationFn: resetPassword,
    onSuccess: () => {
      setComplete(true);
      setError(null);
    },
    onError: (error) => setError(resetErrorMessage(error)),
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const validation =
      validateToken(token) ??
      validatePasswordPolicy(password) ??
      validatePasswordConfirmation(password, confirmation);

    if (validation) {
      setError(validation);
      return;
    }

    resetMutation.mutate({
      token,
      password,
      password_confirmation: confirmation,
    });
  }

  return (
    <AuthPageFrame>
      <AuthCard
        description="Choose a new password for your Fosslate account."
        title="Reset password"
      >
        {error ? <AuthMessage tone="error">{error}</AuthMessage> : null}

        {complete ? (
          <div className="space-y-4">
            <AuthMessage tone="notice">Your password has been reset.</AuthMessage>
            <Button
              className="w-full"
              onClick={() => router.push("/login")}
              type="button"
            >
              Continue
            </Button>
          </div>
        ) : (
          <form className="space-y-4" onSubmit={handleSubmit}>
            <AuthField label="Password">
              <Input
                autoComplete="new-password"
                onChange={(event) => {
                  setPassword(event.target.value);
                  setError(null);
                }}
                placeholder="Enter a password"
                type="password"
                value={password}
              />
            </AuthField>

            <AuthField label="Repeat password">
              <Input
                autoComplete="new-password"
                onChange={(event) => {
                  setConfirmation(event.target.value);
                  setError(null);
                }}
                placeholder="Repeat your password"
                type="password"
                value={confirmation}
              />
            </AuthField>

            <PasswordPolicy />

            <Button
              className="w-full"
              disabled={resetMutation.isPending}
              type="submit"
            >
              {resetMutation.isPending ? "Resetting..." : "Reset password"}
            </Button>
          </form>
        )}
      </AuthCard>
    </AuthPageFrame>
  );
}

function validateToken(token: string): string | null {
  return token.trim() ? null : "Reset token is missing.";
}
