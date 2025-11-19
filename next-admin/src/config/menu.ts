export type AppMenuItem = {
  label: string;
  path?: string;
  children?: AppMenuItem[];
};

/**
 * 菜单配置（参考 Vue3 版本的 systemRoutes 与常用模块）
 */
export const menuItems: AppMenuItem[] = [
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
    children: [
      { label: "系统日志", path: "/monitor/log" },
    ],
  },
  {
    label: "个人中心",
    path: "/user/profile",
  },
];
