import { Check, Copy } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/retroui/Button";
import { SetupField } from "@/components/setup/SetupField";

export function CallbackUrl({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);

  async function copyCallbackUrl() {
    await window.navigator.clipboard.writeText(value);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  }

  return (
    <SetupField label="Callback URL">
      <div className="flex gap-2">
        <code className="min-w-0 flex-1 truncate rounded border-2 border-border bg-background px-3 py-2 text-sm">
          {value}
        </code>
        <Button
          aria-label="Copy callback URL"
          className="size-10 bg-background p-0"
          onClick={copyCallbackUrl}
          size="icon"
          type="button"
          variant="outline"
        >
          {copied ? <Check size={18} strokeWidth={2.25} /> : <Copy size={18} strokeWidth={2.25} />}
        </Button>
      </div>
    </SetupField>
  );
}
