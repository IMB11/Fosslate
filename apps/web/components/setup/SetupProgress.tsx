import { Text } from "@/components/retroui/Text";
import { isStepDone } from "@/components/setup/setup-utils";
import type { SetupStatus, SetupStep } from "@/lib/setup-types";
import { cn } from "@/lib/utils";

const steps: { id: SetupStep; label: string }[] = [
  { id: "github", label: "GitHub" },
  { id: "gitlab", label: "GitLab" },
  { id: "email", label: "Email" },
  { id: "complete", label: "Finish" },
];

export function SetupProgress({
  activeStep,
  status,
}: {
  activeStep: SetupStep;
  status?: SetupStatus;
}) {
  const activeIndex = steps.findIndex((step) => step.id === activeStep);

  return (
    <div className="grid grid-cols-4 gap-2 border-t-2 border-border pt-5">
      {steps.map((step, index) => {
        const done = isStepDone(step.id, status) || index < activeIndex;
        const active = step.id === activeStep;

        return (
          <div className="min-w-0 space-y-1" key={step.id}>
            <div
              className={cn(
                "h-2 rounded border-2 border-border bg-background",
                done || active ? "bg-primary" : "bg-background",
              )}
            />
            <Text className="truncate text-xs">{step.label}</Text>
          </div>
        );
      })}
    </div>
  );
}

