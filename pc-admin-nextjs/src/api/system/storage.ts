import { api } from "@/lib/api";
import type { IdResponse, StorageCommand, StorageQuery, StorageResp } from "@/types/system";

const BASE_URL = "/system/storage";

export function listStorage(query: StorageQuery = {}) {
  return api.get<StorageResp[]>(`${BASE_URL}/list`, { ...query });
}

export function getStorage(id: number) {
  return api.get<StorageResp>(`${BASE_URL}/${id}`);
}

export function addStorage(data: StorageCommand) {
  return api.post<IdResponse>(BASE_URL, data);
}

export function updateStorage(id: number, data: StorageCommand) {
  return api.put<boolean>(`${BASE_URL}/${id}`, data);
}

export function deleteStorage(id: number) {
  return api.delete<boolean>(BASE_URL, { ids: [id] });
}

export function updateStorageStatus(id: number, status: number) {
  return api.put<boolean>(`${BASE_URL}/${id}/status`, { status });
}

export function setDefaultStorage(id: number) {
  return api.put<boolean>(`${BASE_URL}/${id}/default`);
}
