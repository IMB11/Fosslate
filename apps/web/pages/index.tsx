import type { GetServerSideProps, InferGetServerSidePropsType } from "next";

import { AuthButton } from "@/components/auth/AuthButton";
import { Card } from "@/components/retroui/Card";
import { Text } from "@/components/retroui/Text";
import { getBackendMeta, type BackendMetaResult } from "@/lib/api";

type HomeProps = {
  backend: BackendMetaResult;
};

export const getServerSideProps = (async () => {
  return {
    props: {
      backend: await getBackendMeta(),
    },
  };
}) satisfies GetServerSideProps<HomeProps>;

export default function Home({
  backend,
}: InferGetServerSidePropsType<typeof getServerSideProps>) {
  return (
    <main className="min-h-screen bg-background p-6 text-foreground">
      <Card>
        <Card.Content className="space-y-3">
          <Text as="h1">Hello world</Text>
          <Text>
            {backend.ok ? "Backend request succeeded" : backend.error}
          </Text>
          <AuthButton />
        </Card.Content>
      </Card>
    </main>
  );
}
