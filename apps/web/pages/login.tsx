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
} from "@/components/auth/AuthLayout";
import { Button } from "@/components/retroui/Button";
import { Input } from "@/components/retroui/Input";
import {
  getAuthProviders,
  login,
} from "@/lib/auth-client";
import {
  authErrorMessage,
  safeRedirectPath,
  validateEmailPassword,
} from "@/lib/auth-validation";

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const redirectTo = safeRedirectPath(router.query.next);

  const providersQuery = useQuery({
    queryKey: ["authProviders"],
    queryFn: getAuthProviders,
  });

  const loginMutation = useMutation({
    mutationFn: login,
    onSuccess: () => router.push(redirectTo),
    onError: (error) => setError(authErrorMessage(error, "Login failed.")),
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const validation = validateEmailPassword(email, password);
    if (validation) {
      setError(validation);
      return;
    }

    loginMutation.mutate({
      email: email.trim(),
      password,
    });
  }

  return (
    <AuthPageFrame>
      <AuthCard
        description="Enter your email below to login to your account"
        title="Login to your account"
      >
        {error ? <AuthMessage tone="error">{error}</AuthMessage> : null}

        {providersQuery.data?.password === false ? (
          <AuthMessage tone="notice">
            Password login is disabled for this instance.
          </AuthMessage>
        ) : (
          <form className="space-y-4" onSubmit={handleSubmit}>
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

            <AuthField
              action={
                <Link
                  className="text-sm underline underline-offset-2"
                  href={{
                    pathname: "/forgot-password",
                    query: email ? { email } : undefined,
                  }}
                >
                  Forgot your password?
                </Link>
              }
              label="Password"
            >
              <Input
                autoComplete="current-password"
                onChange={(event) => {
                  setPassword(event.target.value);
                  setError(null);
                }}
                placeholder="Enter your password"
                type="password"
                value={password}
              />
            </AuthField>

            <Button
              className="w-full"
              disabled={loginMutation.isPending}
              type="submit"
            >
              {loginMutation.isPending ? "Logging in..." : "Login"}
            </Button>
          </form>
        )}

        <AuthSsoButtons
          providers={providersQuery.data}
          redirectTo={redirectTo}
        />

        <div className="text-center font-sans text-sm leading-5">
          Don&apos;t have an account?{" "}
          <Link className="underline underline-offset-2" href="/signup">
            Sign up
          </Link>
        </div>
      </AuthCard>
    </AuthPageFrame>
  );
}
