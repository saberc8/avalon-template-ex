"use client";

import { Fragment } from "react";
import { RefreshCw } from "lucide-react";
import { usePathname } from "next/navigation";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { AppearanceSwitcher } from "@/components/theme/appearance-switcher";
import { useCurrentUser } from "@/hooks/use-current-user";
import { findRouteTrail } from "@/lib/menu";

export function HeaderBar() {
  const pathname = usePathname();
  const { routes, reload } = useCurrentUser();
  const trail = findRouteTrail(routes, pathname);
  const title = trail.at(-1)?.title || "工作台";

  return (
    <header className="sticky top-0 z-20 flex h-14 shrink-0 items-center gap-2 border-b bg-background/95 px-3 backdrop-blur lg:px-4">
      <SidebarTrigger className="-ml-1" />
      <Separator orientation="vertical" className="mx-1 h-4" />
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm font-semibold">{title}</div>
        <Breadcrumb className="hidden md:block">
          <BreadcrumbList>
            {(trail.length > 0 ? trail : [{ id: 0, title: "工作台" }]).map((route, index, items) => (
              <Fragment key={route.id}>
                <BreadcrumbItem>
                  <BreadcrumbPage className="text-xs">{route.title}</BreadcrumbPage>
                </BreadcrumbItem>
                {index < items.length - 1 ? <BreadcrumbSeparator /> : null}
              </Fragment>
            ))}
          </BreadcrumbList>
        </Breadcrumb>
      </div>
      <Button size="icon" variant="ghost" aria-label="刷新用户状态" onClick={() => void reload()}>
        <RefreshCw className="app-icon" />
      </Button>
      <AppearanceSwitcher />
    </header>
  );
}
