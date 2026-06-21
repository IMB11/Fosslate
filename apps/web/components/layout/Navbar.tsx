import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronDown, LogOut, Settings } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/router";

import { Avatar } from "@/components/retroui/Avatar";
import { buttonVariants } from "@/components/retroui/Button";
import { Menu } from "@/components/retroui/Menu";
import { FosslateLogo } from "@/components/setup/FosslateLogo";
import {
  getOptionalAuthSession,
  logout,
  type AuthUser,
} from "@/lib/auth-client";
import { cn } from "@/lib/utils";

const authSessionQueryKey = ["authSession"];

type NavbarProps = {
  initialUser?: AuthUser | null;
};

export function Navbar({ initialUser }: NavbarProps) {
  const router = useRouter();
  const queryClient = useQueryClient();
  const sessionQuery = useQuery({
    queryKey: authSessionQueryKey,
    queryFn: getOptionalAuthSession,
    initialData: initialUser,
  });

  const logoutMutation = useMutation({
    mutationFn: logout,
    onSuccess: async () => {
      queryClient.setQueryData(authSessionQueryKey, null);
      await router.push("/login");
    },
  });

  const user = sessionQuery.data;
  const authResolved = user !== undefined || !sessionQuery.isPending;

  return (
    <header className="border-b-2 border-border bg-background text-foreground">
      <nav
        aria-label="Primary"
        className="mx-auto flex h-20 w-full max-w-7xl items-center gap-6 px-4 sm:px-6 lg:px-10"
      >
        <Link aria-label="Fosslate home" href="/">
          <FosslateLogo />
        </Link>

        <div className="flex-1" />

        {!authResolved ? (
          <div aria-hidden="true" className="h-11 w-28" />
        ) : user ? (
          <AccountMenu
            loggingOut={logoutMutation.isPending}
            onLogout={() => logoutMutation.mutate()}
            user={user}
          />
        ) : (
          <div className="flex items-center gap-3">
            <Link
              className={cn(buttonVariants({ variant: "outline" }), "bg-background")}
              href="/login"
            >
              Log in
            </Link>
            <Link className={buttonVariants()} href="/signup">
              Sign up
            </Link>
          </div>
        )}
      </nav>
    </header>
  );
}

function AccountMenu({
  loggingOut,
  onLogout,
  user,
}: {
  loggingOut: boolean;
  onLogout: () => void;
  user: AuthUser;
}) {
  const router = useRouter();
  const fallback = avatarFallback(user);

  return (
    <Menu>
      <Menu.Trigger
        aria-label="Open account menu"
        className="group flex items-center gap-2 border-2 border-border bg-background px-2 py-1 shadow-sm transition-all hover:translate-y-0.5 hover:shadow-xs focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
      >
        <Avatar className="size-10">
          {user.avatar_url ? <Avatar.Image alt="" src={user.avatar_url} /> : null}
          <Avatar.Fallback className="font-head text-base">
            {fallback}
          </Avatar.Fallback>
        </Avatar>
        <ChevronDown aria-hidden="true" className="size-4" />
      </Menu.Trigger>

      <Menu.Content className="min-w-44 border-border bg-background p-1 shadow-md">
        <Menu.Item
          className="gap-2 px-3 py-2 font-head text-sm"
          onClick={() => {
            void router.push("/settings");
          }}
        >
          <Settings aria-hidden="true" className="size-4" />
          Settings
        </Menu.Item>
        <Menu.Item
          className="gap-2 px-3 py-2 font-head text-sm"
          disabled={loggingOut}
          onClick={onLogout}
        >
          <LogOut aria-hidden="true" className="size-4" />
          {loggingOut ? "Logging out..." : "Log out"}
        </Menu.Item>
      </Menu.Content>
    </Menu>
  );
}

function avatarFallback(user: AuthUser): string {
  const source = user.username || user.email;
  const first = source.trim().charAt(0);

  return first ? first.toUpperCase() : "F";
}
