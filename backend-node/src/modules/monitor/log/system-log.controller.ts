import { Controller, Get, Param, Query, Res } from '@nestjs/common';
import { PrismaService } from '../../../shared/prisma/prisma.service';
import { ok, fail } from '../../../shared/api-response/api-response';
import { PageResult } from '../../system/user/dto';
import { LogDetailResp, LogQuery, LogResp } from './dto';

/**
 * 系统日志接口集合，实现 /system/log 系列接口：
 * - GET /system/log           分页查询日志
 * - GET /system/log/:id       查询日志详情
 * - GET /system/log/export/login     导出登录日志
 * - GET /system/log/export/operation 导出操作日志
 *
 * 行为参考 backend-go/internal/interfaces/http/log_handler.go。
 */
@Controller()
export class SystemLogController {
  constructor(private readonly prisma: PrismaService) {}

  /** GET /system/log 日志分页列表。 */
  @Get('/system/log')
  async listLog(@Query() query: LogQuery) {
    try {
      const desc = (query.description || '').trim();
      const module = (query.module || '').trim();
      const ip = (query.ip || '').trim();
      const createUser = (query.createUserString || '').trim();
      const statusFilter = Number(query.status ?? 0) || 0;

      let page = Number(query.page ?? '1');
      let size = Number(query.size ?? '10');
      if (!Number.isFinite(page) || page <= 0) page = 1;
      if (!Number.isFinite(size) || size <= 0) size = 10;

      const range = query.createTime;
      const arr = Array.isArray(range) ? range : range ? [range] : [];
      let startTime: Date | null = null;
      let endTime: Date | null = null;
      if (arr.length === 2) {
        const s = parseDateTime(arr[0]);
        const e = parseDateTime(arr[1]);
        if (s) startTime = s;
        if (e) endTime = e;
      }

      const baseFrom = `
FROM sys_log AS t1
LEFT JOIN sys_user AS t2 ON t2.id = t1.create_user
`;

      let where = 'WHERE 1=1';
      const args: any[] = [];
      let argPos = 1;

      if (desc) {
        where += ` AND t1.description ILIKE $${argPos}`;
        args.push(`%${desc}%`);
        argPos++;
      }
      if (module) {
        where += ` AND t1.module = $${argPos}`;
        args.push(module);
        argPos++;
      }
      if (ip) {
        where += ` AND COALESCE(t1.ip,'') ILIKE $${argPos}`;
        args.push(`%${ip}%`);
        argPos++;
      }
      if (createUser) {
        where += ` AND COALESCE(t2.nickname,'') ILIKE $${argPos}`;
        args.push(`%${createUser}%`);
        argPos++;
      }
      if (statusFilter !== 0) {
        where += ` AND COALESCE(t1.status,1) = $${argPos}`;
        args.push(statusFilter);
        argPos++;
      }
      if (startTime && endTime) {
        where += ` AND t1.create_time BETWEEN $${argPos} AND $${argPos + 1}`;
        args.push(startTime, endTime);
        argPos += 2;
      }

      const countSql = `SELECT COUNT(*)::bigint AS total ${baseFrom} ${where};`;
      const countRows = await this.prisma.$queryRawUnsafe<
        { total: bigint }[]
      >(countSql, ...args);
      const total = countRows.length ? Number(countRows[0].total) : 0;
      if (!total) {
        const empty: PageResult<LogResp> = { list: [], total: 0 };
        return ok(empty);
      }

      const offset = (page - 1) * size;
      const limitPos = argPos;
      const offsetPos = argPos + 1;
      const argsWithPage = [...args, size, offset];

      const sql = `
SELECT t1.id,
       t1.description,
       t1.module,
       COALESCE(t1.time_taken, 0)        AS time_taken,
       COALESCE(t1.ip, '')              AS ip,
       COALESCE(t1.address, '')         AS address,
       COALESCE(t1.browser, '')         AS browser,
       COALESCE(t1.os, '')              AS os,
       COALESCE(t1.status, 1)           AS status,
       COALESCE(t1.error_msg, '')       AS error_msg,
       t1.create_time,
       COALESCE(t2.nickname, '')        AS create_user_string
${baseFrom}
${where}
ORDER BY t1.create_time DESC, t1.id DESC
LIMIT $${limitPos} OFFSET $${offsetPos};
`;

      const rows = await this.prisma.$queryRawUnsafe<
        {
          id: bigint;
          description: string;
          module: string;
          time_taken: number | null;
          ip: string;
          address: string;
          browser: string;
          os: string;
          status: number | null;
          error_msg: string;
          create_time: Date;
          create_user_string: string;
        }[]
      >(sql, ...argsWithPage);

      const list: LogResp[] = rows.map((r) => ({
        id: Number(r.id),
        description: r.description,
        module: r.module,
        // 防止 BigInt 直接返回导致 JSON 序列化报错，这里统一转为 number
        timeTaken: Number(r.time_taken ?? 0),
        ip: r.ip,
        address: r.address,
        browser: r.browser,
        os: r.os,
        // 状态字段也可能为 BigInt，统一转为 number
        status: Number(r.status ?? 1),
        errorMsg: r.error_msg,
        createUserString: r.create_user_string,
        createTime: r.create_time.toISOString(),
      }));

      const resp: PageResult<LogResp> = { list, total };
      return ok(resp);
    } catch {
      return fail('500', '查询日志失败');
    }
  }

