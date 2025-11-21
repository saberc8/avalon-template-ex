/**
 * 用户列表/详情响应结构，对齐前端 admin/src/apis/system/type.ts 中的 UserResp/UserDetailResp，
 * 以及 Go 版 internal/interfaces/http/system_user_handler.go 定义。
 */
export interface SystemUserResp {
  id: number;
  username: string;
  nickname: string;
  avatar: string;
  gender: number;
  email: string;
  phone: string;
  description: string;
  status: number;
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

export interface SystemUserDetailResp extends SystemUserResp {
  pwdResetTime?: string;
}

export interface SystemUserListQuery {
  page?: string;
  size?: string;
  description?: string;
  status?: string;
  deptId?: string;
}

export interface SystemUserReq {
  username: string;
  nickname: string;
  password?: string;
  gender: number;
  email: string;
  phone: string;
  avatar: string;
  description: string;
  status: number;
  deptId: number;
  roleIds: number[];
}

export interface SystemUserPasswordResetReq {
  newPassword: string;
}

export interface SystemUserRoleUpdateReq {
  roleIds: number[];
}

/**
 * 通用分页结果封装，与 Go 版 PageResult[T] 一致。
 */
export interface PageResult<T> {
  list: T[];
  total: number;
}

/**
 * 批量 ID 请求体。
 */
export interface IdsRequest {
  ids: number[];
}

/**
 * 导入相关响应，保持字段名与 Go/Java 一致，便于前端复用。
 */
export interface UserImportParseResp {
  importKey: string;
  totalRows: number;
  validRows: number;
  duplicateUserRows: number;
  duplicateEmailRows: number;
  duplicatePhoneRows: number;
}

export interface UserImportResultResp {
  totalRows: number;
  insertRows: number;
  updateRows: number;
}

