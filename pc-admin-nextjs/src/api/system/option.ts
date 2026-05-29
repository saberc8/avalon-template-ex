import { api } from "@/lib/api";
import type { OptionQuery, OptionResp, OptionUpdateItem } from "@/types/system";

const BASE_URL = "/system/option";

export function listOption(query: OptionQuery) {
  return api.get<OptionResp[]>(BASE_URL, { ...query });
}

export function updateOption(data: OptionUpdateItem[]) {
  return api.put<boolean>(BASE_URL, data);
}

export function resetOptionValue(query: OptionQuery) {
  return api.patch<boolean>(`${BASE_URL}/value`, query);
}
