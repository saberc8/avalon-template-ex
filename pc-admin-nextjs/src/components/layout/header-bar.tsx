"use client";

import { Menu, RefreshCw } from "lucide-react";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetTitle,
  SheetTrigger
} from "@/components/ui/sheet";
import { AppSidebar } from "@/components/layout/app-sidebar";
import { UserMenu } from "@/components/layout/user-menu";
import { useCurrentUser } from "@/hooks/use-current-user";
import { findRouteTrail } from "@/lib/menu";

export function HeaderBar() {
  const pathname = usePathname();
  const { routes, reload } = useCurrentUser();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const trail = findRouteTrail(routes, pathname);
  const title = trail.at(-1)?.title || "工作台";

  return (
    <header className="sticky top-0 z-20 flex h-14 shrink-0 items-center gap-3 border-b bg-background/95 px-3 backdrop-blur md:px-4">
      <Sheet open={mobileMenuOpen} onOpenChange={setMobileMenuOpen}>
        <SheetTrigger asChild>
          <Button className="md:hidden" size="icon" variant="ghost" aria-label="打开菜单">
            <Menu />
          </Button>
        </SheetTrigger>
        <SheetContent side="left" className="w-72 p-0">
          <SheetTitle className="sr-only">导航菜单</SheetTitle>
          <AppSidebar routes={routes} onNavigate={() => setMobileMenuOpen(false)} />
        </SheetContent>
      </Sheet>
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm font-semibold">{title}</div>
        <div className="hidden text-xs text-muted-foreground md:block">
          {trail.length > 0 ? trail.map((route) => route.title).join(" / ") : "Dashboard / Workplace"}
        </div>
      </div>
      <Button size="icon" variant="ghost" aria-label="刷新用户状态" onClick={() => void reload()}>
        <RefreshCw />
      </Button>
      <UserMenu />
    </header>
  );
}
