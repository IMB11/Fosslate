import type { ReactNode } from "react";

import { Alert } from "@/components/retroui/Alert";
import { Card } from "@/components/retroui/Card";
import { Text } from "@/components/retroui/Text";

export function SetupCard({
  title,
  description,
  error,
  footer,
  children,
}: {
  title: string;
  description: string;
  error?: string | null;
  footer?: ReactNode;
  children: ReactNode;
}) {
  return (
    <Card className="w-full bg-card hover:shadow-md">
      <Card.Content className="space-y-6 p-6">
        <div className="space-y-2">
          <h1 className="font-head text-base font-bold leading-5 text-foreground">
            {title}
          </h1>
          <Text className="text-sm leading-5 text-foreground">{description}</Text>
        </div>

        {error ? (
          <Alert status="error">
            <Alert.Description className="text-red-900">{error}</Alert.Description>
          </Alert>
        ) : null}

        {children}
        {footer}
      </Card.Content>
    </Card>
  );
}
