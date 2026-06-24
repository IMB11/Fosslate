import { ServerCog, User } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/router";
import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

const navItems = [
  { href: "/settings/profile", label: "Profile", icon: User },
  { href: "/settings/instance", label: "Instance", icon: ServerCog },
];

export function SettingsLayout({ children }: { children: ReactNode }) {
  const router = useRouter();

  return (
    <main className="min-h-[calc(100vh-5rem)] bg-background text-foreground">
      <div className="mx-auto grid w-full max-w-7xl gap-8 px-4 py-8 sm:px-6 lg:grid-cols-[240px_minmax(0,1fr)] lg:px-10">
        <aside className="border-b-2 border-border pb-4 lg:border-b-0 lg:border-r-2 lg:pb-0 lg:pr-5">
          <nav aria-label="Settings" className="flex gap-2 overflow-x-auto lg:block lg:space-y-2">
            {navItems.map((item) => {
              const Icon = item.icon;
              const active = router.pathname === item.href;

              return (
                <Link
                  aria-current={active ? "page" : undefined}
                  className={cn(
                    "flex min-h-11 shrink-0 items-center gap-3 rounded border-2 border-transparent px-3 py-2 font-head text-sm transition",
                    "hover:border-border hover:bg-accent",
                    active && "border-border bg-primary text-primary-foreground shadow-sm",
                  )}
                  href={item.href}
                  key={item.href}
                >
                  <Icon aria-hidden="true" className="size-4" />
                  {item.label}
                </Link>
              );
            })}
          </nav>
        </aside>

        <section className="min-w-0">{children}</section>
      </div>
    </main>
  );
}
