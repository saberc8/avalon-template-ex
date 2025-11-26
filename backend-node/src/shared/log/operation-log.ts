import { PrismaService } from '../prisma/prisma.service';
import { nextId } from '../id/id';

/**
 * 系统操作日志写入参数。
 * 用于将管理端各类操作统一落盘到 sys_log 表，便于“系统日志”页面展示。
 */
export interface OperationLogParams {
  /** 原始请求对象，用于提取 URL、方法、请求头、IP、User-Agent 等信息。 */
  req: any;
  /** 当前登录用户 ID，没有则传 0。 */
  userId: number;
  /** 所属模块名称，例如“用户管理”“角色管理”“部门管理”等。 */
  module: string;
  /** 操作描述，例如“新增用户[张三]”“修改角色[管理员]”。 */
  description: string;
  /** 是否执行成功（成功：1，失败：2）。 */
  success: boolean;
  /** 失败时的错误信息，成功时可为空字符串。 */
  message?: string;
  /** 本次操作耗时（毫秒）。 */
  timeTakenMs: number;
}

/**
 * 将一条操作日志写入 sys_log 表。
 * 失败时静默忽略，不影响业务流程。
 */
export async function writeOperationLog(
  prisma: PrismaService,
  params: OperationLogParams,
): Promise<void> {
  try {
    const id = nextId();
    const now = new Date();
    const req = params.req || {};
    const url: string =
      (req.originalUrl as string | undefined) ||
      (req.url as string | undefined) ||
      '';
    const method: string =
      (req.method as string | undefined) || 'GET';
    const headers = req.headers || {};
    const reqHeadersJson = (() => {
      try {
        return JSON.stringify(headers);
      } catch {
        return '';
      }
    })();
    const ipHeader =
      (req.headers?.['x-forwarded-for'] as string | undefined) ||
      '';
    const realIp =
      (ipHeader.split(',')[0] || '').trim() ||
      (req.ip as string | undefined) ||
      '';
    const ua =
      (req.headers?.['user-agent'] as string | undefined) || '';
    const statusCode = params.success ? 200 : 400;
    const status = params.success ? 1 : 2;
    const errorMsg = params.success ? '' : params.message || '';

    await prisma.$executeRawUnsafe(
      `
INSERT INTO sys_log (
    id, trace_id, description, module,
    request_url, request_method, request_headers, request_body,
    status_code, response_headers, response_body,
    time_taken, ip, address, browser, os,
    status, error_msg, create_user, create_time
) VALUES (
    $1, $2, $3, $4,
    $5, $6, $7, $8,
    $9, $10, $11,
    $12, $13, $14, $15, $16,
    $17, $18, $19, $20
);
`,
      id,
      '',
      params.description,
      params.module,
      url,
      method,
      reqHeadersJson,
      '',
      statusCode,
      '',
      '',
      BigInt(params.timeTakenMs || 0),
      realIp.slice(0, 100),
      '',
      ua.slice(0, 100),
      '',
      status,
      errorMsg,
      params.userId ? BigInt(params.userId) : null,
      now,
    );
  } catch {
    // 忽略日志写入失败，不影响主流程。
  }
}

