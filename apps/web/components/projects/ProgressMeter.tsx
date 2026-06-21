import { cn } from "@/lib/utils";

type ProgressMeterProps = {
  approved: number;
  className?: string;
  translated: number;
};

export function ProgressMeter({
  approved,
  className,
  translated,
}: ProgressMeterProps) {
  const translatedWidth = clampPercent(translated);
  const approvedWidth = clampPercent(Math.min(approved, translated));

  return (
    <div
      aria-label={`${translatedWidth}% translated, ${approvedWidth}% approved`}
      className={cn(
        "relative h-4 w-full overflow-hidden border-2 border-border bg-background",
        className,
      )}
      role="img"
    >
      <div
        className="absolute inset-y-0 left-0 bg-accent"
        style={{ width: `${translatedWidth}%` }}
      />
      <div
        className="absolute inset-y-0 left-0 bg-primary"
        style={{ width: `${approvedWidth}%` }}
      />
    </div>
  );
}

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) {
    return 0;
  }

  return Math.max(0, Math.min(100, value));
}
