import { api } from "@/lib/api";
import type { DeptCommand, DeptQuery, DeptResp } from "@/types/system";

const BASE_URL = "/system/dept";

export function listDept(query: DeptQuery = {}) {
  return api.get<DeptResp[]>(`${BASE_URL}/tree`, { ...query });
}

export function getDept(id: number) {
  return api.get<DeptResp>(`${BASE_URL}/${id}`);
}

export function addDept(data: DeptCommand) {
  return api.post<boolean>(BASE_URL, data);
}

export function updateDept(id: number, data: DeptCommand) {
  return api.put<boolean>(`${BASE_URL}/${id}`, data);
}

export function deleteDept(id: number) {
  return api.delete<boolean>(BASE_URL, { ids: [id] });
}

export function exportDept(query: DeptQuery = {}) {
  return api.download(`${BASE_URL}/export`, { query: { ...query } });
}
