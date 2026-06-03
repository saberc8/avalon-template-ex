"use client";

import { ChevronRight } from "lucide-react";
import type { ComponentType } from "react";
import { PermissionGate } from "@/components/permission/permission-gate";
import { ConfigClient } from "@/components/system/config-client";
import { ConfigLogin } from "@/components/system/config-login";
import { ConfigSecurity } from "@/components/system/config-security";
import { ConfigSite } from "@/components/system/config-site";
import { ConfigStorage } from "@/components/system/config-storage";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

interface ConfigSection {
  value: string;
  title: string;
  description: string;
  permissions: string[];
  Component: ComponentType;
}

const configSections: ConfigSection[] = [
  {
    value: "site",
    title: "网站配置",
    description: "站点名称、备案信息和前台展示参数",
    permissions: ["system:siteConfig:get"],
    Component: ConfigSite
  },
  {
    value: "security",
    title: "安全配置",
    description: "密码复杂度、登录限制和账号安全策略",
    permissions: ["system:securityConfig:get"],
    Component: ConfigSecurity
  },
  {
    value: "login",
    title: "登录配置",
    description: "登录页面、认证方式和会话参数",
    permissions: ["system:loginConfig:get"],
    Component: ConfigLogin
  },
  {
    value: "storage",
    title: "存储配置",
    description: "上传存储端、默认存储和文件访问参数",
    permissions: ["system:storage:list"],
    Component: ConfigStorage
  },
  {
    value: "client",
    title: "客户端配置",
    description: "不同终端的认证方式和会话有效期",
    permissions: ["system:client:list"],
    Component: ConfigClient
  }
];

export default function ConfigPage() {
  return (
    <div className="mx-auto w-full max-w-7xl">
      <Tabs
        defaultValue="site"
        className="grid w-full items-start gap-4 lg:grid-cols-[340px_1fr]"
        data-testid="config-layout"
      >
        <section className="self-start rounded-lg border bg-background p-4" data-testid="config-list-panel">
          <div className="mb-3">
            <h2 className="text-base font-semibold">系统配置</h2>
            <p className="text-xs text-muted-foreground">{configSections.length} 个配置分组</p>
          </div>
          <TabsList className="grid h-auto w-full items-stretch gap-2 bg-transparent p-0 text-foreground">
            {configSections.map((section) => (
              <PermissionGate key={section.value} permissions={section.permissions}>
                <TabsTrigger
                  value={section.value}
                  className="group relative h-auto w-full justify-start rounded-md border bg-background p-3 pr-16 text-left shadow-none hover:bg-muted/35 data-[state=active]:border-primary data-[state=active]:bg-primary/5 data-[state=active]:shadow-none"
                  data-testid={`config-card-${section.value}`}
                >
                  <span className="grid min-w-0 gap-1">
                    <span className="truncate font-medium">{section.title}</span>
                    <span className="line-clamp-2 whitespace-normal text-xs font-normal text-muted-foreground">
                      {section.description}
                    </span>
                  </span>
                  <span
                    aria-hidden="true"
                    className="pointer-events-none absolute right-2 top-2 inline-flex size-8 items-center justify-center rounded-md bg-background/85 opacity-0 shadow-sm transition-opacity group-focus-visible:opacity-100 group-hover:opacity-100"
                    data-testid={`config-card-action-${section.value}`}
                  >
                    <ChevronRight className="size-4" />
                  </span>
                </TabsTrigger>
              </PermissionGate>
            ))}
          </TabsList>
        </section>
        <section className="grid self-start content-start gap-4" data-testid="config-content-panel">
          {configSections.map(({ Component, ...section }) => (
            <TabsContent key={section.value} value={section.value} className="mt-0">
              <PermissionGate permissions={section.permissions}>
                <Component />
              </PermissionGate>
            </TabsContent>
          ))}
        </section>
      </Tabs>
    </div>
  );
}
