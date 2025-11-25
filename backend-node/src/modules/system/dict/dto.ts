/**
 * 字典管理相关 DTO，字段与前端 `UserDictResp`/`DictItemResp` 等类型保持一致。
 */
export interface DictResp {
  id: number;
  name: string;
  code: string;
  description: string;
  isSystem: boolean;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
}

export interface DictReq {
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
  status: number;
  dictId: number;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
}

export interface DictItemReq {
  dictId: number;
  label: string;
  value: string;
  color: string;
  sort: number;
  description: string;
  status: number;
}

export interface DictItemListQuery {
  dictId?: string;
  page?: string;
  size?: string;
  description?: string;
  status?: string;
}

export interface IdsRequest {
  ids: number[];
}

export interface PageResult<T> {
  list: T[];
  total: number;
}
