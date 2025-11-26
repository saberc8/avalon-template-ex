/**
 * 客户端列表/详情响应结构，对齐前端 ClientResp/ClientDetailResp，
 * 以及 Go 版 internal/interfaces/http/client_handler.go 中的 ClientResp。
 */
export interface ClientResp {
  id: number;
  clientId: string;
  clientType: string;
  authType: string;
  activeTimeout: string;
  timeout: string;
  status: string;
  createUser: string;
  createTime: string;
  updateUser: string;
  updateTime: string;
  createUserString: string;
  updateUserString: string;
}

export interface ClientDetailResp extends ClientResp {}

/**
 * 客户端查询参数，对齐前端 ClientQuery。
 */
export interface ClientQuery {
  clientType?: string;
  authType?: string[];
  status?: string;
  sort?: string[];
  page?: string;
  size?: string;
}

/**
 * 客户端创建/修改请求体，对齐 Go clientReq 与前端表单字段。
 */
export interface ClientReq {
  clientType: string;
  authType: string[];
  activeTimeout: number;
  timeout: number;
  status: number;
}

