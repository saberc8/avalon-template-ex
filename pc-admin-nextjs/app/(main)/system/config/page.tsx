"use client";

import { PermissionGate } from "@/components/permission/permission-gate";
import { ConfigClient } from "@/components/system/config-client";
import { ConfigLogin } from "@/components/system/config-login";
import { ConfigSecurity } from "@/components/system/config-security";
import { ConfigSite } from "@/components/system/config-site";
import { ConfigStorage } from "@/components/system/config-storage";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export default function ConfigPage() {
  return (
    <div className="mx-auto w-full max-w-7xl">
      <Tabs defaultValue="site" className="grid gap-4">
        <TabsList className="w-full justify-start overflow-x-auto">
          <PermissionGate permissions={["system:siteConfig:get"]}>
            <TabsTrigger value="site">网站</TabsTrigger>
          </PermissionGate>
          <PermissionGate permissions={["system:securityConfig:get"]}>
            <TabsTrigger value="security">安全</TabsTrigger>
          </PermissionGate>
          <PermissionGate permissions={["system:loginConfig:get"]}>
            <TabsTrigger value="login">登录</TabsTrigger>
          </PermissionGate>
          <PermissionGate permissions={["system:storage:list"]}>
            <TabsTrigger value="storage">存储</TabsTrigger>
          </PermissionGate>
          <PermissionGate permissions={["system:client:list"]}>
            <TabsTrigger value="client">客户端</TabsTrigger>
          </PermissionGate>
        </TabsList>
        <TabsContent value="site">
          <PermissionGate permissions={["system:siteConfig:get"]}>
            <ConfigSite />
          </PermissionGate>
        </TabsContent>
        <TabsContent value="security">
          <PermissionGate permissions={["system:securityConfig:get"]}>
            <ConfigSecurity />
          </PermissionGate>
        </TabsContent>
        <TabsContent value="login">
          <PermissionGate permissions={["system:loginConfig:get"]}>
            <ConfigLogin />
          </PermissionGate>
        </TabsContent>
        <TabsContent value="storage">
          <PermissionGate permissions={["system:storage:list"]}>
            <ConfigStorage />
          </PermissionGate>
        </TabsContent>
        <TabsContent value="client">
          <PermissionGate permissions={["system:client:list"]}>
            <ConfigClient />
          </PermissionGate>
        </TabsContent>
      </Tabs>
    </div>
  );
}
