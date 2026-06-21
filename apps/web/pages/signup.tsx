import { useMutation, useQuery } from "@tanstack/react-query";
import type { FormEvent } from "react";
import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/router";

import {
  AuthCard,
  AuthField,
  AuthMessage,
  AuthPageFrame,
  AuthSsoButtons,
  PasswordPolicy,
} from "@/components/auth/AuthLayout";
import { Button } from "@/components/retroui/Button";
import { Input } from "@/components/retroui/Input";
import {
  completeSignup,
  getAuthProviders,
  startSignup,
} from "@/lib/auth-client";
import {
  authErrorMessage,
  safeRedirectPath,
  validateEmailPassword,
  validatePasswordConfirmation,
  validatePasswordPolicy,
} from "@/lib/auth-validation";

type SignupStep = "details" | "verify";

export default function SignupPage() {
  const router = useRouter();
  const [step, setStep] = useState<SignupStep>("details");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [passwordConfirmation, setPasswordConfirmation] = useState("");
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const redirectTo = safeRedirectPath(router.query.next);

  const providersQuery = useQuery({
    queryKey: ["authProviders"],
    queryFn: getAuthProviders,
  });

  const startMutation = useMutation({
    mutationFn: startSignup,
    onSuccess: () => {
      setStep("verify");
      setError(null);
      setNotice("Check your email for the 6-digit verification code.");
    },
    onError: (error) =>
      setError(authErrorMessage(error, "Could not start signup.")),
  });

  const completeMutation = useMutation({
    mutationFn: completeSignup,
    onSuccess: () => router.push(redirectTo),
    onError: (error) =>
      setError(authErrorMessage(error, "Could not complete signup.")),
  });

  function handleStart(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const validation =
      validateEmailPassword(email, password) ??
      validatePasswordPolicy(password) ??
      validatePasswordConfirmation(password, passwordConfirmation);

    if (validation) {
      setError(validation);
      return;
    }

    startMutation.mutate({
      email: email.trim(),
      password,
    });
  }

  function handleComplete(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!/^\d{6}$/.test(code.trim())) {
      setError("Enter the 6-digit code from your email.");
      return;
    }

    completeMutation.mutate({
      email: email.trim(),
      password,
      code: code.trim(),
    });
  }

  return (
    <AuthPageFrame>
      <AuthCard
        description={
          step === "details"
            ? "Create an account to start translating."
            : "Enter the code we sent to finish creating your account."
        }
        title={step === "details" ? "Sign up" : "Verify your email"}
      >
        {error ? <AuthMessage tone="error">{error}</AuthMessage> : null}
        {notice ? <AuthMessage tone="notice">{notice}</AuthMessage> : null}

        {providersQuery.data?.password === false ? (
          <AuthMessage tone="notice">
            Password signup is disabled for this instance.
          </AuthMessage>
        ) : step === "details" ? (
          <form className="space-y-4" onSubmit={handleStart}>
            <AuthField label="Email">
              <Input
                autoComplete="email"
                onChange={(event) => {
                  setEmail(event.target.value);
                  setError(null);
                }}
                placeholder="john.doe@example.com"
                type="email"
                value={email}
              />
            </AuthField>

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
                  setPasswordConfirmation(event.target.value);
                  setError(null);
                }}
                placeholder="Repeat your password"
                type="password"
                value={passwordConfirmation}
              />
            </AuthField>

            <PasswordPolicy />

            <Button
              className="w-full"
              disabled={startMutation.isPending}
              type="submit"
            >
              {startMutation.isPending ? "Sending code..." : "Sign up"}
            </Button>
          </form>
        ) : (
          <form className="space-y-4" onSubmit={handleComplete}>
            <AuthField label="Verification code">
              <Input
                autoComplete="one-time-code"
                inputMode="numeric"
                maxLength={6}
                onChange={(event) => {
                  setCode(event.target.value);
                  setError(null);
                }}
                placeholder="000000"
                value={code}
              />
            </AuthField>

            <div className="flex gap-3">
              <Button
                className="flex-1"
                onClick={() => {
                  setStep("details");
                  setNotice(null);
                  setError(null);
                }}
                type="button"
                variant="outline"
              >
                Back
              </Button>
              <Button
                className="flex-1"
                disabled={completeMutation.isPending}
                type="submit"
              >
                {completeMutation.isPending ? "Verifying..." : "Verify"}
              </Button>
            </div>
          </form>
        )}

        {step === "details" ? (
          <AuthSsoButtons
            providers={providersQuery.data}
            redirectTo={redirectTo}
          />
        ) : null}

        <div className="text-center font-sans text-sm leading-5">
          Already have an account?{" "}
          <Link className="underline underline-offset-2" href="/login">
            Log in
          </Link>
        </div>
      </AuthCard>
    </AuthPageFrame>
  );
}
