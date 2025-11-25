import {
  Body,
  Controller,
  Get,
  Headers,
  Patch,
  Put,
  Query,
} from '@nestjs/common';
import { Prisma } from '@prisma/client';
import { PrismaService } from '../../../shared/prisma/prisma.service';
import { ok, fail } from '../../../shared/api-response/api-response';
import { TokenService } from '../../auth/jwt/jwt.service';
import { OptionResetReq, OptionResp, OptionUpdateItem } from './dto';

/**
 * 系统配置控制器，复刻 Java/Go `/system/option*` 行为。
 */
@Controller()
export class SystemOptionController {
  constructor(
    private readonly prisma: PrismaService,
    private readonly tokenService: TokenService,
  ) {}

  /** 解析当前登录用户 ID。 */
  private currentUserId(authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) {
      return 0;
    }
    return claims.userId;
  }

  /**
   * 查询系统配置列表，支持 code 多值与 category 筛选。
   */
  @Get('/system/option')
  async listOption(
    @Query('code') code?: string | string[],
    @Query('category') category?: string,
  ) {
    const codes = this.normalizeMultiValue(code);
    const categoryFilter = (category || '').trim();

    let where = 'WHERE 1=1';
    const args: any[] = [];
    let pos = 1;
    if (codes.length) {
      const placeholders = codes.map((_, idx) => `$${pos + idx}`);
      where += ` AND code IN (${placeholders.join(',')})`;
      args.push(...codes);
      pos += codes.length;
    }
    if (categoryFilter) {
      where += ` AND category = $${pos}`;
      args.push(categoryFilter);
      pos += 1;
    }

    const sql = `
SELECT id,
       name,
       code,
       COALESCE(value, default_value, '') AS value,
       COALESCE(description, '')          AS description
FROM sys_option
${where}
ORDER BY id ASC;
`;
    try {
      const rows = await this.prisma.$queryRawUnsafe<
        { id: bigint; name: string; code: string; value: string; description: string }[]
      >(sql, ...args);
      const list: OptionResp[] = rows.map((r) => ({
        id: Number(r.id),
        name: r.name,
        code: r.code,
        value: r.value ?? '',
        description: r.description ?? '',
      }));
      return ok(list);
    } catch {
      return fail('500', '查询系统配置失败');
    }
  }

  /**
   * 批量保存系统配置值：PUT /system/option。
   */
  @Put('/system/option')
  async updateOption(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: OptionUpdateItem[],
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    if (!Array.isArray(body) || body.length === 0) {
      return fail('400', '请求参数不正确');
    }

    const now = new Date();
    let statements: Prisma.PrismaPromise<unknown>[];
    try {
      statements = body.map((item) => {
        const idNum = Number(item.id);
        const code = (item.code || '').trim();
        if (!Number.isFinite(idNum) || idNum <= 0 || !code) {
          throw new Error('invalid option');
        }
        return this.prisma.$executeRawUnsafe(
          `
UPDATE sys_option
   SET value = $1,
       update_user = $2,
       update_time = $3
 WHERE id = $4 AND code = $5;
`,
          toOptionValueString(item.value),
          BigInt(currentUserId),
          now,
          BigInt(idNum),
          code,
        );
      });
    } catch {
      return fail('400', '请求参数不正确');
    }

    try {
      await this.prisma.$transaction(statements);
      return ok(true);
    } catch {
      return fail('500', '保存系统配置失败');
    }
  }

  /**
   * 恢复默认值：PATCH /system/option/value。
   */
  @Patch('/system/option/value')
  async resetOptionValue(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: OptionResetReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    if (!body) {
      return fail('400', '请求参数不正确');
    }
    const category = (body.category || '').trim();
    const codes = Array.isArray(body.code)
      ? body.code
          .map((c) => c?.trim())
          .filter((c): c is string => !!c)
      : [];
    if (!category && !codes.length) {
      return fail('400', '键列表或类别不能为空');
    }

    let stmt = 'UPDATE sys_option SET value = NULL';
    const args: any[] = [];
    if (category) {
      stmt += ' WHERE category = $1';
      args.push(category);
    } else if (codes.length) {
      const placeholders = codes.map((_, idx) => `$${idx + 1}`);
      stmt += ` WHERE code IN (${placeholders.join(',')})`;
      args.push(...codes);
    }

    try {
      await this.prisma.$executeRawUnsafe(stmt, ...args);
      return ok(true);
    } catch {
      return fail('500', '恢复默认配置失败');
    }
  }

  /** 解析 code 查询参数，兼容逗号分隔与重复参数形式。 */
  private normalizeMultiValue(value?: string | string[]) {
    if (!value) {
      return [] as string[];
    }
    const list = Array.isArray(value) ? value : [value];
    const result: string[] = [];
    for (const raw of list) {
      const parts = raw.split(',');
      for (const part of parts) {
        const trimmed = part.trim();
        if (trimmed) {
          result.push(trimmed);
        }
      }
    }
    return result;
  }
}

/**
 * 将任意值转成字符串，保持与 Java/Go 写入 sys_option 的逻辑一致。
 */
export function toOptionValueString(value: unknown): string {
  if (value === null || value === undefined) {
    return '';
  }
  if (typeof value === 'string') {
    return value;
  }
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) {
      return '';
    }
    return Math.trunc(value).toString();
  }
  if (typeof value === 'boolean') {
    return value ? 'true' : 'false';
  }
  try {
    return JSON.stringify(value);
  } catch {
    return '';
  }
}