  /** GET /system/log/:id 日志详情。 */
  @Get('/system/log/:id')
  async getLog(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        trace_id: string;
        description: string;
        module: string;
        request_url: string;
        request_method: string;
        request_headers: string;
        request_body: string;
        status_code: number;
        response_headers: string;
        response_body: string;
        time_taken: number | null;
        ip: string;
        address: string;
        browser: string;
        os: string;
        status: number | null;
        error_msg: string;
        create_time: Date;
        create_user_string: string;
      }[]
    >`
SELECT t1.id,
       COALESCE(t1.trace_id, '')          AS trace_id,
       t1.description,
       t1.module,
       t1.request_url,
       t1.request_method,
       COALESCE(t1.request_headers, '')   AS request_headers,
       COALESCE(t1.request_body, '')      AS request_body,
       t1.status_code,
       COALESCE(t1.response_headers, '')  AS response_headers,
       COALESCE(t1.response_body, '')     AS response_body,
       COALESCE(t1.time_taken, 0)         AS time_taken,
       COALESCE(t1.ip, '')                AS ip,
       COALESCE(t1.address, '')           AS address,
       COALESCE(t1.browser, '')           AS browser,
       COALESCE(t1.os, '')                AS os,
       COALESCE(t1.status, 1)             AS status,
       COALESCE(t1.error_msg, '')         AS error_msg,
       t1.create_time,
       COALESCE(t2.nickname, '')          AS create_user_string
FROM sys_log AS t1
LEFT JOIN sys_user AS t2 ON t2.id = t1.create_user
WHERE t1.id = ${BigInt(id)};
`;
    if (!rows.length) {
      return fail('404', '日志不存在');
    }
    const r = rows[0];
    const resp: LogDetailResp = {
      id: Number(r.id),
      traceId: r.trace_id,
      description: r.description,
      module: r.module,
      requestUrl: r.request_url,
      requestMethod: r.request_method,
      requestHeaders: r.request_headers,
      requestBody: r.request_body,
      // 统一将可能为 BigInt 的字段转为 number，避免 JSON 序列化失败
      statusCode: Number(r.status_code),
      responseHeaders: r.response_headers,
      responseBody: r.response_body,
      timeTaken: Number(r.time_taken ?? 0),
      ip: r.ip,
      address: r.address,
      browser: r.browser,
      os: r.os,
      status: Number(r.status ?? 1),
      errorMsg: r.error_msg,
      createUserString: r.create_user_string,
      createTime: r.create_time.toISOString(),
    };
    return ok(resp);
  }

  /** GET /system/log/export/login 导出登录日志 CSV。 */
  @Get('/system/log/export/login')
  async exportLogin(@Query() query: LogQuery, @Res() res: Response) {
    return this.exportCSV(query, res, true);
  }

  /** GET /system/log/export/operation 导出操作日志 CSV。 */
  @Get('/system/log/export/operation')
  async exportOperation(@Query() query: LogQuery, @Res() res: Response) {
    return this.exportCSV(query, res, false);
  }

  /** 公共导出逻辑，行为参考 Go 版 ExportLoginLog/ExportOperationLog。 */
  private async exportCSV(
    query: LogQuery,
    res: any,
    isLogin: boolean,
  ): Promise<void> {
    try {
      const desc = (query.description || '').trim();
      const module = (query.module || '').trim();
      const ip = (query.ip || '').trim();
      const createUser = (query.createUserString || '').trim();
      const statusFilter = Number(query.status ?? 0) || 0;

      const range = query.createTime;
      const arr = Array.isArray(range) ? range : range ? [range] : [];
      let startTime: Date | null = null;
      let endTime: Date | null = null;
      if (arr.length === 2) {
        const s = parseDateTime(arr[0]);
        const e = parseDateTime(arr[1]);
        if (s) startTime = s;
        if (e) endTime = e;
      }

      const baseFrom = `
FROM sys_log AS t1
LEFT JOIN sys_user AS t2 ON t2.id = t1.create_user
`;

      let where = 'WHERE 1=1';
      const args: any[] = [];
      let argPos = 1;

      if (desc) {
        where += ` AND t1.description ILIKE $${argPos}`;
        args.push(`%${desc}%`);
        argPos++;
      }
      if (module) {
        where += ` AND t1.module = $${argPos}`;
        args.push(module);
        argPos++;
      }
      if (ip) {
        where += ` AND COALESCE(t1.ip,'') ILIKE $${argPos}`;
        args.push(`%${ip}%`);
        argPos++;
      }
      if (createUser) {
        where += ` AND COALESCE(t2.nickname,'') ILIKE $${argPos}`;
        args.push(`%${createUser}%`);
        argPos++;
      }
      if (statusFilter !== 0) {
        where += ` AND COALESCE(t1.status,1) = $${argPos}`;
        args.push(statusFilter);
        argPos++;
      }
      if (startTime && endTime) {
        where += ` AND t1.create_time BETWEEN $${argPos} AND $${argPos + 1}`;
        args.push(startTime, endTime);
        argPos += 2;
      }

      const selectSql = `
SELECT t1.id,
       t1.create_time,
       COALESCE(t2.nickname, '') AS user_nick,
       t1.description,
       t1.module,
       COALESCE(t1.status, 1)    AS status,
       COALESCE(t1.ip, '')       AS ip,
       COALESCE(t1.address, '')  AS address,
       COALESCE(t1.browser, '')  AS browser,
       COALESCE(t1.os, '')       AS os,
       COALESCE(t1.time_taken,0) AS time_taken
`;

      const querySql = `${selectSql}${baseFrom}${where} ORDER BY t1.create_time DESC, t1.id DESC;`;
      const rows = await this.prisma.$queryRawUnsafe<
        {
          id: bigint;
          create_time: Date;
          user_nick: string;
          description: string;
          module: string;
          status: number;
          ip: string;
          address: string;
          browser: string;
          os: string;
          time_taken: number;
        }[]
      >(querySql, ...args);

      res.setHeader('Content-Type', 'text/csv; charset=utf-8');
      res.setHeader(
        'Content-Disposition',
        isLogin
          ? 'attachment; filename=\"login-log.csv\"'
          : 'attachment; filename=\"operation-log.csv\"',
      );

      if (!rows.length) {
        res.write('');
        res.end();
        return;
      }

      const lines: string[] = [];
      if (isLogin) {
        // 登录日志导出列
        lines.push(
          'ID,登录时间,用户昵称,登录行为,状态,登录 IP,登录地点,浏览器,终端系统',
        );
        for (const r of rows) {
          const statusText = r.status === 1 ? '成功' : '失败';
          lines.push(
            [
              String(r.id),
              formatDateTime(r.create_time),
              escapeCSV(r.user_nick),
              escapeCSV(r.description),
              statusText,
              escapeCSV(r.ip),
              escapeCSV(r.address),
              escapeCSV(r.browser),
              escapeCSV(r.os),
            ].join(','),
          );
        }
      } else {
        // 操作日志导出列
        lines.push(
          'ID,操作时间,操作人,操作内容,所属模块,状态,操作 IP,操作地点,耗时（ms）,浏览器,终端系统',
        );
        for (const r of rows) {
          const statusText = r.status === 1 ? '成功' : '失败';
          lines.push(
            [
              String(r.id),
              formatDateTime(r.create_time),
              escapeCSV(r.user_nick),
              escapeCSV(r.description),
              escapeCSV(r.module),
              statusText,
              escapeCSV(r.ip),
              escapeCSV(r.address),
              String(r.time_taken),
              escapeCSV(r.browser),
              escapeCSV(r.os),
            ].join(','),
          );
        }
      }

      res.write(lines.join('\n'));
      res.end();
    } catch {
      res.status(500).send('导出日志失败');
    }
  }
}

function pad(num: number): string {
  return num < 10 ? `0${num}` : `${num}`;
}

function formatDateTime(date: Date): string {
  const y = date.getFullYear();
  const m = pad(date.getMonth() + 1);
  const d = pad(date.getDate());
  const hh = pad(date.getHours());
  const mm = pad(date.getMinutes());
  const ss = pad(date.getSeconds());
  return `${y}-${m}-${d} ${hh}:${mm}:${ss}`;
}

function parseDateTime(value: string | undefined): Date | null {
  if (!value) return null;
  const s = value.trim();
  if (!s) return null;
  const iso = s.replace(' ', 'T');
  const d = new Date(iso);
  // eslint-disable-next-line no-restricted-globals
  if (isNaN(d.getTime())) return null;
  return d;
}

/** 对包含逗号或引号的字段进行简单 CSV 转义。 */
function escapeCSV(val: string): string {
  if (!val) return '';
  if (!/[,"\n\r]/.test(val)) {
    return val;
  }
  const escaped = val.replace(/"/g, '""');
  return `"${escaped}"`;
}
