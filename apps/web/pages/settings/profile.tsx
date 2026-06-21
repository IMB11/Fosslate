import { useQuery } from "@tanstack/react-query";

import { Badge } from "@/components/retroui/Badge";
import { Text } from "@/components/retroui/Text";
import { SettingsLayout } from "@/components/settings/SettingsLayout";
import {
  getOptionalAuthSession,
  type AuthUser,
} from "@/lib/auth-client";

type ProfileSettingsPageProps = {
  initialAuthUser?: AuthUser | null;
};

export default function ProfileSettingsPage({
  initialAuthUser,
}: ProfileSettingsPageProps) {
  const sessionQuery = useQuery({
    queryKey: ["authSession"],
    queryFn: getOptionalAuthSession,
    initialData: initialAuthUser,
  });
  const user = sessionQuery.data ?? null;

  return (
    <SettingsLayout>
      <div className="max-w-3xl space-y-6">
        <div className="space-y-2">
          <Text as="h1" className="text-3xl">
            Profile
          </Text>
          <p className="text-sm text-muted-foreground">Your account details.</p>
        </div>

        <div className="overflow-hidden border-2 border-border bg-card shadow-md">
          <dl className="divide-y-2 divide-border">
            <ProfileRow label="Username" value={user?.username ?? "Not signed in"} />
            <ProfileRow label="Email" value={user?.email ?? "Not signed in"} />
            <div className="grid gap-2 p-4 sm:grid-cols-[160px_minmax(0,1fr)]">
              <dt className="font-head text-sm font-bold">Admin</dt>
              <dd className="flex flex-wrap gap-2">
                {user?.is_admin ? (
                  <Badge size="sm" variant="surface">
                    Admin
                  </Badge>
                ) : (
                  <span className="text-sm text-muted-foreground">No</span>
                )}
              </dd>
            </div>
          </dl>
        </div>
      </div>
    </SettingsLayout>
  );
}

function ProfileRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid gap-2 p-4 sm:grid-cols-[160px_minmax(0,1fr)]">
      <dt className="font-head text-sm font-bold">{label}</dt>
      <dd className="min-w-0 break-words text-sm">{value}</dd>
    </div>
  );
}
