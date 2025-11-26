/**
 * 存储配置响应结构，字段与前端 StorageResp 对齐，
 * 参考 backend-go/internal/interfaces/http/storage_handler.go 中的 StorageResp。
 */
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

/**
 * 存储配置查询参数，字段与前端 StorageQuery 对齐。
 */
export interface StorageQuery {
  description?: string;
  type?: number;
  sort?: string[];
}

/**
 * 存储状态更新请求体。
 */
export interface StorageStatusUpdateReq {
  status: number;
}

/**
 * 存储配置请求体，用于新增和修改。
 * 与 Java/Go storageReq 以及前端存储表单字段保持一致。
 */
export interface StorageReq {
  name: string;
  code: string;
  type: number;
  accessKey: string;
  secretKey?: string;
  endpoint: string;
  region: string;
  bucketName: string;
  domain: string;
  description: string;
  isDefault?: boolean;
  sort: number;
  status: number;
}
