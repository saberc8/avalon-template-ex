/**
 * 统一 API 响应结构，与前端 ApiRes<T>、Go/Python 版 APIResponse 完全对齐。
 */
export interface ApiResponse<T> {
  code: string;
  data: T;
  msg: string;
  success: boolean;
  timestamp: string;
}

/**
 * 返回当前时间戳（毫秒）的字符串表示，前端使用 Number(timestamp)。
 */
function nowString(): string {
  return Date.now().toString();
}

/**
 * 成功响应包装。
 */
export function ok<T>(data: T): ApiResponse<T> {
  return {
    code: '200',
    data,
    msg: '操作成功',
    success: true,
    timestamp: nowString(),
  };
}

/**
 * 失败响应包装。
 */
export function fail(code: string, msg: string): ApiResponse<null> {
  return {
    code,
    data: null,
    msg,
    success: false,
    timestamp: nowString(),
  };
}

