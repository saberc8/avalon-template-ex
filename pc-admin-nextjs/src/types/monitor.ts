import type { PageQuery } from "@/types/api";

export interface OnlineUserResp {
  id: number;
  token: string;
  username: string;
  nickname: string;
  clientType: string;
  clientId: string;
  ip: string;
  address: string;
  browser: string;
  os: string;
  loginTime: string;
  lastActiveTime: string;
}

export interface OnlineUserQuery extends PageQuery {
  nickname?: string;
  loginTime?: string[];
  sort?: string[];
}

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

export interface LogQuery extends PageQuery {
  description?: string;
  module?: string;
  ip?: string;
  createUserString?: string;
  createTime?: string[];
  status?: number;
  sort?: string[];
}
