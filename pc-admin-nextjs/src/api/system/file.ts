import { api } from "@/lib/api";
import type { PageResult } from "@/types/api";
import type { FileDirCalcSizeResp, FileItem, FileQuery, FileStatisticsResp } from "@/types/system";

const BASE_URL = "/system/file";

export function uploadFile(data: FormData) {
  return api.post<FileItem>(`${BASE_URL}/upload`, data);
}

export function listFile(query: FileQuery) {
  return api.get<PageResult<FileItem>>(BASE_URL, { ...query });
}

export function updateFile(id: number, originalName: string) {
  return api.put<boolean>(`${BASE_URL}/${id}`, { originalName });
}

export function deleteFile(ids: number[]) {
  return api.delete<boolean>(BASE_URL, { ids });
}

export function getFileStatistics() {
  return api.get<FileStatisticsResp>(`${BASE_URL}/statistics`);
}

export function checkFile(sha256: string) {
  return api.get<FileItem | null>(`${BASE_URL}/check`, { fileHash: sha256 });
}

export function createDir(parentPath: string, originalName: string) {
  return api.post<FileItem>(`${BASE_URL}/dir`, { parentPath, originalName });
}

export function calcDirSize(id: number) {
  return api.get<FileDirCalcSizeResp>(`${BASE_URL}/dir/${id}/size`);
}
