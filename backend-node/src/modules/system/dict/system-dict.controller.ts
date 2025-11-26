import {
  Body,
  Controller,
  Delete,
  Get,
  Headers,
  Param,
  Post,
  Put,
  Query,
  Req,
} from '@nestjs/common';
import { PrismaService } from '../../../shared/prisma/prisma.service';
import { ok, fail } from '../../../shared/api-response/api-response';
import { TokenService } from '../../auth/jwt/jwt.service';
import { nextId } from '../../../shared/id/id';
import { writeOperationLog } from '../../../shared/log/operation-log';
import {
  DictItemListQuery,
  DictItemReq,
  DictItemResp,
  DictReq,
  DictResp,
  IdsRequest,
  PageResult,
} from './dto';

/**
 * 字典管理控制器，覆盖 /system/dict 与 /system/dict/item 全部接口。
 */
@Controller()
export class SystemDictController {
  constructor(
    private readonly prisma: PrismaService,
    private readonly tokenService: TokenService,
  ) {}

  /** 解析登录用户 ID，未登录时返回 0。 */
  private currentUserId(authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) {
      return 0;
    }
    return claims.userId;
  }

  /** 查询字典列表：GET /system/dict/list */
  @Get('/system/dict/list')
  async listDict(@Query('description') description?: string) {
    const desc = (description || '').trim();
    try {
      const rows = await this.prisma.$queryRaw<
        {
          id: bigint;
          name: string;
          code: string;
          description: string;
          is_system: boolean;
          create_time: Date;
          create_user_string: string;
          update_time: Date | null;
          update_user_string: string;
        }[]
      >`
SELECT d.id,
       d.name,
       d.code,
       COALESCE(d.description, '') AS description,
       COALESCE(d.is_system, FALSE) AS is_system,
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
ORDER BY d.create_time DESC, d.id DESC;
`;

      const list: DictResp[] = [];
      for (const row of rows) {
        const item: DictResp = {
          id: Number(row.id),
          name: row.name,
          code: row.code,
          description: row.description ?? '',
          isSystem: !!row.is_system,
          createUserString: row.create_user_string,
          createTime: row.create_time.toISOString(),
          updateUserString: row.update_user_string,
          updateTime: row.update_time ? row.update_time.toISOString() : '',
        };
        if (
          desc &&
          !item.name.includes(desc) &&
          !item.description.includes(desc)
        ) {
          continue;
        }
        list.push(item);
      }
      return ok(list);
    } catch {
      return fail('500', '查询字典失败');
    }
  }

  /** 分页查询字典项：GET /system/dict/item */
  @Get('/system/dict/item')
  async listDictItem(@Query() query: DictItemListQuery) {
    let page = Number(query.page ?? '1');
    let size = Number(query.size ?? '10');
    if (!Number.isFinite(page) || page <= 0) page = 1;
    if (!Number.isFinite(size) || size <= 0) size = 10;
    const description = (query.description || '').trim();
    const statusStr = (query.status || '').trim();
    let status = 0;
    if (statusStr) {
      const parsed = Number(statusStr);
      if (Number.isFinite(parsed)) {
        status = parsed;
      }
    }
    const dictIdStr = (query.dictId || '').trim();

    const baseSql = `
SELECT di.id,
       di.label,
       di.value,
       COALESCE(di.color, '') AS color,
       COALESCE(di.sort, 999) AS sort,
       COALESCE(di.description, '') AS description,
       di.status,
       di.dict_id,
       di.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       di.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict_item AS di
LEFT JOIN sys_user AS cu ON cu.id = di.create_user
LEFT JOIN sys_user AS uu ON uu.id = di.update_user
`;
    const orderBy = 'ORDER BY di.sort ASC, di.id ASC;';

    try {
      let rows: {
        id: bigint;
        label: string;
        value: string;
        color: string;
        sort: number;
        description: string;
        status: number;
        dict_id: bigint;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[] = [];
      if (dictIdStr) {
        const dictId = Number(dictIdStr);
        if (!Number.isFinite(dictId) || dictId <= 0) {
          return fail('400', '字典 ID 不正确');
        }
        rows = await this.prisma.$queryRawUnsafe(
          `${baseSql}WHERE di.dict_id = $1 ${orderBy}`,
          BigInt(dictId),
        );
      } else {
        rows = await this.prisma.$queryRawUnsafe(`${baseSql}${orderBy}`);
      }

      const filtered: DictItemResp[] = [];
      for (const row of rows) {
        const item: DictItemResp = {
          id: Number(row.id),
          label: row.label,
          value: row.value,
          color: row.color ?? '',
          sort: Number(row.sort ?? 999),
          description: row.description ?? '',
          status: Number(row.status),
          dictId: Number(row.dict_id),
          createUserString: row.create_user_string,
          createTime: row.create_time.toISOString(),
          updateUserString: row.update_user_string,
          updateTime: row.update_time ? row.update_time.toISOString() : '',
        };
        if (
          description &&
          !item.label.includes(description) &&
          !item.description.includes(description)
        ) {
          continue;
        }
        if (status && item.status !== status) {
          continue;
        }
        filtered.push(item);
      }

      const total = filtered.length;
      const start = Math.min((page - 1) * size, total);
      const end = Math.min(start + size, total);
      const pageList = filtered.slice(start, end);
      const resp: PageResult<DictItemResp> = {
        list: pageList,
        total,
      };
      return ok(resp);
    } catch {
      return fail('500', '查询字典项失败');
    }
  }

  /** 查询字典项详情：GET /system/dict/item/:id */
  @Get('/system/dict/item/:id')
  async getDictItem(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }
    try {
      const rows = await this.prisma.$queryRaw<
        {
          id: bigint;
          label: string;
          value: string;
          color: string;
          sort: number;
          description: string;
          status: number;
          dict_id: bigint;
          create_time: Date;
          create_user_string: string;
          update_time: Date | null;
          update_user_string: string;
        }[]
      >`
SELECT di.id,
       di.label,
       di.value,
       COALESCE(di.color, '') AS color,
       COALESCE(di.sort, 999) AS sort,
       COALESCE(di.description, '') AS description,
       di.status,
       di.dict_id,
       di.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       di.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict_item AS di
LEFT JOIN sys_user AS cu ON cu.id = di.create_user
LEFT JOIN sys_user AS uu ON uu.id = di.update_user
WHERE di.id = ${BigInt(id)};
`;
      if (!rows.length) {
        return fail('404', '字典项不存在');
      }
      const row = rows[0];
      const resp: DictItemResp = {
        id: Number(row.id),
        label: row.label,
        value: row.value,
        color: row.color ?? '',
        sort: Number(row.sort ?? 999),
        description: row.description ?? '',
        status: Number(row.status),
        dictId: Number(row.dict_id),
        createUserString: row.create_user_string,
        createTime: row.create_time.toISOString(),
        updateUserString: row.update_user_string,
        updateTime: row.update_time ? row.update_time.toISOString() : '',
      };
      return ok(resp);
    } catch {
      return fail('500', '查询字典项失败');
    }
  }

  /** 新增字典项：POST /system/dict/item */
  @Post('/system/dict/item')
  async createDictItem(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: DictItemReq,
    @Req() req: any,
  ) {
    const begin = Date.now();
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const label = (body.label ?? '').trim();
    const value = (body.value ?? '').trim();
    const dictId = Number(body.dictId ?? 0);
    if (!label || !value || !Number.isFinite(dictId) || dictId <= 0) {
      return fail('400', '标签、值和字典 ID 不能为空');
    }
    const sort = body.sort && body.sort > 0 ? body.sort : 999;
    const status = body.status && body.status > 0 ? body.status : 1;

    try {
      // 校验同一字典下 value 是否重复
      const existsRows = await this.prisma.$queryRaw<{ exists: number }[]>`
SELECT 1 AS exists
FROM sys_dict_item
WHERE dict_id = ${BigInt(dictId)} AND value = ${value}
LIMIT 1;
`;
      if (existsRows.length) {
        return fail('400', `新增失败，字典值 [${value}] 已存在`);
      }

      await this.prisma.$executeRawUnsafe(
        `
INSERT INTO sys_dict_item (
    id, label, value, color, sort, description, status, dict_id,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10);
`,
        nextId(),
        label,
        value,
        body.color ?? '',
        sort,
        body.description ?? '',
        status,
        BigInt(dictId),
        BigInt(currentUserId),
        new Date(),
      );
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: `新增字典项[${label}]`,
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
      return ok(true);
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: `新增字典项[${label}]`,
        success: false,
        message: '新增字典项失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '新增字典项失败');
    }
  }

  /** 修改字典项：PUT /system/dict/item/:id */
  @Put('/system/dict/item/:id')
  async updateDictItem(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: DictItemReq,
    @Req() req: any,
  ) {
    const begin = Date.now();
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }
    const label = (body.label ?? '').trim();
    const value = (body.value ?? '').trim();
    if (!label || !value) {
      return fail('400', '标签和值不能为空');
    }
    const sort = body.sort && body.sort > 0 ? body.sort : 999;
    const status = body.status && body.status > 0 ? body.status : 1;

    try {
      // 查询当前字典项所属字典
      const dictRows = await this.prisma.$queryRaw<
        { dict_id: bigint }[]
      >`
SELECT dict_id
FROM sys_dict_item
WHERE id = ${BigInt(id)}
LIMIT 1;
`;
      if (!dictRows.length) {
        return fail('404', '字典项不存在');
      }
      const dictId = dictRows[0].dict_id;

      // 校验同一字典下 value 是否重复（排除自身）
      const existsRows = await this.prisma.$queryRaw<{ exists: number }[]>`
SELECT 1 AS exists
FROM sys_dict_item
WHERE dict_id = ${dictId} AND value = ${value} AND id <> ${BigInt(id)}
LIMIT 1;
`;
      if (existsRows.length) {
        return fail('400', `修改失败，字典值 [${value}] 已存在`);
      }

      await this.prisma.$executeRawUnsafe(
        `
UPDATE sys_dict_item
   SET label       = $1,
       value       = $2,
       color       = $3,
       sort        = $4,
       description = $5,
       status      = $6,
       update_user = $7,
       update_time = $8
 WHERE id          = $9;
`,
        label,
        value,
        body.color ?? '',
        sort,
        body.description ?? '',
        status,
        BigInt(currentUserId),
        new Date(),
        BigInt(id),
      );
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: `修改字典项[${label}]`,
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
      return ok(true);
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: `修改字典项[${label}]`,
        success: false,
        message: '修改字典项失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '修改字典项失败');
    }
  }

  /** 删除字典项：DELETE /system/dict/item */
  @Delete('/system/dict/item')
  async deleteDictItem(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: IdsRequest,
    @Req() req: any,
  ) {
    const begin = Date.now();
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const rawIds = Array.isArray(body?.ids) ? body.ids : [];
    if (!rawIds.length) {
      return fail('400', 'ID 列表不能为空');
    }
    const ids: bigint[] = [];
    for (const raw of rawIds) {
      const num = Number(raw);
      if (!Number.isFinite(num) || num <= 0) {
        return fail('400', 'ID 列表不能为空');
      }
      ids.push(BigInt(num));
    }

    const statements = ids.map((dictItemId) =>
      this.prisma.$executeRawUnsafe(
        `DELETE FROM sys_dict_item WHERE id = $1;`,
        dictItemId,
      ),
    );

    try {
      await this.prisma.$transaction(statements);
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: '删除字典项',
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
      return ok(true);
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: '删除字典项',
        success: false,
        message: '删除字典项失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '删除字典项失败');
    }
  }

  /** 查询单个字典：GET /system/dict/:id */
  @Get('/system/dict/:id')
  async getDict(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }
    try {
      const rows = await this.prisma.$queryRaw<
        {
          id: bigint;
          name: string;
          code: string;
          description: string;
          is_system: boolean;
          create_time: Date;
          create_user_string: string;
          update_time: Date | null;
          update_user_string: string;
        }[]
      >`
SELECT d.id,
       d.name,
       d.code,
       COALESCE(d.description, '') AS description,
       COALESCE(d.is_system, FALSE) AS is_system,
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dict AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
WHERE d.id = ${BigInt(id)};
`;
      if (!rows.length) {
        return fail('404', '字典不存在');
      }
      const row = rows[0];
      const resp: DictResp = {
        id: Number(row.id),
        name: row.name,
        code: row.code,
        description: row.description ?? '',
        isSystem: !!row.is_system,
        createUserString: row.create_user_string,
        createTime: row.create_time.toISOString(),
        updateUserString: row.update_user_string,
        updateTime: row.update_time ? row.update_time.toISOString() : '',
      };
      return ok(resp);
    } catch {
      return fail('500', '查询字典失败');
    }
  }

  /** 新增字典：POST /system/dict */
  @Post('/system/dict')
  async createDict(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: DictReq,
    @Req() req: any,
  ) {
    const begin = Date.now();
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const name = (body.name ?? '').trim();
    const code = (body.code ?? '').trim();
    if (!name || !code) {
      return fail('400', '名称和编码不能为空');
    }

    try {
      const nameRows = await this.prisma.$queryRaw<{ exists: number }[]>`
SELECT 1 AS exists FROM sys_dict WHERE name = ${name} LIMIT 1;
`;
      if (nameRows.length) {
        return fail('400', `新增失败，[${name}] 已存在`);
      }
      const codeRows = await this.prisma.$queryRaw<{ exists: number }[]>`
SELECT 1 AS exists FROM sys_dict WHERE code = ${code} LIMIT 1;
`;
      if (codeRows.length) {
        return fail('400', `新增失败，[${code}] 已存在`);
      }
    } catch {
      return fail('500', '新增字典失败');
    }

    const now = new Date();
    const newId = nextId();
    try {
      await this.prisma.$executeRawUnsafe(
        `
INSERT INTO sys_dict (id, name, code, description, is_system, create_user, create_time)
VALUES ($1, $2, $3, $4, FALSE, $5, $6);
`,
        newId,
        name,
        code,
        body.description ?? '',
        BigInt(currentUserId),
        now,
      );
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: `新增字典[${name}]`,
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
      return ok({ id: Number(newId) });
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: `新增字典[${name}]`,
        success: false,
        message: '新增字典失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '新增字典失败');
    }
  }

  /** 修改字典：PUT /system/dict/:id */
  @Put('/system/dict/:id')
  async updateDict(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: DictReq,
    @Req() req: any,
  ) {
    const begin = Date.now();
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }
    const name = (body.name ?? '').trim();
    if (!name) {
      return fail('400', '名称不能为空');
    }
    try {
      await this.prisma.$executeRawUnsafe(
        `
UPDATE sys_dict
   SET name = $1,
       description = $2,
       update_user = $3,
       update_time = $4
 WHERE id = $5;
`,
        name,
        body.description ?? '',
        BigInt(currentUserId),
        new Date(),
        BigInt(id),
      );
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: `修改字典[${name}]`,
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
      return ok(true);
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: `修改字典[${name}]`,
        success: false,
        message: '修改字典失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '修改字典失败');
    }
  }

  /** 删除字典：DELETE /system/dict */
  @Delete('/system/dict')
  async deleteDict(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: IdsRequest,
    @Req() req: any,
  ) {
    const begin = Date.now();
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const rawIds = Array.isArray(body?.ids) ? body.ids : [];
    if (!rawIds.length) {
      return fail('400', 'ID 列表不能为空');
    }
    const ids: bigint[] = [];
    for (const raw of rawIds) {
      const num = Number(raw);
      if (!Number.isFinite(num) || num <= 0) {
        return fail('400', 'ID 列表不能为空');
      }
      ids.push(BigInt(num));
    }

    // 校验是否包含系统内置字典
    try {
      const rows = await this.prisma.$queryRaw<
        { name: string; is_system: boolean }[]
      >`
SELECT name, COALESCE(is_system, FALSE) AS is_system
FROM sys_dict
WHERE id = ANY(${ids as any}::bigint[]);
`;
      const systemDict = rows.find((r) => r.is_system);
      if (systemDict) {
        return fail(
          '400',
          `所选字典 [${systemDict.name}] 是系统内置字典，不允许删除`,
        );
      }
    } catch {
      return fail('500', '删除字典失败');
    }

    const statements = ids.flatMap((dictId) => [
      this.prisma.$executeRawUnsafe(
        `DELETE FROM sys_dict_item WHERE dict_id = $1;`,
        dictId,
      ),
      this.prisma.$executeRawUnsafe(
        `DELETE FROM sys_dict WHERE id = $1;`,
        dictId,
      ),
    ]);

    try {
      await this.prisma.$transaction(statements);
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: '删除字典',
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
      return ok(true);
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '字典管理',
        description: '删除字典',
        success: false,
        message: '删除字典失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '删除字典失败');
    }
  }

  /** 清理字典缓存：DELETE /system/dict/cache/:code（当前无缓存逻辑，直接返回成功） */
  @Delete('/system/dict/cache/:code')
  async clearDictCache() {
    return ok(true);
  }
}
