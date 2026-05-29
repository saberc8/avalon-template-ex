import { api } from "@/lib/api";
import type { AccountLoginRequest, LoginResponse, RouteItem, UserInfo } from "@/types/auth";

const DEFAULT_CLIENT_ID = "default";

export function accountLogin(request: AccountLoginRequest) {
  return api.post<LoginResponse>("/auth/login", {
    ...request,
    clientId: request.clientId || process.env.NEXT_PUBLIC_CLIENT_ID || DEFAULT_CLIENT_ID,
    authType: "ACCOUNT"
  });
}

export function logout() {
  return api.post<void>("/auth/logout");
}

export function getUserInfo() {
  return api.get<UserInfo>("/auth/user/info");
}

export function getUserRoutes() {
  return api.get<RouteItem[]>("/auth/user/route");
}
