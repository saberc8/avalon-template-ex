/**
 * 系统日志列表/详情响应结构，对齐前端 LogResp/LogDetailResp，
 * 以及 Go 版 internal/interfaces/http/log_handler.go 中的 LogResp/LogDetailResp。
 */
export interface LogResp {
  id: number;
  description: string;
  module: string;
  timeTaken: number;
  ip: string;
  address: string;
  browser: string;
  os: string;
  status: number;
  errorMsg: string;
  createUserString: string;
  createTime: string;
}

export interface LogDetailResp extends LogResp {
  traceId: string;
  requestUrl: string;
  requestMethod: string;
  requestHeaders: string;
  requestBody: string;
  statusCode: number;
  responseHeaders: string;
  responseBody: string;
}

/**
 * 日志查询参数，对齐前端 LogQuery。
 */
export interface LogQuery {
  description?: string;
  module?: string;
  ip?: string;
  createUserString?: string;
  createTime?: string[] | string;
  status?: number;
  sort?: string[];
  page?: string;
  size?: string;
}

