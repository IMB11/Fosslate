import { Field as BaseField } from "@base-ui/react/field";
import type { ReactNode } from "react";

import { Button } from "@/components/retroui/Button";
import { Card } from "@/components/retroui/Card";
import { Label } from "@/components/retroui/Label";
import { Text } from "@/components/retroui/Text";
import { FosslateLogo } from "@/components/setup/FosslateLogo";
import type { AuthProviderAvailability, AuthProviders } from "@/lib/auth-client";

export function AuthPageFrame({ children }: { children: ReactNode }) {
  return (
    <main className="flex min-h-screen items-center justify-center bg-accent px-4 py-10 text-foreground">
      <div className="flex w-full max-w-[390px] flex-col items-center gap-4">
        <FosslateLogo />
        {children}
      </div>
    </main>
  );
}

export function AuthCard({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <Card className="w-full hover:shadow-md">
      <Card.Content className="space-y-6 p-6">
        <div className="space-y-2">
          <h1 className="font-head text-base font-bold leading-5">{title}</h1>
          <Text className="text-sm leading-5">{description}</Text>
        </div>
        {children}
      </Card.Content>
    </Card>
  );
}

export function AuthField({
  label,
  action,
  children,
}: {
  label: string;
  action?: ReactNode;
  children: ReactNode;
}) {
  return (
    <BaseField.Root className="space-y-2">
      <div className="flex min-h-4 items-center justify-between gap-3">
        <Label className="font-head text-sm font-bold">{label}</Label>
        {action}
      </div>
      {children}
    </BaseField.Root>
  );
}

export function AuthMessage({
  tone,
  children,
}: {
  tone: "error" | "notice";
  children: ReactNode;
}) {
  return (
    <div
      className={
        tone === "error"
          ? "border-2 border-destructive bg-background px-3 py-2 font-sans text-sm leading-5 text-destructive"
          : "border-2 border-border bg-background px-3 py-2 font-sans text-sm leading-5"
      }
    >
      {children}
    </div>
  );
}

export function PasswordPolicy() {
  return (
    <Text className="text-sm leading-5">
      Password must be a minimum of 8 characters, contain 1 uppercase character
      and 1 special character.
    </Text>
  );
}

export function AuthSsoButtons({
  providers,
  redirectTo,
}: {
  providers: AuthProviders | undefined;
  redirectTo: string;
}) {
  const buttons = [
    providerButton("GitHub", providers?.sso.github, redirectTo),
    providerButton("GitLab", providers?.sso.gitlab, redirectTo),
  ].filter((button): button is { label: string; url: string } => Boolean(button));

  if (buttons.length === 0) {
    return null;
  }

  return (
    <div className="space-y-2">
      <Text className="text-center text-sm leading-5">or</Text>
      <div
        className={
          buttons.length === 1 ? "grid grid-cols-1 gap-3" : "grid grid-cols-2 gap-3"
        }
      >
        {buttons.map((button) => (
          <Button
            className="w-full"
            key={button.label}
            onClick={() => window.location.assign(button.url)}
            type="button"
            variant="outline"
          >
            {button.label}
          </Button>
        ))}
      </div>
    </div>
  );
}

function providerButton(
  label: string,
  provider: AuthProviderAvailability | undefined,
  redirectTo: string,
): { label: string; url: string } | null {
  if (!provider?.enabled || !provider.start_url) {
    return null;
  }

  const separator = provider.start_url.includes("?") ? "&" : "?";
  const url = `${provider.start_url}${separator}redirect_to=${encodeURIComponent(
    redirectTo,
  )}`;

  return { label, url };
}
