import { api } from "@/lib/api";
import type { IdResponse, MenuCommand, MenuQuery, MenuResp } from "@/types/system";

const BASE_URL = "/system/menu";

export function listMenu(query: MenuQuery = {}) {
  return api.get<MenuResp[]>(`${BASE_URL}/tree`, { ...query });
}

export function getMenu(id: number) {
  return api.get<MenuResp>(`${BASE_URL}/${id}`);
}

export function addMenu(data: MenuCommand) {
  return api.post<IdResponse>(BASE_URL, data);
}

export function updateMenu(id: number, data: MenuCommand) {
  return api.put<boolean>(`${BASE_URL}/${id}`, data);
}

export function deleteMenu(id: number) {
  return api.delete<boolean>(BASE_URL, { ids: [id] });
}

export function clearMenuCache() {
  return api.delete<boolean>(`${BASE_URL}/cache`);
}
