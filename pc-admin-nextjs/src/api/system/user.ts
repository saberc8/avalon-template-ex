import { api } from "@/lib/api";
import type { PageResult } from "@/types/api";
import type { IdResponse, UserCommand, UserDetailResp, UserQuery, UserResp } from "@/types/system";

const BASE_URL = "/system/user";

export function listUser(query: UserQuery = {}) {
  return api.get<PageResult<UserResp>>(BASE_URL, toQuery(query));
}

export function listAllUser(query: Partial<UserQuery> = {}) {
  return api.get<UserResp[]>(`${BASE_URL}/list`, toQuery(query));
}

export function getUser(id: number) {
  return api.get<UserDetailResp>(`${BASE_URL}/${id}`);
}

export function addUser(data: UserCommand) {
  return api.post<IdResponse>(BASE_URL, data);
}

export function updateUser(id: number, data: UserCommand) {
  return api.put<boolean>(`${BASE_URL}/${id}`, data);
}

export function deleteUser(id: number) {
  return api.delete<boolean>(BASE_URL, { ids: [id] });
}

export function exportUser(query: UserQuery = {}) {
  return api.download(`${BASE_URL}/export`, { query: toQuery(query) });
}

export function downloadUserImportTemplate() {
  return api.download(`${BASE_URL}/import/template`);
}

export function parseImportUser(data: FormData) {
  return api.post(`${BASE_URL}/import/parse`, data);
}

export function importUser(data: unknown) {
  return api.post(`${BASE_URL}/import`, data);
}

export function resetUserPwd(id: number, newPassword: string) {
  return api.patch<boolean>(`${BASE_URL}/${id}/password`, { newPassword });
}

export function updateUserRole(id: number, roleIds: number[]) {
  return api.patch<boolean>(`${BASE_URL}/${id}/role`, { roleIds });
}

function toQuery(query: Partial<UserQuery>): Record<string, unknown> {
  return { ...query };
}
