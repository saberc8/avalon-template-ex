/**
 * /auth/user/route 返回的单个路由节点结构。
 */
export interface RouteItem {
  id: number;
  title: string;
  parentId: number;
  type: number;
  path: string;
  name: string;
  component: string;
  redirect: string;
  icon: string;
  isExternal: boolean;
  isHidden: boolean;
  isCache: boolean;
  permission: string;
  roles: string[];
  sort: number;
  status: number;
  children: RouteItem[];
  activeMenu: string;
  alwaysShow: boolean;
  breadcrumb: boolean;
  showInTabs: boolean;
  affix: boolean;
}

