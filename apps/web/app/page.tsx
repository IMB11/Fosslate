import { Card } from "@/components/retroui/Card";
import { Text } from "@/components/retroui/Text";
import { getBackendMeta } from "@/lib/api";

export const dynamic = "force-dynamic";

export default async function Home() {
  const backend = await getBackendMeta();

  return (
    <main className="min-h-screen bg-background p-6 text-foreground">
      <Card>
        <Card.Content className="space-y-3">
          <Text as="h1">Hello world</Text>
          <Text>
            {backend.ok ? "Backend request succeeded" : backend.error}
          </Text>
        </Card.Content>
      </Card>
    </main>
  );
}
