import type { RouteItem } from "@/types/route";

export type AppMenuItem = {
  label: string;
  path?: string;
  children?: AppMenuItem[];
};

/**
 * 将后端返回的路由树转成侧边菜单数据
 * 参考 Vue3 版本的动态路由菜单渲染，只保留菜单需要的字段。
 */
export function buildMenuItemsFromRoutes(routes: RouteItem[]): AppMenuItem[] {
  const walk = (nodes: RouteItem[]): AppMenuItem[] => {
    return nodes
      .filter((node) => !node.isHidden)
      .map<AppMenuItem>((node) => {
        const children = node.children ? walk(node.children) : [];

        if (children.length > 0) {
          // 有子节点时作为分组展示，不直接可点击
          return {
            label: node.title,
            children,
          };
        }

        // 叶子节点直接使用自身路径
        return {
          label: node.title,
          path: node.path || undefined,
        };
      });
  };

  return walk(routes);
}

/**
 * 静态菜单兜底配置（在路由接口异常时使用）
 */
export const fallbackMenuItems: AppMenuItem[] = [
  {
    label: "仪表盘",
    children: [
      {
        label: "工作台",
        path: "/dashboard/workplace",
      },
    ],
  },
  {
    label: "系统管理",
    children: [
      { label: "用户管理", path: "/system/user" },
      { label: "角色管理", path: "/system/role" },
      { label: "菜单管理", path: "/system/menu" },
      { label: "文件管理", path: "/system/file" },
    ],
  },
  {
    label: "系统监控",
    children: [{ label: "系统日志", path: "/monitor/log" }],
  },
  {
    label: "个人中心",
    path: "/user/profile",
  },
];
