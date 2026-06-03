"use client";

import {
  AppWindow,
  Bookmark,
  ChevronRight,
  Clock3,
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
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger
} from "@/components/ui/collapsible";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from "@/components/ui/dropdown-menu";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  useSidebar
} from "@/components/ui/sidebar";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { useCurrentUser } from "@/hooks/use-current-user";
import { firstAccessiblePath, routeHref, routeIsActive, visibleMenuRoutes } from "@/lib/menu";
import type { RouteItem } from "@/types/auth";

type IconComponent = ComponentType<SVGProps<SVGSVGElement>>;

const ICONS: Record<string, IconComponent> = {
  apps: LayoutGrid,
  bookmark: Bookmark,
  clock: Clock3,
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
}

export function AppSidebar({ routes }: AppSidebarProps) {
  return (
    <Sidebar collapsible="icon" variant="inset">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" asChild>
              <Link href="/dashboard/workplace">
                <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                  <LayoutDashboard className="size-4" />
                </div>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate font-semibold">Avalon Admin</span>
                  <span className="truncate text-xs">Management Console</span>
                </div>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>导航</SidebarGroupLabel>
          <SidebarMenu>
            {visibleMenuRoutes(routes).map((route) => (
              <SidebarRoute key={route.id} route={route} />
            ))}
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <SidebarUserMenu />
      </SidebarFooter>
    </Sidebar>
  );
}

function SidebarRoute({ route }: { route: RouteItem }) {
  const pathname = usePathname();
  const { setOpenMobile } = useSidebar();
  const children = visibleMenuRoutes(route.children);
  const active = routeIsActive(route, pathname) || children.some((child) => routeIsActive(child, pathname));
  const href = route.redirect || (children.length > 0 ? firstAccessiblePath(children) : routeHref(route));
  const Icon = iconFor(route.icon);

  if (children.length === 0) {
    return (
      <SidebarMenuItem>
        <SidebarMenuButton asChild tooltip={route.title} isActive={active}>
          <Link href={href || "#"} onClick={() => setOpenMobile(false)}>
            <Icon className="app-icon" />
            <span>{route.title}</span>
          </Link>
        </SidebarMenuButton>
      </SidebarMenuItem>
    );
  }

  return (
    <Collapsible asChild defaultOpen={active}>
      <SidebarMenuItem>
        <SidebarMenuButton asChild tooltip={route.title} isActive={active}>
          <Link href={href || "#"} onClick={() => setOpenMobile(false)}>
            <Icon className="app-icon" />
            <span>{route.title}</span>
          </Link>
        </SidebarMenuButton>
        <CollapsibleTrigger asChild>
          <SidebarMenuAction className="data-[state=open]:rotate-90">
            <ChevronRight className="size-4" />
            <span className="sr-only">展开 {route.title}</span>
          </SidebarMenuAction>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <SidebarMenuSub>
            {children.map((child) => (
              <SidebarSubRoute key={child.id} route={child} pathname={pathname} />
            ))}
          </SidebarMenuSub>
        </CollapsibleContent>
      </SidebarMenuItem>
    </Collapsible>
  );
}

function SidebarSubRoute({ route, pathname }: { route: RouteItem; pathname: string }) {
  const { setOpenMobile } = useSidebar();
  const active = routeIsActive(route, pathname);
  const href = route.redirect || routeHref(route);

  return (
    <SidebarMenuSubItem>
      <SidebarMenuSubButton asChild isActive={active}>
        <Link href={href || "#"} onClick={() => setOpenMobile(false)}>
          <span>{route.title}</span>
        </Link>
      </SidebarMenuSubButton>
    </SidebarMenuSubItem>
  );
}

function SidebarUserMenu() {
  const { user, logout } = useCurrentUser();
  const { isMobile } = useSidebar();
  const displayName = user?.nickname || user?.username || "用户";
  const fallback = displayName.slice(0, 1).toUpperCase();

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <Avatar className="size-8 rounded-lg">
                {user?.avatar ? <AvatarImage src={user.avatar} alt={displayName} /> : null}
                <AvatarFallback className="rounded-lg text-xs">{fallback}</AvatarFallback>
              </Avatar>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-medium">{displayName}</span>
                <span className="truncate text-xs text-sidebar-foreground/70">{user?.deptName || "-"}</span>
              </div>
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            align="end"
            side={isMobile ? "bottom" : "right"}
            sideOffset={4}
            className="w-[--radix-dropdown-menu-trigger-width] min-w-56"
          >
            <DropdownMenuLabel>
              <div className="truncate">{displayName}</div>
              <div className="truncate text-xs font-normal text-muted-foreground">{user?.username || "-"}</div>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem asChild>
              <Link href="/user/profile">
                <User className="size-4" />
                个人中心
              </Link>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => void logout()}>
              <Lock className="size-4" />
              退出登录
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}

function iconFor(icon: string): IconComponent {
  return ICONS[icon] ?? AppWindow;
}
