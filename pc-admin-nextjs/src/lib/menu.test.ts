import { describe, expect, it } from "vitest";
import { findRouteTrail, firstAccessiblePath, flattenVisibleRoutes, isPathAccessible } from "@/lib/menu";
import type { RouteItem } from "@/types/auth";

const routes: RouteItem[] = [
  {
    id: 1000,
    title: "系统管理",
    parentId: 0,
    type: 1,
    path: "/system",
    name: "System",
    component: "Layout",
    redirect: "/system/user",
    icon: "settings",
    isExternal: false,
    isHidden: false,
    isCache: false,
    permission: "",
    roles: ["admin"],
    sort: 2,
    status: 1,
    activeMenu: "",
    alwaysShow: false,
    breadcrumb: true,
    showInTabs: true,
    affix: false,
    children: [
      {
        id: 1010,
        title: "用户管理",
        parentId: 1000,
        type: 2,
        path: "/system/user",
        name: "SystemUser",
        component: "system/user/index",
        redirect: "",
        icon: "user",
        isExternal: false,
        isHidden: false,
        isCache: false,
        permission: "",
        roles: ["admin"],
        sort: 2,
        status: 1,
        activeMenu: "",
        alwaysShow: false,
        breadcrumb: true,
        showInTabs: true,
        affix: false,
        children: []
      },
      {
        id: 1011,
        title: "新增",
        parentId: 1010,
        type: 3,
        path: "",
        name: "",
        component: "",
        redirect: "",
        icon: "",
        isExternal: false,
        isHidden: false,
        isCache: false,
        permission: "system:user:create",
        roles: ["admin"],
        sort: 1,
        status: 1,
        activeMenu: "",
        alwaysShow: false,
        breadcrumb: true,
        showInTabs: true,
        affix: false,
        children: []
      },
      {
        id: 1150,
        title: "系统配置",
        parentId: 1000,
        type: 2,
        path: "/system/config?tab=site",
        name: "SystemConfig",
        component: "system/config/index",
        redirect: "",
        icon: "config",
        isExternal: false,
        isHidden: true,
        isCache: false,
        permission: "",
        roles: ["admin"],
        sort: 1,
        status: 1,
        activeMenu: "",
        alwaysShow: false,
        breadcrumb: true,
        showInTabs: true,
        affix: false,
        children: []
      }
    ]
  },
  {
    id: 2000,
    title: "系统监控",
    parentId: 0,
    type: 1,
    path: "/monitor",
    name: "Monitor",
    component: "Layout",
    redirect: "/monitor/online",
    icon: "computer",
    isExternal: false,
    isHidden: false,
    isCache: false,
    permission: "",
    roles: ["admin"],
    sort: 1,
    status: 1,
    activeMenu: "",
    alwaysShow: false,
    breadcrumb: true,
    showInTabs: true,
    affix: false,
    children: [
      {
        id: 2010,
        title: "在线用户",
        parentId: 2000,
        type: 2,
        path: "/monitor/online",
        name: "MonitorOnline",
        component: "monitor/online/index",
        redirect: "",
        icon: "user",
        isExternal: false,
        isHidden: false,
        isCache: false,
        permission: "",
        roles: ["admin"],
        sort: 1,
        status: 1,
        activeMenu: "",
        alwaysShow: false,
        breadcrumb: true,
        showInTabs: true,
        affix: false,
        children: []
      }
    ]
  }
];

describe("menu helpers", () => {
  it("flattens visible routes in backend sort order and excludes buttons", () => {
    expect(flattenVisibleRoutes(routes).map((route) => route.path)).toEqual([
      "/monitor",
      "/monitor/online",
      "/system",
      "/system/user"
    ]);
  });

  it("returns the first leaf route as the post-login landing path", () => {
    expect(firstAccessiblePath(routes)).toBe("/monitor/online");
  });

  it("finds the current route trail by pathname", () => {
    expect(findRouteTrail(routes, "/system/user").map((route) => route.title)).toEqual(["系统管理", "用户管理"]);
  });

  it("checks whether a direct pathname is accessible from dynamic routes", () => {
    expect(isPathAccessible(routes, "/system/user")).toBe(true);
    expect(isPathAccessible(routes, "/system/user/detail")).toBe(true);
    expect(isPathAccessible(routes, "/dashboard/workplace")).toBe(true);
    expect(isPathAccessible(routes, "/system/role")).toBe(false);
  });
});
