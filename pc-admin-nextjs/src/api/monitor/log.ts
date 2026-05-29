import { api } from "@/lib/api";
import type { PageResult } from "@/types/api";
import type { LogDetailResp, LogQuery, LogResp } from "@/types/monitor";

const BASE_URL = "/system/log";

export function listLog(query: LogQuery) {
  return api.get<PageResult<LogResp>>(BASE_URL, { ...query });
}

export function getLog(id: number) {
  return api.get<LogDetailResp>(`${BASE_URL}/${id}`);
}

export function exportLoginLog(query: LogQuery = {}) {
  return api.download(`${BASE_URL}/export/login`, { query: { ...query } });
}

export function exportOperationLog(query: LogQuery = {}) {
  return api.download(`${BASE_URL}/export/operation`, { query: { ...query } });
}
