import Link from "next/link";

import { buttonVariants } from "@/components/retroui/Button";

export function AuthButton() {
  return (
    <Link className={buttonVariants()} href="/login">
      Auth
    </Link>
  );
}
