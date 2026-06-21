import { Text } from "@/components/retroui/Text";

export function FosslateLogo() {
  return (
    <div className="flex items-center gap-2">
      <div className="flex size-7 items-center justify-center rounded border-2 border-border bg-primary shadow-sm">
        <span className="font-head text-sm leading-none">F</span>
      </div>
      <Text as="h2" className="text-3xl leading-9">
        Fosslate
      </Text>
    </div>
  );
}

