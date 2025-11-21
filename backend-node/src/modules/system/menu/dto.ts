export interface MenuResp {
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
  isCache: boolean;
  isHidden: boolean;
  permission: string;
  sort: number;
  status: number;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
  children: MenuResp[];
}

export interface MenuReq {
  type: number;
  icon: string;
  title: string;
  sort: number;
  permission: string;
  path: string;
  name: string;
  component: string;
  redirect: string;
  isExternal?: boolean;
  isCache?: boolean;
  isHidden?: boolean;
  parentId: number;
  status: number;
}

export interface IdsRequest {
  ids: number[];
}

