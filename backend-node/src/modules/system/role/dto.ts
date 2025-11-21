export interface RoleResp {
  id: number;
  name: string;
  code: string;
  sort: number;
  description: string;
  dataScope: number;
  isSystem: boolean;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
  disabled: boolean;
}

export interface RoleDetailResp extends RoleResp {
  menuIds: number[];
  deptIds: number[];
  menuCheckStrictly: boolean;
  deptCheckStrictly: boolean;
}

export interface RoleUserResp {
  id: number;
  roleId: number;
  userId: number;
  username: string;
  nickname: string;
  gender: number;
  status: number;
  isSystem: boolean;
  description: string;
  deptId: number;
  deptName: string;
  roleIds: number[];
  roleNames: string[];
  disabled: boolean;
}

export interface RoleReq {
  name: string;
  code: string;
  sort: number;
  description: string;
  dataScope: number;
  deptIds: number[];
  deptCheckStrictly: boolean;
}

export interface RolePermissionReq {
  menuIds: number[];
  menuCheckStrictly: boolean;
}

export interface PageQuery {
  page?: string;
  size?: string;
  description?: string;
}

export interface PageResult<T> {
  list: T[];
  total: number;
}

