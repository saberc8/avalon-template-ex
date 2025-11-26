import { Body, Controller, Delete, Get, Headers, Param, Post, Put, Query } from '@nestjs/common';
import { PrismaService } from '../../../shared/prisma/prisma.service';
import { ok, fail } from '../../../shared/api-response/api-response';
import { StorageResp, StorageQuery, StorageStatusUpdateReq, StorageReq } from './dto';
import { IdsRequest } from '../user/dto';
import { nextId } from '../../../shared/id/id';
import { TokenService } from '../../auth/jwt/jwt.service';

/**
 * 存储配置管理接口集合，实现 /system/storage 系列接口中的列表查询，
 * 行为参考 backend-go/internal/interfaces/http/storage_handler.go#ListStorage。
 */
@Controller()
export class SystemStorageController {
  constructor(
    private readonly prisma: PrismaService,
    private readonly tokenService: TokenService,
  ) {}

  /** 从 Authorization 解析当前用户 ID，失败返回 0。 */
  private currentUserId(authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) return 0;
    return claims.userId;
  }

  /** GET /system/storage/list 存储配置列表 */
  @Get('/system/storage/list')
  async listStorage(@Query() query: StorageQuery) {
    try {
      const description = (query.description || '').trim();
      const typeVal =
        typeof query.type === 'number'
          ? query.type
          : Number((query.type as any) ?? 0) || 0;

      let where = 'WHERE 1=1';
      const params: any[] = [];
      let idx = 1;

      if (description) {
        where +=
          ` AND (s.name ILIKE $${idx}` +
          ` OR s.code ILIKE $${idx}` +
          ` OR COALESCE(s.description,'') ILIKE $${idx})`;
        params.push(`%${description}%`);
        idx++;
      }
      if (typeVal) {
        where += ` AND s.type = $${idx}`;
        params.push(typeVal);
        idx++;
      }

      const sql = `
SELECT s.id,
       s.name,
       s.code,
       s.type,
       COALESCE(s.access_key, '')   AS access_key,
       COALESCE(s.region, '')       AS region,
       COALESCE(s.endpoint, '')     AS endpoint,
       s.bucket_name,
       COALESCE(s.domain, '')       AS domain,
       COALESCE(s.description, '')  AS description,
       s.is_default,
       COALESCE(s.sort, 999)        AS sort,
       s.status,
       s.create_time,
       COALESCE(cu.nickname, '')    AS create_user_string,
       s.update_time,
       COALESCE(uu.nickname, '')    AS update_user_string
FROM sys_storage AS s
LEFT JOIN sys_user AS cu ON cu.id = s.create_user
LEFT JOIN sys_user AS uu ON uu.id = s.update_user
${where}
ORDER BY s.sort ASC, s.id ASC;
`;

      const rows = await this.prisma.$queryRawUnsafe<
        {
          id: bigint;
          name: string;
          code: string;
          type: number;
          access_key: string;
          region: string;
          endpoint: string;
          bucket_name: string;
          domain: string;
          description: string;
          is_default: boolean;
          sort: number;
          status: number;
          create_time: Date;
          create_user_string: string;
          update_time: Date | null;
          update_user_string: string;
        }[]
      >(sql, ...params);

      const list: StorageResp[] = rows.map((r) => ({
        id: Number(r.id),
        name: r.name,
        code: r.code,
        type: Number(r.type),
        accessKey: r.access_key,
        secretKey: '',
        endpoint: r.endpoint,
        region: r.region,
        bucketName: r.bucket_name,
        domain: r.domain,
        description: r.description,
        isDefault: !!r.is_default,
        sort: Number(r.sort ?? 999),
        status: Number(r.status),
        createUserString: r.create_user_string,
        createTime: r.create_time.toISOString(),
        updateUserString: r.update_user_string,
        updateTime: r.update_time ? r.update_time.toISOString() : '',
      }));

      return ok(list);
    } catch {
      return fail('500', '查询存储配置失败');
    }
  }

  /** PUT /system/storage/:id/status 修改存储状态，仅允许启用/禁用非默认存储。 */
  @Put('/system/storage/:id/status')
  async updateStatus(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: StorageStatusUpdateReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }
    const status = Number(body?.status ?? 0);
    if (status !== 1 && status !== 2) {
      return fail('400', '状态参数不正确');
    }

    // 默认存储不允许被禁用
    const rows = await this.prisma.$queryRaw<
      { is_default: boolean | null }[]
    >`SELECT is_default FROM sys_storage WHERE id = ${BigInt(id)};`;
    if (!rows.length) {
      return fail('404', '存储配置不存在');
    }
    const isDefault = !!rows[0].is_default;
    if (isDefault && status !== 1) {
      return fail('400', '不允许禁用默认存储');
    }

    try {
      await this.prisma.$executeRawUnsafe(
        `
UPDATE sys_storage
   SET status = $1,
       update_user = $2,
       update_time = $3
 WHERE id = $4;
`,
        status,
        BigInt(currentUserId),
        new Date(),
        BigInt(id),
      );
    } catch {
      return fail('500', '更新存储状态失败');
    }

    return ok(true);
  }

  /** PUT /system/storage/:id/default 设为默认存储。 */
  @Put('/system/storage/:id/default')
  async setDefault(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    try {
      await this.prisma.$executeRawUnsafe(
        `UPDATE sys_storage SET is_default = FALSE;`,
      );
      await this.prisma.$executeRawUnsafe(
        `
UPDATE sys_storage
   SET is_default = TRUE,
       update_user = $1,
       update_time = $2
 WHERE id = $3;
`,
        BigInt(currentUserId),
        new Date(),
        BigInt(id),
      );
    } catch {
      return fail('500', '设为默认存储失败');
    }

    return ok(true);
  }

  /** GET /system/storage/:id 查询存储配置详情。 */
  @Get('/system/storage/:id')
  async getStorage(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        name: string;
        code: string;
        type: number;
        access_key: string;
        secret_key: string;
        endpoint: string;
        region: string;
        bucket_name: string;
        domain: string;
        description: string;
        is_default: boolean;
        sort: number;
        status: number;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >`
SELECT s.id,
       s.name,
       s.code,
       s.type,
       COALESCE(s.access_key, '')   AS access_key,
       COALESCE(s.secret_key, '')   AS secret_key,
       COALESCE(s.endpoint, '')     AS endpoint,
       COALESCE(s.region, '')       AS region,
       s.bucket_name,
       COALESCE(s.domain, '')       AS domain,
       COALESCE(s.description, '')  AS description,
       s.is_default,
       COALESCE(s.sort, 999)        AS sort,
       s.status,
       s.create_time,
       COALESCE(cu.nickname, '')    AS create_user_string,
       s.update_time,
       COALESCE(uu.nickname, '')    AS update_user_string
FROM sys_storage AS s
LEFT JOIN sys_user AS cu ON cu.id = s.create_user
LEFT JOIN sys_user AS uu ON uu.id = s.update_user
WHERE s.id = ${BigInt(id)};
`;
    if (!rows.length) {
      return fail('404', '存储配置不存在');
    }
    const r = rows[0];
    const resp: StorageResp = {
      id: Number(r.id),
      name: r.name,
      code: r.code,
      type: Number(r.type),
      accessKey: r.access_key,
      // 详情场景不返回真实密钥，只返回掩码，行为与 Go/Java 一致。
      secretKey: r.secret_key ? '******' : '',
      endpoint: r.endpoint,
      region: r.region,
      bucketName: r.bucket_name,
      domain: r.domain,
      description: r.description,
      isDefault: !!r.is_default,
      sort: Number(r.sort ?? 999),
      status: Number(r.status),
      createUserString: r.create_user_string,
      createTime: r.create_time.toISOString(),
      updateUserString: r.update_user_string,
      updateTime: r.update_time ? r.update_time.toISOString() : '',
    };
    return ok(resp);
  }

  /** POST /system/storage 新增存储配置。 */
  @Post('/system/storage')
  async createStorage(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: StorageReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }

    const name = (body.name ?? '').trim();
    const code = (body.code ?? '').trim();
    const bucketName = (body.bucketName ?? '').trim();
    const domain = (body.domain ?? '').trim();
    const endpoint = (body.endpoint ?? '').trim();
    const region = (body.region ?? '').trim();

    if (!name || !code) {
      return fail('400', '名称和编码不能为空');
    }
    const type = body.type || 1;
    const sort = body.sort > 0 ? body.sort : 999;
    const status = body.status || 1;

    // 编码唯一性校验
    const codeRows = await this.prisma.$queryRaw<
      { exists: number }[]
    >`SELECT 1::int AS exists FROM sys_storage WHERE code = ${code} LIMIT 1;`;
    if (codeRows.length) {
      return fail('400', '存储编码已存在');
    }

    // 对象存储需要解密 SecretKey；当前 Node 版暂不实现 RSA 解密，这里直接存空或明文。
    let secretVal = '';
    if (type === 2) {
      const encrypted = (body.secretKey ?? '').trim();
      if (encrypted.length > 255) {
        return fail('400', '私有密钥长度不能超过 255 个字符');
      }
      secretVal = encrypted;
    }

    const now = new Date();
    const id = nextId();
    const isDefault = !!body.isDefault;

    try {
      await this.prisma.$executeRawUnsafe(
        `
INSERT INTO sys_storage (
    id, name, code, type, access_key, secret_key, endpoint,
    region,
    bucket_name, domain, description, is_default, sort, status,
    create_user, create_time
) VALUES (
    $1, $2, $3, $4, $5, $6, $7,
    $8,
    $9, $10, $11, $12, $13, $14,
    $15, $16
);
`,
        id,
        name,
        code,
        type,
        body.accessKey ?? '',
        secretVal,
        endpoint,
        region,
        bucketName,
        domain,
        body.description ?? '',
        isDefault,
        sort,
        status,
        BigInt(currentUserId),
        now,
      );
    } catch {
      return fail('500', '新增存储配置失败');
    }

    return ok({ id: id.toString() });
  }

  /** PUT /system/storage/:id 修改存储配置。 */
  @Put('/system/storage/:id')
  async updateStorage(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: StorageReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const name = (body.name ?? '').trim();
    const bucketName = (body.bucketName ?? '').trim();
    const domain = (body.domain ?? '').trim();
    const endpoint = (body.endpoint ?? '').trim();
    const region = (body.region ?? '').trim();

    if (!name) {
      return fail('400', '名称不能为空');
    }
    const type = body.type || 1;
    const sort = body.sort > 0 ? body.sort : 999;
    const status = body.status || 1;

    // 查询旧的 secret_key
    const oldRows = await this.prisma.$queryRaw<
      { secret_key: string | null }[]
    >`SELECT COALESCE(secret_key, '') AS secret_key FROM sys_storage WHERE id = ${BigInt(id)};`;
    if (!oldRows.length) {
      return fail('404', '存储配置不存在');
    }
    const oldSecret = oldRows[0].secret_key ?? '';

    // 根据请求判断是否需要更新密钥：为空则保持原值。
    let secretVal = oldSecret;
    const encrypted = (body.secretKey ?? '').trim();
    if (encrypted) {
      if (encrypted.length > 255) {
        return fail('400', '私有密钥长度不能超过 255 个字符');
      }
      secretVal = encrypted;
    }

    const now = new Date();

    try {
      if (type === 2) {
        await this.prisma.$executeRawUnsafe(
          `
UPDATE sys_storage
   SET name = $1,
       type = $2,
       access_key = $3,
       secret_key = $4,
       endpoint = $5,
       region = $6,
       bucket_name = $7,
       domain = $8,
       description = $9,
       sort = $10,
       status = $11,
       update_user = $12,
       update_time = $13
 WHERE id = $14;
`,
          name,
          type,
          body.accessKey ?? '',
          secretVal,
          endpoint,
          region,
          bucketName,
          domain,
          body.description ?? '',
          sort,
          status,
          BigInt(currentUserId),
          now,
          BigInt(id),
        );
      } else {
        await this.prisma.$executeRawUnsafe(
          `
UPDATE sys_storage
   SET name = $1,
       type = $2,
       access_key = $3,
       endpoint = $4,
       region = $5,
       bucket_name = $6,
       domain = $7,
       description = $8,
       sort = $9,
       status = $10,
       update_user = $11,
       update_time = $12
 WHERE id = $13;
`,
          name,
          type,
          body.accessKey ?? '',
          endpoint,
          region,
          bucketName,
          domain,
          body.description ?? '',
          sort,
          status,
          BigInt(currentUserId),
          now,
          BigInt(id),
        );
      }
    } catch {
      return fail('500', '修改存储配置失败');
    }

    return ok(true);
  }

  /** DELETE /system/storage 删除存储配置（批量）。 */
  @Delete('/system/storage')
  async deleteStorage(
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

    // 不允许删除默认存储
    const rows = await this.prisma.$queryRaw<
      { id: bigint; is_default: boolean | null }[]
    >`SELECT id, is_default FROM sys_storage WHERE id = ANY(${ids as any}::bigint[]);`;
    for (const r of rows) {
      if (r.is_default) {
        return fail('400', '不允许删除默认存储');
      }
    }

    try {
      await this.prisma.$executeRawUnsafe(
        `DELETE FROM sys_storage WHERE id = ANY($1::bigint[]);`,
        ids.map((v) => BigInt(v)),
      );
    } catch {
      return fail('500', '删除存储配置失败');
    }

    return ok(true);
  }
}
