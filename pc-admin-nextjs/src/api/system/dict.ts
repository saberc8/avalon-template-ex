import { api } from "@/lib/api";
import type { PageResult } from "@/types/api";
import type {
  DictCommand,
  DictItemCommand,
  DictItemQuery,
  DictItemResp,
  DictQuery,
  DictResp,
  IdResponse
} from "@/types/system";

const BASE_URL = "/system/dict";

export function listDict(query: DictQuery = {}) {
  return api.get<DictResp[]>(`${BASE_URL}/list`, { ...query });
}

export function getDict(id: number) {
  return api.get<DictResp>(`${BASE_URL}/${id}`);
}

export function addDict(data: DictCommand) {
  return api.post<IdResponse>(BASE_URL, data);
}

export function updateDict(id: number, data: DictCommand) {
  return api.put<boolean>(`${BASE_URL}/${id}`, data);
}

export function deleteDict(id: number) {
  return api.delete<boolean>(BASE_URL, { ids: [id] });
}

export function clearDictCache(code: string) {
  return api.delete<boolean>(`${BASE_URL}/cache/${code}`);
}

export function listDictItem(query: DictItemQuery) {
  return api.get<PageResult<DictItemResp>>(`${BASE_URL}/item`, { ...query });
}

export function getDictItem(id: number) {
  return api.get<DictItemResp>(`${BASE_URL}/item/${id}`);
}

export function addDictItem(data: DictItemCommand) {
  return api.post<IdResponse>(`${BASE_URL}/item`, data);
}

export function updateDictItem(id: number, data: DictItemCommand) {
  return api.put<boolean>(`${BASE_URL}/item/${id}`, data);
}

export function deleteDictItem(id: number) {
  return api.delete<boolean>(`${BASE_URL}/item`, { ids: [id] });
}
