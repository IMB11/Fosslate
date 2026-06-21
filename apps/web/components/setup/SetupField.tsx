import { Field as BaseField } from "@base-ui/react/field";
import type { ReactNode } from "react";

import { Label } from "@/components/retroui/Label";

export function SetupField({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <BaseField.Root className="space-y-2">
      <Label className="font-head text-sm font-bold">{label}</Label>
      {children}
    </BaseField.Root>
  );
}

