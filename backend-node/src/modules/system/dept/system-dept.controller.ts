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
import { DeptReq, DeptResp, DeptTreeQuery, DeleteDeptReq } from './dto';
import { writeOperationLog } from '../../../shared/log/operation-log';

/**
 * 部门管理接口集合，对齐 /system/dept*。
 */
@Controller()
export class SystemDeptController {
  constructor(
    private readonly prisma: PrismaService,
    private readonly tokenService: TokenService,
  ) {}

  private currentUserId(authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) return 0;
    return claims.userId;
  }

  /** 部门树：GET /system/dept/tree */
  @Get('/system/dept/tree')
  async listDeptTree(@Query() query: DeptTreeQuery) {
    const desc = (query.description || '').trim();
    const statusStr = (query.status || '').trim();
    let status = 0;
    if (statusStr) {
      const v = Number(statusStr);
      if (Number.isFinite(v) && v > 0) status = v;
    }

    let where = 'WHERE 1=1';
    const args: any[] = [];
    let argPos = 1;
    if (desc) {
      where += ` AND (d.name ILIKE $${argPos} OR COALESCE(d.description,'') ILIKE $${argPos})`;
      args.push(`%${desc}%`);
      argPos++;
    }
    if (status) {
      where += ` AND d.status = $${argPos}`;
      args.push(status);
      argPos++;
    }

    const sql = `
SELECT d.id,
       d.name,
       d.parent_id,
       d.sort,
       d.status,
       d.is_system,
       COALESCE(d.description, ''),
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
${where}
ORDER BY d.sort ASC, d.id ASC;
`;

    const rows = await this.prisma.$queryRawUnsafe<
      {
        id: bigint;
        name: string;
        parent_id: bigint;
        sort: number;
        status: number;
        is_system: boolean;
        description: string;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >(sql, ...args);

    if (!rows.length) {
      return ok<DeptResp[]>([]);
    }

    const nodeMap = new Map<number, DeptResp>();
    for (const r of rows) {
      const node: DeptResp = {
        id: Number(r.id),
        name: r.name,
        sort: r.sort,
        status: r.status,
        isSystem: r.is_system,
        description: r.description,
        createUserString: r.create_user_string,
        createTime: r.create_time.toISOString(),
        updateUserString: r.update_user_string,
        updateTime: r.update_time ? r.update_time.toISOString() : '',
        parentId: Number(r.parent_id),
        children: [],
      };
      nodeMap.set(node.id, node);
    }

    const roots: DeptResp[] = [];
    for (const node of nodeMap.values()) {
      if (node.parentId === 0) {
        roots.push(node);
        continue;
      }
      const parent = nodeMap.get(node.parentId);
      if (!parent) {
        roots.push(node);
        continue;
      }
      parent.children.push(node);
    }

    return ok(roots);
  }

  /** GET /system/dept/:id */
  @Get('/system/dept/:id')
  async getDept(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', '无效的部门 ID');
    }

    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        name: string;
        parent_id: bigint;
        sort: number;
        status: number;
        is_system: boolean;
        description: string;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >`
SELECT d.id,
       d.name,
       d.parent_id,
       d.sort,
       d.status,
       d.is_system,
       COALESCE(d.description, ''),
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
WHERE d.id = ${BigInt(id)};
`;
    if (!rows.length) {
      return fail('404', '部门不存在');
    }
    const r = rows[0];
    const resp: DeptResp = {
      id: Number(r.id),
      name: r.name,
      sort: r.sort,
      status: r.status,
      isSystem: r.is_system,
      description: r.description,
      createUserString: r.create_user_string,
      createTime: r.create_time.toISOString(),
      updateUserString: r.update_user_string,
      updateTime: r.update_time ? r.update_time.toISOString() : '',
      parentId: Number(r.parent_id),
      children: [],
    };
    return ok(resp);
  }

  /** POST /system/dept */
  @Post('/system/dept')
  async createDept(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: DeptReq,
    @Req() req: any,
  ) {
    const begin = Date.now();
    const name = (body.name ?? '').trim();
    if (!name) {
      return fail('400', '名称不能为空');
    }
    if (!body.parentId) {
      return fail('400', '上级部门不能为空');
    }
    const sort = body.sort > 0 ? body.sort : 1;
    const status = body.status || 1;

    // 名称唯一性
    const dupRows = await this.prisma.$queryRaw<
      { exists: boolean }[]
    >`
SELECT EXISTS(
  SELECT 1 FROM sys_dept WHERE name = ${name} AND parent_id = ${BigInt(
      body.parentId,
    )}
) AS exists;
`;
    if (dupRows[0]?.exists) {
      return fail('400', '新增失败，该名称在当前上级下已存在');
    }

    // 上级是否存在
    const parentRows = await this.prisma.$queryRaw<
      { exists: boolean }[]
    >`
SELECT EXISTS(SELECT 1 FROM sys_dept WHERE id = ${BigInt(
      body.parentId,
    )}) AS exists;
`;
    if (!parentRows[0]?.exists) {
      return fail('400', '上级部门不存在');
    }

    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }

    const now = new Date();
    const newId = BigInt(Date.now());
    try {
      await this.prisma.$executeRawUnsafe(
        `
INSERT INTO sys_dept (
    id, name, parent_id, sort, status, is_system, description,
    create_user, create_time
) VALUES (
    $1, $2, $3, $4, $5, FALSE, $6,
    $7, $8
);
`,
        newId,
        name,
        BigInt(body.parentId),
        sort,
        status,
        (body.description ?? '').trim(),
        BigInt(currentUserId),
        now,
      );
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '部门管理',
        description: `新增部门[${name}]`,
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '部门管理',
        description: `新增部门[${name}]`,
        success: false,
        message: '新增部门失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '新增部门失败');
    }

    return ok(true);
  }

  /** PUT /system/dept/:id */
  @Put('/system/dept/:id')
  async updateDept(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: DeptReq,
    @Req() req: any,
  ) {
    const begin = Date.now();
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', '无效的部门 ID');
    }

    const name = (body.name ?? '').trim();
    if (!name) {
      return fail('400', '名称不能为空');
    }
    if (!body.parentId) {
      return fail('400', '上级部门不能为空');
    }
    const sort = body.sort > 0 ? body.sort : 1;
    const status = body.status || 1;

    // 查询旧值
    const oldRows = await this.prisma.$queryRaw<
      {
        id: bigint;
        name: string;
        parent_id: bigint;
        status: number;
        is_system: boolean;
      }[]
    >`
SELECT id, name, parent_id, status, is_system
FROM sys_dept
WHERE id = ${BigInt(id)};
`;
    if (!oldRows.length) {
      return fail('404', '部门不存在');
    }
    const old = oldRows[0];

    if (old.is_system) {
      if (status === 2) {
        return fail(
          '400',
          `[${old.name}] 是系统内置部门，不允许禁用`,
        );
      }
      if (BigInt(body.parentId) !== old.parent_id) {
        return fail(
          '400',
          `[${old.name}] 是系统内置部门，不允许变更上级部门`,
        );
      }
    }

    // 名称唯一性（排除自身）
    const dupRows = await this.prisma.$queryRaw<
      { exists: boolean }[]
    >`
SELECT EXISTS(
  SELECT 1 FROM sys_dept WHERE name = ${name} AND parent_id = ${BigInt(
      body.parentId,
    )} AND id <> ${BigInt(id)}
) AS exists;
`;
    if (dupRows[0]?.exists) {
      return fail('400', '修改失败，该名称在当前上级下已存在');
    }

    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }

    try {
      await this.prisma.$executeRawUnsafe(
        `
UPDATE sys_dept
   SET name = $1,
       parent_id = $2,
       sort = $3,
       status = $4,
       description = $5,
       update_user = $6,
       update_time = $7
 WHERE id = $8;
`,
        name,
        BigInt(body.parentId),
        sort,
        status,
        (body.description ?? '').trim(),
        BigInt(currentUserId),
        new Date(),
        BigInt(id),
      );
      // 部门修改成功后记录操作日志，便于在系统日志中审计该操作。
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        description: `修改部门[${old.name}]`,
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
    } catch {
      // 记录失败操作日志，但不再抛出二次异常。
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        description: `修改部门[${old.name}]`,
        success: false,
        message: '修改部门失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '修改部门失败');
    }

    return ok(true);
  }

  /** DELETE /system/dept */
  @Delete('/system/dept')
  async deleteDept(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: DeleteDeptReq,
    @Req() req: any,
  ) {
    const begin = Date.now();
    if (!body?.ids?.length) {
      return fail('400', '参数错误');
    }
    const ids = body.ids.filter((v) => Number.isFinite(v) && v > 0);
    if (!ids.length) {
      return fail('400', '参数错误');
    }
    const idsBig = ids.map((v) => BigInt(v));
    const currentUserId = this.currentUserId(authorization);

    // 1. 系统内置校验
    const sysRows = await this.prisma.$queryRaw<
      { name: string }[]
    >`
SELECT name
FROM sys_dept
WHERE id = ANY(${idsBig as any}) AND is_system = TRUE
LIMIT 1;
`;
    if (sysRows.length) {
      return fail(
        '400',
        `所选部门 [${sysRows[0].name}] 是系统内置部门，不允许删除`,
      );
    }

    // 2. 子部门校验
    const childRows = await this.prisma.$queryRaw<
      { exists: boolean }[]
    >`
SELECT EXISTS(
  SELECT 1 FROM sys_dept WHERE parent_id = ANY(${idsBig as any})
) AS exists;
`;
    if (childRows[0]?.exists) {
      return fail('400', '所选部门存在下级部门，不允许删除');
    }

    // 3. 用户关联校验
    const userRows = await this.prisma.$queryRaw<
      { exists: boolean }[]
    >`
SELECT EXISTS(
  SELECT 1 FROM sys_user WHERE dept_id = ANY(${idsBig as any})
) AS exists;
`;
    if (userRows[0]?.exists) {
      return fail(
        '400',
        '所选部门存在用户关联，请解除关联后重试',
      );
    }

    // 4. 删除角色部门关系
    await this.prisma.$executeRawUnsafe(
      `DELETE FROM sys_role_dept WHERE dept_id = ANY($1::bigint[])`,
      idsBig,
    );

    // 5. 删除部门
    await this.prisma.$executeRawUnsafe(
      `DELETE FROM sys_dept WHERE id = ANY($1::bigint[])`,
      idsBig,
    );

    await writeOperationLog(this.prisma, {
      req,
      userId: currentUserId,
      module: '部门管理',
      description: '删除部门',
      success: true,
      message: '',
      timeTakenMs: Date.now() - begin,
    });

    return ok(true);
  }

  /** 导出部门 CSV：GET /system/dept/export */
  @Get('/system/dept/export')
  async exportDept(@Query() query: DeptTreeQuery) {
    const desc = (query.description || '').trim();
    const statusStr = (query.status || '').trim();
    let status = 0;
    if (statusStr) {
      const v = Number(statusStr);
      if (Number.isFinite(v) && v > 0) status = v;
    }

    let where = 'WHERE 1=1';
    const args: any[] = [];
    let argPos = 1;
    if (desc) {
      where += ` AND (d.name ILIKE $${argPos} OR COALESCE(d.description,'') ILIKE $${argPos})`;
      args.push(`%${desc}%`);
      argPos++;
    }
    if (status) {
      where += ` AND d.status = $${argPos}`;
      args.push(status);
      argPos++;
    }

    const sql = `
SELECT d.id,
       d.name,
       d.parent_id,
       d.status,
       d.sort,
       d.is_system,
       COALESCE(d.description, ''),
       d.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       d.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_dept AS d
LEFT JOIN sys_user AS cu ON cu.id = d.create_user
LEFT JOIN sys_user AS uu ON uu.id = d.update_user
${where}
ORDER BY d.sort ASC, d.id ASC;
`;

    const rows = await this.prisma.$queryRawUnsafe<
      {
        id: bigint;
        name: string;
        parent_id: bigint;
        status: number;
        sort: number;
        is_system: boolean;
        description: string;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >(sql, ...args);

    const header =
      'ID,名称,上级部门ID,状态,排序,系统内置,描述,创建时间,创建人,修改时间,修改人';
    const lines = [header];
    for (const r of rows) {
      const ut = r.update_time ? r.update_time.toISOString() : '';
      const line = [
        String(r.id),
        r.name,
        String(r.parent_id),
        String(r.status),
        String(r.sort),
        String(r.is_system),
        r.description,
        r.create_time.toISOString(),
        r.create_user_string,
        ut,
        r.update_user_string,
      ].join(',');
      lines.push(line);
    }
    return lines.join('\n');
  }
}
