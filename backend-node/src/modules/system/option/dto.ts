/**
 * 系统配置 DTO，与 `/system/option` 接口返回/入参保持一致。
 */
export interface OptionResp {
  id: number;
  name: string;
  code: string;
  value: string;
  description: string;
}

export interface OptionUpdateItem {
  id: number;
  code: string;
  value: unknown;
}

export interface OptionResetReq {
  code?: string[];
  category?: string;
}
