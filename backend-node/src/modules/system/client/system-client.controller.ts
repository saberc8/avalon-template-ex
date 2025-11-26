import { Body, Controller, Delete, Get, Headers, Param, Post, Put, Query } from '@nestjs/common';
import { PrismaService } from '../../../shared/prisma/prisma.service';
import { ok, fail } from '../../../shared/api-response/api-response';
import { PageResult, IdsRequest } from '../user/dto';
import { TokenService } from '../../auth/jwt/jwt.service';
import { nextId } from '../../../shared/id/id';
import { ClientResp, ClientDetailResp, ClientQuery, ClientReq } from './dto';

/**
 * 客户端配置管理接口集合，实现 /system/client 系列接口，
 * 行为参考 backend-go/internal/interfaces/http/client_handler.go。
 */
@Controller()
export class SystemClientController {
  constructor(
    private readonly prisma: PrismaService,
    private readonly tokenService: TokenService,
  ) {}

  /** 从 Authorization 解析当前用户 ID，失败时返回 0。 */
  private currentUserId(authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) return 0;
    return claims.userId;
  }

  /** GET /system/client 客户端分页列表 */
  @Get('/system/client')
  async listClient(@Query() query: ClientQuery) {
    try {
      const clientType = (query.clientType || '').trim();
      const statusStr = (query.status || '').trim();
      const sortArr = query.sort || [];

      let page = Number(query.page ?? '1');
      let size = Number(query.size ?? '10');
      if (!Number.isFinite(page) || page <= 0) page = 1;
      if (!Number.isFinite(size) || size <= 0) size = 10;

      const authTypes = Array.isArray(query.authType)
        ? query.authType.filter((v) => typeof v === 'string' && v.trim())
        : [];

      let where = 'WHERE 1=1';
      const args: any[] = [];
      let argPos = 1;

      if (clientType) {
        where += ` AND c.client_type = $${argPos}`;
        args.push(clientType);
        argPos++;
      }

      if (statusStr) {
        const st = Number(statusStr) || 0;
        if (st) {
          where += ` AND c.status = $${argPos}`;
          args.push(st);
          argPos++;
        }
      }

      if (authTypes.length) {
        // auth_type 存储为 JSON 数组字符串，这里采用 LIKE 简单匹配，与 Go 版逻辑等价（Go 中通过 JSON 解析后过滤）。
        const conds: string[] = [];
        for (const at of authTypes) {
          conds.push(`c.auth_type LIKE $${argPos}`);
          args.push(`%${at}%`);
          argPos++;
        }
        where += ` AND (${conds.join(' OR ')})`;
      }

      const countSql = `SELECT COUNT(*)::bigint AS total FROM sys_client AS c ${where};`;
      const countRows = await this.prisma.$queryRawUnsafe<
        { total: bigint }[]
      >(countSql, ...args);
      const total = countRows.length ? Number(countRows[0].total) : 0;
      if (!total) {
        const empty: PageResult<ClientResp> = { list: [], total: 0 };
        return ok(empty);
      }

      const offset = (page - 1) * size;
      const limitPos = argPos;
      const offsetPos = argPos + 1;
      const argsWithPage = [...args, size, offset];

      // 暂不根据 sort 参数改变排序，保持与 Go 版默认 id DESC 行为一致。
      const sql = `
SELECT c.id,
       c.client_id,
       c.client_type,
       c.auth_type,
       c.active_timeout,
       c.timeout,
       c.status,
       c.create_user,
       c.create_time,
       c.update_user,
       c.update_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_client AS c
LEFT JOIN sys_user AS cu ON cu.id = c.create_user
LEFT JOIN sys_user AS uu ON uu.id = c.update_user
${where}
ORDER BY c.id DESC
LIMIT $${limitPos} OFFSET $${offsetPos};
`;

      const rows = await this.prisma.$queryRawUnsafe<
        {
          id: bigint;
          client_id: string;
          client_type: string;
          auth_type: string | null;
          active_timeout: number;
          timeout: number;
          status: number;
          create_user: bigint | null;
          create_time: Date;
          update_user: bigint | null;
          update_time: Date | null;
          create_user_string: string;
          update_user_string: string;
        }[]
      >(sql, ...argsWithPage);

      const list: ClientResp[] = rows.map((r) => ({
        id: Number(r.id),
        clientId: r.client_id,
        clientType: r.client_type,
        authType: r.auth_type ?? '',
        activeTimeout: String(r.active_timeout ?? 0),
        timeout: String(r.timeout ?? 0),
        status: String(r.status ?? 1),
        createUser: r.create_user_string,
        createTime: r.create_time.toISOString(),
        updateUser: r.update_user_string,
        updateTime: r.update_time ? r.update_time.toISOString() : '',
        createUserString: r.create_user_string,
        updateUserString: r.update_user_string,
      }));

      const resp: PageResult<ClientResp> = { list, total };
      return ok(resp);
    } catch {
      return fail('500', '查询客户端失败');
    }
  }

  /** GET /system/client/:id 查询客户端详情。 */
  @Get('/system/client/:id')
  async getClient(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        client_id: string;
        client_type: string;
        auth_type: string | null;
        active_timeout: number;
        timeout: number;
        status: number;
        create_user: bigint | null;
        create_time: Date;
        update_user: bigint | null;
        update_time: Date | null;
        create_user_string: string;
        update_user_string: string;
      }[]
    >`
SELECT c.id,
       c.client_id,
       c.client_type,
       c.auth_type,
       c.active_timeout,
       c.timeout,
       c.status,
       c.create_user,
       c.create_time,
       c.update_user,
       c.update_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_client AS c
LEFT JOIN sys_user AS cu ON cu.id = c.create_user
LEFT JOIN sys_user AS uu ON uu.id = c.update_user
WHERE c.id = ${BigInt(id)};
`;
    if (!rows.length) {
      return fail('404', '客户端不存在');
    }
    const r = rows[0];
    const resp: ClientDetailResp = {
      id: Number(r.id),
      clientId: r.client_id,
      clientType: r.client_type,
      authType: r.auth_type ?? '',
      activeTimeout: String(r.active_timeout ?? 0),
      timeout: String(r.timeout ?? 0),
      status: String(r.status ?? 1),
      createUser: r.create_user_string,
      createTime: r.create_time.toISOString(),
      updateUser: r.update_user_string,
      updateTime: r.update_time ? r.update_time.toISOString() : '',
      createUserString: r.create_user_string,
      updateUserString: r.update_user_string,
    };
    return ok(resp);
  }

  /** POST /system/client 新增客户端配置。 */
  @Post('/system/client')
  async createClient(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: ClientReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }

    const clientType = (body.clientType ?? '').trim();
    const authType = Array.isArray(body.authType)
      ? body.authType.filter((v) => typeof v === 'string' && v.trim())
      : [];
    if (!clientType || !authType.length) {
      return fail('400', '客户端类型和认证类型不能为空');
    }

    const activeTimeout = body.activeTimeout || 1800;
    const timeout = body.timeout || 86400;
    const status = body.status || 1;

    const clientID = nextId().toString(16);
    const now = new Date();
    const idVal = nextId();
    const authJSON = JSON.stringify(authType);

    try {
      await this.prisma.$executeRawUnsafe(
        `
INSERT INTO sys_client (
    id, client_id, client_type, auth_type,
    active_timeout, timeout, status,
    create_user, create_time
) VALUES (
    $1, $2, $3, $4,
    $5, $6, $7,
    $8, $9
);
`,
        idVal,
        clientID,
        clientType,
        authJSON,
        activeTimeout,
        timeout,
        status,
        BigInt(currentUserId),
        now,
      );
    } catch {
      return fail('500', '新增客户端失败');
    }

    return ok({ id: idVal.toString() });
  }

  /** PUT /system/client/:id 修改客户端配置。 */
  @Put('/system/client/:id')
  async updateClient(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: ClientReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const clientType = (body.clientType ?? '').trim();
    const authType = Array.isArray(body.authType)
      ? body.authType.filter((v) => typeof v === 'string' && v.trim())
      : [];
    if (!clientType || !authType.length) {
      return fail('400', '客户端类型和认证类型不能为空');
    }

    const activeTimeout = body.activeTimeout || 1800;
    const timeout = body.timeout || 86400;
    const status = body.status || 1;
    const authJSON = JSON.stringify(authType);

    try {
      await this.prisma.$executeRawUnsafe(
        `
UPDATE sys_client
   SET client_type    = $1,
       auth_type      = $2,
       active_timeout = $3,
       timeout        = $4,
       status         = $5,
       update_user    = $6,
       update_time    = $7
 WHERE id             = $8;
`,
        clientType,
        authJSON,
        activeTimeout,
        timeout,
        status,
        BigInt(currentUserId),
        new Date(),
        BigInt(id),
      );
    } catch {
      return fail('500', '修改客户端失败');
    }

    return ok(true);
  }

  /** DELETE /system/client 删除客户端配置（批量）。 */
  @Delete('/system/client')
  async deleteClient(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: IdsRequest,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    if (!body?.ids?.length) {
      return fail('400', 'ID 列表不能为空');
    }
    const ids = body.ids.filter((v) => Number.isFinite(v) && v > 0);
    if (!ids.length) {
      return fail('400', 'ID 列表不能为空');
    }

    try {
      await this.prisma.$executeRawUnsafe(
        `DELETE FROM sys_client WHERE id = ANY($1::bigint[]);`,
        ids.map((v) => BigInt(v)),
      );
    } catch {
      return fail('500', '删除客户端失败');
    }

    return ok(true);
  }
}

