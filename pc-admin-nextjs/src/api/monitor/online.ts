import { api } from "@/lib/api";
import type { PageResult } from "@/types/api";
import type { OnlineUserQuery, OnlineUserResp } from "@/types/monitor";

const BASE_URL = "/monitor/online";

export function listOnlineUser(query: OnlineUserQuery) {
  return api.get<PageResult<OnlineUserResp>>(BASE_URL, { ...query });
}

export function kickout(token: string) {
  return api.delete<boolean>(`${BASE_URL}/${token}`);
}
