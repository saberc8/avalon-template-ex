"use client";

import {
  AppWindow,
  Bookmark,
  Circle,
  File,
  HardDrive,
  History,
  LayoutDashboard,
  LayoutGrid,
  Lock,
  Menu,
  Monitor,
  Network,
  Settings,
  ShieldCheck,
  SlidersHorizontal,
  Smartphone,
  User,
  Users
} from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import type { ComponentType, SVGProps } from "react";
import { cn } from "@/lib/utils";
import { firstAccessiblePath, routeHref, routeIsActive, visibleMenuRoutes } from "@/lib/menu";
import type { RouteItem } from "@/types/auth";

type IconComponent = ComponentType<SVGProps<SVGSVGElement>>;

const ICONS: Record<string, IconComponent> = {
  apps: LayoutGrid,
  bookmark: Bookmark,
  computer: Monitor,
  config: SlidersHorizontal,
  file: File,
  history: History,
  "mind-mapping": Network,
  menu: Menu,
  mobile: Smartphone,
  safe: ShieldCheck,
  settings: Settings,
  storage: HardDrive,
  lock: Lock,
  user: User,
  "user-group": Users
};

interface AppSidebarProps {
  routes: RouteItem[];
  className?: string;
  onNavigate?: () => void;
}

export function AppSidebar({ routes, className, onNavigate }: AppSidebarProps) {
  const pathname = usePathname();

  return (
    <aside className={cn("flex h-full min-h-0 flex-col border-r bg-background", className)}>
      <Link
        className="flex h-14 shrink-0 items-center gap-2 border-b px-4 font-semibold"
        href="/dashboard/workplace"
        onClick={onNavigate}
      >
        <span className="flex size-8 items-center justify-center rounded-md bg-primary text-primary-foreground">
          <LayoutDashboard className="size-4" />
        </span>
        <span>Avalon Admin</span>
      </Link>
      <nav className="min-h-0 flex-1 overflow-y-auto px-3 py-3">
        <SidebarTree routes={visibleMenuRoutes(routes)} pathname={pathname} onNavigate={onNavigate} />
      </nav>
    </aside>
  );
}

function SidebarTree({
  routes,
  pathname,
  onNavigate,
  depth = 0
}: {
  routes: RouteItem[];
  pathname: string;
  onNavigate?: () => void;
  depth?: number;
}) {
  return (
    <div className={cn("space-y-1", depth > 0 && "mt-1")}>
      {routes.map((route) => (
        <SidebarItem
          key={route.id}
          route={route}
          pathname={pathname}
          depth={depth}
          onNavigate={onNavigate}
        />
      ))}
    </div>
  );
}

function SidebarItem({
  route,
  pathname,
  depth,
  onNavigate
}: {
  route: RouteItem;
  pathname: string;
  depth: number;
  onNavigate?: () => void;
}) {
  const children = visibleMenuRoutes(route.children);
  const href = route.redirect || (children.length > 0 ? firstAccessiblePath(children) : routeHref(route));
  const active = routeIsActive(route, pathname) || children.some((child) => routeIsActive(child, pathname));
  const Icon = iconFor(route.icon);
  const indent = depth > 0 ? "pl-8" : "";

  return (
    <div>
      {href ? (
        <Link
          className={cn(
            "flex h-9 items-center gap-2 rounded-md px-3 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
            indent,
            active && "bg-primary/10 font-medium text-primary"
          )}
          href={href}
          onClick={onNavigate}
        >
          <Icon className="size-4 shrink-0" />
          <span className="truncate">{route.title}</span>
        </Link>
      ) : (
        <div
          className={cn(
            "flex h-9 items-center gap-2 rounded-md px-3 text-sm font-medium text-muted-foreground",
            indent
          )}
        >
          <Icon className="size-4 shrink-0" />
          <span className="truncate">{route.title}</span>
        </div>
      )}
      {children.length > 0 ? (
        <div className="ml-2 border-l pl-2">
          <SidebarTree routes={children} pathname={pathname} depth={depth + 1} onNavigate={onNavigate} />
        </div>
      ) : null}
    </div>
  );
}

function iconFor(icon: string): IconComponent {
  return ICONS[icon] ?? AppWindow ?? Circle;
}
