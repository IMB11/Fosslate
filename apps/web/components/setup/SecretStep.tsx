import type { FormEvent } from "react";

import { Button } from "@/components/retroui/Button";
import { Input } from "@/components/retroui/Input";
import { SetupField } from "@/components/setup/SetupField";

export function SecretStep({
  secret,
  submitting,
  onSecretChange,
  onSubmit,
}: {
  secret: string;
  submitting: boolean;
  onSecretChange: (secret: string) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
}) {
  return (
    <form className="space-y-6" onSubmit={onSubmit}>
      <SetupField label="Secret code">
        <Input
          autoComplete="off"
          className="h-10 bg-background"
          onChange={(event) => onSecretChange(event.target.value)}
          placeholder="fs_setup_..."
          type="password"
          value={secret}
        />
      </SetupField>
      <Button className="h-10 w-full bg-primary" disabled={submitting} type="submit">
        {submitting ? "Checking..." : "Continue"}
      </Button>
    </form>
  );
}

