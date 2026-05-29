"use client";

import { Activity, FileText, ShieldCheck, UsersRound } from "lucide-react";
import Link from "next/link";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle
} from "@/components/ui/card";
import { PermissionGate } from "@/components/permission/permission-gate";
import { useCurrentUser } from "@/hooks/use-current-user";
import { flattenVisibleRoutes } from "@/lib/menu";

export default function WorkplacePage() {
  const { user, routes } = useCurrentUser();
  const routeCount = flattenVisibleRoutes(routes).length;

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-col gap-4">
      <section className="rounded-lg border bg-background p-5 shadow-sm">
        <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div>
            <div className="inline-flex items-center gap-2 rounded-full border bg-muted/45 px-3 py-1 text-xs text-muted-foreground">
              <Activity className="size-3.5 text-primary" />
              后台管理控制台
            </div>
            <h1 className="mt-3 text-xl font-semibold">工作台</h1>
            <p className="mt-1 text-sm text-muted-foreground">
              {user?.nickname || user?.username} · {user?.deptName || "未分配部门"}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            {(user?.roles ?? []).map((role) => (
              <Badge key={role} variant="outline">
                {role}
              </Badge>
            ))}
          </div>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-3">
        <Card className="shadow-sm">
          <CardHeader className="relative">
            <CardTitle className="text-sm font-medium text-muted-foreground">可访问菜单</CardTitle>
            <FileText className="app-icon absolute right-5 top-5 size-4" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-semibold tabular-nums">{routeCount}</div>
            <div className="mt-1 text-sm text-muted-foreground">来自后端路由树</div>
          </CardContent>
        </Card>
        <Card className="shadow-sm">
          <CardHeader className="relative">
            <CardTitle className="text-sm font-medium text-muted-foreground">权限标识</CardTitle>
            <ShieldCheck className="app-icon absolute right-5 top-5 size-4" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-semibold tabular-nums">{user?.permissions.length ?? 0}</div>
            <div className="mt-1 text-sm text-muted-foreground">前端按钮门控同步使用</div>
          </CardContent>
        </Card>
        <Card className="shadow-sm">
          <CardHeader className="relative">
            <CardTitle className="text-sm font-medium text-muted-foreground">账号状态</CardTitle>
            <UsersRound className="app-icon absolute right-5 top-5 size-4" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-semibold">启用</div>
            <div className="mt-1 text-sm text-muted-foreground">{user?.registrationDate || "-"}</div>
          </CardContent>
        </Card>
      </section>

      <section className="rounded-lg border bg-background p-5 shadow-sm">
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-base font-semibold">常用入口</h2>
        </div>
        <div className="flex flex-wrap gap-2">
          <PermissionGate permissions={["system:user:list"]}>
            <Button asChild variant="outline">
              <Link href="/system/user">用户管理</Link>
            </Button>
          </PermissionGate>
          <PermissionGate permissions={["system:role:list"]}>
            <Button asChild variant="outline">
              <Link href="/system/role">角色管理</Link>
            </Button>
          </PermissionGate>
          <PermissionGate permissions={["monitor:log:list"]}>
            <Button asChild variant="outline">
              <Link href="/monitor/log">系统日志</Link>
            </Button>
          </PermissionGate>
        </div>
      </section>
    </div>
  );
}
