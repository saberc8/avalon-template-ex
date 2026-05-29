import { authHeader, clearToken } from "@/lib/auth";
import type { ApiResponse } from "@/types/api";

const DEFAULT_API_BASE_URL = "http://localhost:4398";

export interface RequestOptions extends Omit<RequestInit, "body"> {
  query?: Record<string, unknown>;
  body?: unknown;
}

export class ApiError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly response?: Response
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export async function apiRequest<T>(path: string, options: RequestOptions = {}) {
  const response = await fetch(apiUrl(path, options.query), buildRequestInit(options));
  const payload = (await response.json()) as ApiResponse<T>;

  if (!response.ok || payload.code !== "200") {
    if (payload.code === "401") {
      clearToken();
    }
    throw new ApiError(payload.code, payload.msg || "Request failed", response);
  }

  return payload.data;
}

export async function apiDownload(path: string, options: RequestOptions = {}) {
  const response = await fetch(apiUrl(path, options.query), buildRequestInit(options));
  if (!response.ok) {
    throw new ApiError(String(response.status), response.statusText, response);
  }
  return response.blob();
}

export const api = {
  get: <T>(path: string, query?: Record<string, unknown>) =>
    apiRequest<T>(path, { method: "GET", query }),
  post: <T>(path: string, body?: unknown, options?: RequestOptions) =>
    apiRequest<T>(path, { ...options, method: "POST", body }),
  put: <T>(path: string, body?: unknown, options?: RequestOptions) =>
    apiRequest<T>(path, { ...options, method: "PUT", body }),
  patch: <T>(path: string, body?: unknown, options?: RequestOptions) =>
    apiRequest<T>(path, { ...options, method: "PATCH", body }),
  delete: <T>(path: string, body?: unknown, options?: RequestOptions) =>
    apiRequest<T>(path, { ...options, method: "DELETE", body }),
  download: apiDownload
};

function buildRequestInit(options: RequestOptions): RequestInit {
  const headers = new Headers(options.headers);
  for (const [key, value] of Object.entries(authHeader())) {
    headers.set(key, value);
  }

  let body: BodyInit | undefined;
  if (options.body instanceof FormData) {
    body = options.body;
  } else if (options.body !== undefined) {
    headers.set("Content-Type", "application/json");
    body = JSON.stringify(options.body);
  }

  return {
    ...options,
    headers,
    body
  };
}

function apiUrl(path: string, query?: Record<string, unknown>) {
  const base = process.env.NEXT_PUBLIC_API_BASE_URL || DEFAULT_API_BASE_URL;
  const url = new URL(path, base);
  for (const [key, value] of Object.entries(query ?? {})) {
    appendQuery(url, key, value);
  }
  return url.toString();
}

function appendQuery(url: URL, key: string, value: unknown) {
  if (value === undefined || value === null || value === "") {
    return;
  }
  if (Array.isArray(value)) {
    for (const item of value) {
      appendQuery(url, key, item);
    }
    return;
  }
  url.searchParams.append(key, String(value));
}
