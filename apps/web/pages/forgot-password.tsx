import { useMutation } from "@tanstack/react-query";
import type { FormEvent } from "react";
import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/router";

import {
  AuthCard,
  AuthField,
  AuthMessage,
  AuthPageFrame,
} from "@/components/auth/AuthLayout";
import { Button } from "@/components/retroui/Button";
import { Input } from "@/components/retroui/Input";
import { forgotPassword } from "@/lib/auth-client";
import {
  authErrorMessage,
  validEmail,
} from "@/lib/auth-validation";

export default function ForgotPasswordPage() {
  const router = useRouter();
  const [email, setEmail] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    if (typeof router.query.email === "string") {
      setEmail(router.query.email);
    }
  }, [router.query.email]);

  const forgotMutation = useMutation({
    mutationFn: forgotPassword,
    onSuccess: () => {
      setError(null);
      setNotice("If that account exists, a reset link will be sent.");
    },
    onError: (error) =>
      setError(authErrorMessage(error, "Could not request a reset link.")),
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!validEmail(email)) {
      setError("Enter a valid email address.");
      return;
    }

    forgotMutation.mutate({ email: email.trim() });
  }

  return (
    <AuthPageFrame>
      <AuthCard
        description="Enter your email address to get a password reset link"
        title="Forgot password"
      >
        {error ? <AuthMessage tone="error">{error}</AuthMessage> : null}
        {notice ? <AuthMessage tone="notice">{notice}</AuthMessage> : null}

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

          <Button
            className="w-full"
            disabled={forgotMutation.isPending}
            type="submit"
          >
            {forgotMutation.isPending ? "Sending..." : "Send reset request"}
          </Button>
        </form>

        <div className="text-center font-sans text-sm leading-5">
          Remember your password?{" "}
          <Link className="underline underline-offset-2" href="/login">
            Log in
          </Link>
        </div>
      </AuthCard>
    </AuthPageFrame>
  );
}
