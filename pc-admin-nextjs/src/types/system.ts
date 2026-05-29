import type { PageQuery } from "@/types/api";

export interface IdResponse {
  id: number;
}

export interface UserResp {
  id: number;
  username: string;
  nickname: string;
  avatar: string;
  gender: number;
  email: string;
  phone: string;
  description: string;
  status: 1 | 2;
  isSystem: boolean;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
  deptId: number;
  deptName: string;
  roleIds: number[];
  roleNames: string[];
  disabled: boolean;
}

export type UserDetailResp = UserResp & {
  pwdResetTime?: string;
};

export interface UserQuery extends PageQuery {
  description?: string;
  status?: number;
  deptId?: number | string;
  sort?: string[];
  userIds?: number[];
  roleId?: number | string;
}

export interface UserCommand {
  username: string;
  nickname: string;
  password?: string;
  gender: number;
  email: string;
  phone: string;
  avatar?: string;
  description: string;
  status: number;
  deptId: number;
  roleIds: number[];
}

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

export type RoleDetailResp = RoleResp & {
  menuIds: number[];
  deptIds: number[];
  menuCheckStrictly: boolean;
  deptCheckStrictly: boolean;
};

export interface RoleCommand {
  name: string;
  code: string;
  sort: number;
  description: string;
  dataScope: number;
  deptIds: number[];
  deptCheckStrictly: boolean;
}

export interface RolePermissionCommand {
  menuIds: number[];
  menuCheckStrictly: boolean;
}

export interface RoleQuery {
  description?: string;
  sort?: string[];
}

export interface RoleUserResp {
  id: number;
  roleId: number;
  userId: number;
  username: string;
  nickname: string;
  gender: number;
  description: string;
  status: 1 | 2;
  isSystem: boolean;
  deptId: number;
  deptName: string;
  roleIds: number[];
  roleNames: string[];
  disabled: boolean;
}

export interface RoleUserPageQuery extends PageQuery {
  description?: string;
  sort?: string[];
}

export interface MenuResp {
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
  isCache: boolean;
  isHidden: boolean;
  permission: string;
  sort: number;
  status: 1 | 2;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
  children: MenuResp[];
}

export interface MenuQuery {
  title?: string;
  status?: number;
}

export interface MenuCommand {
  type: 1 | 2 | 3;
  icon: string;
  title: string;
  sort: number;
  permission: string;
  path: string;
  name: string;
  component: string;
  redirect: string;
  isExternal: boolean;
  isCache: boolean;
  isHidden: boolean;
  parentId: number;
  status: number;
}

export interface DeptResp {
  id: number;
  name: string;
  sort: number;
  status: 1 | 2;
  isSystem: boolean;
  description: string;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
  parentId: number;
  children: DeptResp[];
}

export interface DeptQuery {
  description?: string;
  status?: number;
}

export interface DeptCommand {
  name: string;
  parentId: number;
  sort: number;
  status: number;
  description: string;
}

export interface DictResp {
  id: number;
  name: string;
  code: string;
  isSystem: boolean;
  description: string;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
}

export interface DictQuery {
  description?: string;
  sort?: string[];
}

export interface DictCommand {
  name: string;
  code: string;
  description: string;
}

export interface DictItemResp {
  id: number;
  label: string;
  value: string;
  color: string;
  sort: number;
  description: string;
  status: 1 | 2;
  dictId: number;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
}

export interface DictItemQuery extends PageQuery {
  description?: string;
  status?: number;
  sort?: string[];
  dictId?: number | string;
}

export interface DictItemCommand {
  dictId: number;
  label: string;
  value: string;
  color: string;
  sort: number;
  description: string;
  status: number;
}

export interface FileItem {
  id: number;
  name: string;
  originalName: string;
  size: number;
  url: string;
  parentPath: string;
  path: string;
  sha256: string;
  contentType: string;
  metadata: string;
  thumbnailSize: number;
  thumbnailName: string;
  thumbnailMetadata: string;
  thumbnailUrl: string;
  extension: string;
  type: number;
  storageId: number;
  storageName: string;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
}

export interface FileStatisticsResp {
  type: string;
  size: number;
  number: number;
  unit: string;
  data: FileStatisticsResp[];
}

export interface FileDirCalcSizeResp {
  size: number;
}

export interface FileQuery extends PageQuery {
  originalName?: string;
  type?: string;
  parentPath?: string;
  sort?: string[];
}

export interface OptionResp {
  id: number;
  name: string;
  code: string;
  value: string;
  description: string;
}

export interface OptionQuery {
  code?: string[];
  category?: string;
}

export interface OptionUpdateItem {
  id: number;
  code: string;
  value: unknown;
}

export interface StorageResp {
  id: number;
  name: string;
  code: string;
  type: number;
  accessKey: string;
  secretKey: string;
  endpoint: string;
  region: string;
  bucketName: string;
  domain: string;
  description: string;
  isDefault: boolean;
  sort: number;
  status: number;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
}

export interface StorageQuery {
  description?: string;
  type?: number;
  sort?: string[];
}

export interface StorageCommand {
  name: string;
  code: string;
  type: number;
  accessKey: string;
  secretKey: string;
  endpoint: string;
  region: string;
  bucketName: string;
  domain: string;
  description: string;
  isDefault: boolean;
  sort: number;
  status: number;
}

export interface ClientResp {
  id: number;
  clientId: string;
  clientType: string;
  authType: string[];
  activeTimeout: number;
  timeout: number;
  status: number;
  createUser: number;
  createTime: string;
  updateUser: number;
  updateTime: string;
  createUserString: string;
  updateUserString: string;
}

export interface ClientQuery extends PageQuery {
  clientType?: string;
  authType?: string[];
  status?: number;
  sort?: string[];
}

export interface ClientCommand {
  clientType: string;
  authType: string[];
  activeTimeout: number;
  timeout: number;
  status: number;
}
