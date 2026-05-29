import { api } from "@/lib/api";
import type { PageResult } from "@/types/api";
import type {
  IdResponse,
  RoleCommand,
  RoleDetailResp,
  RolePermissionCommand,
  RoleQuery,
  RoleResp,
  RoleUserPageQuery,
  RoleUserResp
} from "@/types/system";

const BASE_URL = "/system/role";

export function listRole(query: RoleQuery = {}) {
  return api.get<RoleResp[]>(`${BASE_URL}/list`, toQuery(query));
}

export function getRole(id: number) {
  return api.get<RoleDetailResp>(`${BASE_URL}/${id}`);
}

export function addRole(data: RoleCommand) {
  return api.post<IdResponse>(BASE_URL, data);
}

export function updateRole(id: number, data: RoleCommand) {
  return api.put<boolean>(`${BASE_URL}/${id}`, data);
}

export function deleteRole(id: number) {
  return api.delete<boolean>(BASE_URL, { ids: [id] });
}

export function updateRolePermission(id: number, data: RolePermissionCommand) {
  return api.put<boolean>(`${BASE_URL}/${id}/permission`, data);
}

export function listRoleUser(id: number, query: RoleUserPageQuery = {}) {
  return api.get<PageResult<RoleUserResp>>(`${BASE_URL}/${id}/user`, toQuery(query));
}

export function assignToUsers(id: number, userIds: number[]) {
  return api.post<boolean>(`${BASE_URL}/${id}/user`, userIds);
}

export function unassignFromUsers(userRoleIds: number[]) {
  return api.delete<boolean>(`${BASE_URL}/user`, userRoleIds);
}

export function listRoleUserId(id: number) {
  return api.get<number[]>(`${BASE_URL}/${id}/user/id`);
}

function toQuery(query: Partial<RoleQuery | RoleUserPageQuery>): Record<string, unknown> {
  return { ...query };
}
