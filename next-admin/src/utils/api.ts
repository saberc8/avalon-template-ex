export type ApiResponse<T> = {
  code: number;
  data: T;
  msg: string;
  success: boolean;
  timestamp: string;
};

const DEFAULT_API_BASE_URL = "http://localhost:4398";

export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? DEFAULT_API_BASE_URL;

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  const url = `${API_BASE_URL}${path}`;

  const baseHeaders = (options.headers ?? {}) as Record<string, string>;
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...baseHeaders,
  };

  const token = getToken();
  if (token && !headers.Authorization) {
    headers.Authorization = `Bearer ${token}`;
  }

  const res = await fetch(url, {
    ...options,
    headers,
  });

  if (!res.ok) {
    throw new Error(`网络错误：${res.status}`);
  }

  const json = (await res.json()) as ApiResponse<T>;
  if (!json.success) {
    throw new Error(json.msg || "服务器端错误");
  }

  return json;
}

export function get<T>(path: string): Promise<ApiResponse<T>> {
  return request<T>(path, { method: "GET" });
}

export function post<T, B = unknown>(
  path: string,
  body: B
): Promise<ApiResponse<T>> {
  return request<T>(path, {
    method: "POST",
    body: JSON.stringify(body),
  });
}
import { getToken } from "./auth";

