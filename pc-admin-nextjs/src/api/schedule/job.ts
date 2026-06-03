import { api } from "@/lib/api";
import type { PageResult } from "@/types/api";
import type { IdResponse } from "@/types/system";
import type {
  JobCommand,
  JobLogQuery,
  JobLogResp,
  JobQuery,
  JobResp,
  JobTriggerResp
} from "@/types/schedule";

const BASE_URL = "/schedule/job";

export function listJob(query: JobQuery = {}) {
  return api.get<PageResult<JobResp>>(`${BASE_URL}/page`, { ...query });
}

export function getJob(id: number) {
  return api.get<JobResp>(`${BASE_URL}/${id}`);
}

export function addJob(data: JobCommand) {
  return api.post<IdResponse>(BASE_URL, data);
}

export function updateJob(id: number, data: JobCommand) {
  return api.put<boolean>(`${BASE_URL}/${id}`, data);
}

export function deleteJob(id: number) {
  return api.delete<boolean>(BASE_URL, { ids: [id] });
}

export function updateJobStatus(id: number, status: number) {
  return api.patch<boolean>(`${BASE_URL}/${id}/status`, { status });
}

export function runJob(id: number) {
  return api.post<JobTriggerResp>(`${BASE_URL}/${id}/run`);
}

export function listJobLog(id: number, query: JobLogQuery = {}) {
  return api.get<PageResult<JobLogResp>>(`${BASE_URL}/${id}/log`, { ...query });
}
