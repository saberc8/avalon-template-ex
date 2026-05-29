import type { ReactNode } from "react";
import { AdminShell } from "@/components/layout/admin-shell";
import { CurrentUserProvider } from "@/hooks/use-current-user";

export default function MainLayout({ children }: { children: ReactNode }) {
  return (
    <CurrentUserProvider>
      <AdminShell>{children}</AdminShell>
    </CurrentUserProvider>
  );
}
