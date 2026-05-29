import { api } from "@/lib/api";
import type { PageResult } from "@/types/api";
import type { ClientCommand, ClientQuery, ClientResp, IdResponse } from "@/types/system";

const BASE_URL = "/system/client";

export function listClient(query: ClientQuery) {
  return api.get<PageResult<ClientResp>>(BASE_URL, { ...query });
}

export function getClient(id: number) {
  return api.get<ClientResp>(`${BASE_URL}/${id}`);
}

export function addClient(data: ClientCommand) {
  return api.post<IdResponse>(BASE_URL, data);
}

export function updateClient(id: number, data: ClientCommand) {
  return api.put<boolean>(`${BASE_URL}/${id}`, data);
}

export function deleteClient(id: number) {
  return api.delete<boolean>(BASE_URL, { ids: [id] });
}
