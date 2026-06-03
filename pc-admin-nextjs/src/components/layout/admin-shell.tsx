"use client";

import type { ReactNode } from "react";
import { Loader2 } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Button } from "@/components/ui/button";
import { AppSidebar } from "@/components/layout/app-sidebar";
import { HeaderBar } from "@/components/layout/header-bar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { useCurrentUser } from "@/hooks/use-current-user";
import { isPathAccessible } from "@/lib/menu";

export function AdminShell({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  const { routes, loading, error, landingPath, reload } = useCurrentUser();

  if (loading) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-muted/30">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 className="size-4 animate-spin" />
          加载中
        </div>
      </main>
    );
  }

  if (error) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-muted/30 p-4">
        <div className="w-full max-w-sm rounded-lg border bg-background p-5 text-center shadow-sm">
          <div className="font-medium">加载失败</div>
          <div className="mt-2 text-sm text-muted-foreground">{error}</div>
          <Button className="mt-4" variant="outline" onClick={() => void reload()}>
            重试
          </Button>
        </div>
      </main>
    );
  }

  if (!isPathAccessible(routes, pathname)) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-muted/30 p-4">
        <div className="w-full max-w-sm rounded-lg border bg-background p-5 text-center shadow-sm">
          <div className="font-medium">没有访问权限</div>
          <div className="mt-2 text-sm text-muted-foreground">当前账号无权访问该页面</div>
          <Button className="mt-4" asChild>
            <Link href={landingPath}>返回可访问页面</Link>
          </Button>
        </div>
      </main>
    );
  }

  return (
    <SidebarProvider>
      <AppSidebar routes={routes} />
      <SidebarInset>
        <HeaderBar />
        <main className="min-w-0 flex-1 bg-muted/30 p-[var(--content-padding)]">
          {children}
        </main>
      </SidebarInset>
    </SidebarProvider>
  );
}
