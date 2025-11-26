/**
 * 文件管理相关 DTO，字段与前端 pc-admin-vue3/src/apis/system/type.ts 中的 FileItem/FilePageQuery 对齐，
 * 并参考 Go 版 internal/interfaces/http/file_handler.go 的 FileItem 定义。
 */
export interface FileItem {
  id: number;
  name: string;
  originalName: string;
  size: number;
  url: string;
  parentPath: string;
  path: string;
  sha256: string;
  contentType: string;
  metadata: string;
  thumbnailSize: number;
  thumbnailName: string;
  thumbnailMetadata: string;
  thumbnailUrl: string;
  extension: string;
  type: number;
  storageId: number;
  storageName: string;
  createUserString: string;
  createTime: string;
  updateUserString: string;
  updateTime: string;
}

export interface FileListQuery {
  originalName?: string;
  type?: string;
  parentPath?: string;
  page?: string;
  size?: string;
}

/**
 * 文件夹计算大小响应，与前端 FileDirCalcSizeResp 对齐。
 */
export interface FileDirCalcSizeResp {
  size: number;
}

/**
 * 文件资源统计响应，字段与前端 FileStatisticsResp 对齐。
 */
export interface FileStatisticsResp {
  type: string;
  size: number;
  number: number;
  unit: string;
  data: FileStatisticsResp[];
}

/**
 * 文件上传响应结构，兼容前端与 Go/Java FileUploadResp。
 */
export interface FileUploadResp {
  id: string;
  url: string;
  thUrl: string;
  metadata: Record<string, string>;
}
