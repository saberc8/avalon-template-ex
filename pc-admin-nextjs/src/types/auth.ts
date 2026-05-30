export interface UserInfo {
  id: number;
  username: string;
  nickname: string;
  gender: 0 | 1 | 2;
  email: string;
  phone: string;
  avatar: string;
  description: string;
  pwdResetTime: string;
  pwdExpired: boolean;
  registrationDate: string;
  deptName: string;
  roles: string[];
  permissions: string[];
}

export interface RouteItem {
  id: number;
  title: string;
  parentId: number;
  type: 1 | 2 | 3;
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
  status: 0 | 1;
  children: RouteItem[];
  activeMenu: string;
  alwaysShow: boolean;
  breadcrumb: boolean;
  showInTabs: boolean;
  affix: boolean;
}

export interface AccountLoginRequest {
  username: string;
  password: string;
  captcha?: string;
  uuid?: string;
  clientId?: string;
  authType?: "ACCOUNT";
}

export interface LoginResponse {
  token: string;
  expire: string;
}

export interface ImageCaptchaResponse {
  uuid: string;
  img: string;
  expireTime: number;
  isEnabled: boolean;
}
