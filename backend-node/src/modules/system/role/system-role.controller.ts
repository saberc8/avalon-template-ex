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
} from '@nestjs/common';
import { PrismaService } from '../../../shared/prisma/prisma.service';
import { ok, fail } from '../../../shared/api-response/api-response';
import { TokenService } from '../../auth/jwt/jwt.service';
import { nextId } from '../../../shared/id/id';
import {
  PageQuery,
  PageResult,
  RoleDetailResp,
  RolePermissionReq,
  RoleReq,
  RoleResp,
  RoleUserResp,
} from './dto';

interface IdsRequest {
  ids: number[];
}

/**
 * 角色管理接口集合，对齐 Java/Go 的 /system/role* 设计。
 */
@Controller()
export class SystemRoleController {
  constructor(
    private readonly prisma: PrismaService,
    private readonly tokenService: TokenService,
  ) {}

  private currentUserId(authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) return 0;
    return claims.userId;
  }

  /** GET /system/role/list */
  @Get('/system/role/list')
  async listRole(@Query('description') description?: string) {
    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        name: string;
        code: string;
        sort: number;
        description: string | null;
        data_scope: number | null;
        is_system: boolean | null;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >`
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999),
       COALESCE(r.description, ''),
       COALESCE(r.data_scope, 4),
       COALESCE(r.is_system, FALSE),
       r.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       r.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
ORDER BY r.sort ASC, r.id ASC;
`;
    const desc = (description || '').trim();
    const list: RoleResp[] = [];
    for (const r of rows) {
      const item: RoleResp = {
        id: Number(r.id),
        name: r.name,
        code: r.code,
        sort: r.sort,
        description: r.description ?? '',
        dataScope: r.data_scope ?? 4,
        isSystem: !!r.is_system,
        createUserString: r.create_user_string,
        createTime: r.create_time.toISOString(),
        updateUserString: r.update_user_string,
        updateTime: r.update_time ? r.update_time.toISOString() : '',
        disabled: !!r.is_system && r.code === 'admin',
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
  }

  /** GET /system/role/:id */
  @Get('/system/role/:id')
  async getRole(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        name: string;
        code: string;
        sort: number;
        description: string | null;
        data_scope: number | null;
        is_system: boolean | null;
        menu_check_strictly: boolean | null;
        dept_check_strictly: boolean | null;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >`
SELECT r.id,
       r.name,
       r.code,
       COALESCE(r.sort, 999),
       COALESCE(r.description, ''),
       COALESCE(r.data_scope, 4),
       COALESCE(r.is_system, FALSE),
       COALESCE(r.menu_check_strictly, TRUE),
       COALESCE(r.dept_check_strictly, TRUE),
       r.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       r.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_role AS r
LEFT JOIN sys_user AS cu ON cu.id = r.create_user
LEFT JOIN sys_user AS uu ON uu.id = r.update_user
WHERE r.id = ${BigInt(id)};
`;
    if (!rows.length) {
      return fail('404', '角色不存在');
    }
    const baseRow = rows[0];
    const base: RoleResp = {
      id: Number(baseRow.id),
      name: baseRow.name,
      code: baseRow.code,
      sort: baseRow.sort,
      description: baseRow.description ?? '',
      dataScope: baseRow.data_scope ?? 4,
      isSystem: !!baseRow.is_system,
      createUserString: baseRow.create_user_string,
      createTime: baseRow.create_time.toISOString(),
      updateUserString: baseRow.update_user_string,
      updateTime: baseRow.update_time ? baseRow.update_time.toISOString() : '',
      disabled: !!baseRow.is_system && baseRow.code === 'admin',
    };

    const menuRows = await this.prisma.$queryRaw<
      { menu_id: bigint }[]
    >`SELECT menu_id FROM sys_role_menu WHERE role_id = ${BigInt(id)};`;
    const deptRows = await this.prisma.$queryRaw<
      { dept_id: bigint }[]
    >`SELECT dept_id FROM sys_role_dept WHERE role_id = ${BigInt(id)};`;

    const resp: RoleDetailResp = {
      ...base,
      menuIds: menuRows.map((r) => Number(r.menu_id)),
      deptIds: deptRows.map((r) => Number(r.dept_id)),
      menuCheckStrictly: baseRow.menu_check_strictly ?? true,
      deptCheckStrictly: baseRow.dept_check_strictly ?? true,
    };
    return ok(resp);
  }

  /** POST /system/role */
  @Post('/system/role')
  async createRole(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: RoleReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }

    const name = (body.name ?? '').trim();
    const code = (body.code ?? '').trim();
    if (!name || !code) {
      return fail('400', '名称和编码不能为空');
    }
    const sort = body.sort > 0 ? body.sort : 999;
    const dataScope = body.dataScope || 4;
    const deptCheckStrict = body.deptCheckStrictly ?? true;
    const now = new Date();
    const newId = nextId();

    const tx = this.prisma.$transaction;
    try {
      await tx([
        this.prisma.$executeRawUnsafe(
          `
INSERT INTO sys_role (
    id, name, code, data_scope, description, sort,
    is_system, menu_check_strictly, dept_check_strictly,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6,
        FALSE, TRUE, $7,
        $8, $9);
`,
          newId,
          name,
          code,
          dataScope,
          body.description ?? '',
          sort,
          deptCheckStrict,
          BigInt(currentUserId),
          now,
        ),
        ...(body.deptIds || []).map((did) =>
          this.prisma.$executeRawUnsafe(
            `INSERT INTO sys_role_dept (role_id, dept_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;`,
            newId,
            BigInt(did),
          ),
        ),
      ]);
    } catch {
      return fail('500', '新增角色失败');
    }

    return ok({ id: Number(newId) });
  }

  /** PUT /system/role/:id */
  @Put('/system/role/:id')
  async updateRole(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: RoleReq,
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
    if (!name) {
      return fail('400', '名称不能为空');
    }
    const sort = body.sort > 0 ? body.sort : 999;
    const dataScope = body.dataScope || 4;
    const deptCheckStrict = body.deptCheckStrictly ?? true;
    const now = new Date();

    const tx = this.prisma.$transaction;
    try {
      await tx([
        this.prisma.$executeRawUnsafe(
          `
UPDATE sys_role
   SET name               = $1,
       description        = $2,
       sort               = $3,
       data_scope         = $4,
       dept_check_strictly= $5,
       update_user        = $6,
       update_time        = $7
 WHERE id                 = $8;
`,
          name,
          body.description ?? '',
          sort,
          dataScope,
          deptCheckStrict,
          BigInt(currentUserId),
          now,
          BigInt(id),
        ),
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_role_dept WHERE role_id = $1`,
          BigInt(id),
        ),
        ...(body.deptIds || []).map((did) =>
          this.prisma.$executeRawUnsafe(
            `INSERT INTO sys_role_dept (role_id, dept_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;`,
            BigInt(id),
            BigInt(did),
          ),
        ),
      ]);
    } catch {
      return fail('500', '修改角色失败');
    }

    return ok(true);
  }

  /** DELETE /system/role */
  @Delete('/system/role')
  async deleteRole(@Body() body: IdsRequest) {
    if (!body?.ids?.length) {
      return fail('400', 'ID 列表不能为空');
    }
    const ids = body.ids.filter((v) => Number.isFinite(v) && v > 0);
    if (!ids.length) {
      return fail('400', 'ID 列表不能为空');
    }
    const idsBig = ids.map((v) => BigInt(v));

    const tx = this.prisma.$transaction;
    try {
      await tx([
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_user_role WHERE role_id = ANY($1::bigint[])`,
          idsBig,
        ),
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_role_menu WHERE role_id = ANY($1::bigint[])`,
          idsBig,
        ),
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_role_dept WHERE role_id = ANY($1::bigint[])`,
          idsBig,
        ),
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_role WHERE id = ANY($1::bigint[])`,
          idsBig,
        ),
      ]);
    } catch {
      return fail('500', '删除角色失败');
    }
    return ok(true);
  }

  /** PUT /system/role/:id/permission */
  @Put('/system/role/:id/permission')
  async updatePermission(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: RolePermissionReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const tx = this.prisma.$transaction;
    try {
      await tx([
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_role_menu WHERE role_id = $1`,
          BigInt(id),
        ),
        ...(body.menuIds || []).map((mid) =>
          this.prisma.$executeRawUnsafe(
            `INSERT INTO sys_role_menu (role_id, menu_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;`,
            BigInt(id),
            BigInt(mid),
          ),
        ),
        this.prisma.$executeRawUnsafe(
          `
UPDATE sys_role
   SET menu_check_strictly = $1,
       update_user         = $2,
       update_time         = $3
 WHERE id                  = $4;
`,
          body.menuCheckStrictly ?? true,
          BigInt(currentUserId),
          new Date(),
          BigInt(id),
        ),
      ]);
    } catch {
      return fail('500', '保存角色权限失败');
    }

    return ok(true);
  }

  /** GET /system/role/:id/user 分页查询关联用户 */
  @Get('/system/role/:id/user')
  async pageRoleUser(
    @Param('id') idParam: string,
    @Query() query: PageQuery,
  ) {
    const roleId = Number(idParam);
    if (!Number.isFinite(roleId) || roleId <= 0) {
      return fail('400', 'ID 参数不正确');
    }
    let page = Number(query.page ?? 1);
    let size = Number(query.size ?? 10);
    if (!Number.isFinite(page) || page <= 0) page = 1;
    if (!Number.isFinite(size) || size <= 0) size = 10;
    const desc = (query.description || '').trim();

    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        role_id: bigint;
        user_id: bigint;
        username: string;
        nickname: string;
        gender: number;
        status: number;
        is_system: boolean;
        description: string | null;
        dept_id: bigint | null;
        dept_name: string;
      }[]
    >`
SELECT ur.id,
       ur.role_id,
       u.id AS user_id,
       u.username,
       u.nickname,
       u.gender,
       u.status,
       u.is_system,
       COALESCE(u.description, '') AS description,
       u.dept_id,
       COALESCE(d.name, '') AS dept_name
FROM sys_user_role AS ur
JOIN sys_user AS u ON u.id = ur.user_id
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
WHERE ur.role_id = ${BigInt(roleId)}
ORDER BY ur.id DESC;
`;

    const all: RoleUserResp[] = [];
    for (const r of rows) {
      const item: RoleUserResp = {
        id: Number(r.id),
        roleId: Number(r.role_id),
        userId: Number(r.user_id),
        username: r.username,
        nickname: r.nickname,
        gender: r.gender,
        status: r.status,
        isSystem: r.is_system,
        description: r.description ?? '',
        deptId: r.dept_id ? Number(r.dept_id) : 0,
        deptName: r.dept_name,
        roleIds: [],
        roleNames: [],
        disabled: false,
      };
      if (
        desc &&
        !item.username.includes(desc) &&
        !item.nickname.includes(desc) &&
        !item.description.includes(desc)
      ) {
        continue;
      }
      all.push(item);
    }

    // 填充每个用户的角色列表
    if (all.length) {
      const userIds = Array.from(
        new Set(all.map((u) => u.userId).filter((id) => id > 0)),
      );
      const roleRows = await this.prisma.$queryRaw<
        { user_id: bigint; role_id: bigint; name: string }[]
      >`
SELECT ur.user_id, ur.role_id, r.name
FROM sys_user_role AS ur
JOIN sys_role AS r ON r.id = ur.role_id
WHERE ur.user_id = ANY(${userIds as any}::bigint[]);
`;
      const map = new Map<
        number,
        { ids: number[]; names: string[] }
      >();
      for (const r of roleRows) {
        const uid = Number(r.user_id);
        const rid = Number(r.role_id);
        const entry = map.get(uid) ?? { ids: [], names: [] };
        entry.ids.push(rid);
        entry.names.push(r.name);
        map.set(uid, entry);
      }
      for (const u of all) {
        const entry = map.get(u.userId);
        if (entry) {
          u.roleIds = entry.ids;
          u.roleNames = entry.names;
        }
        u.disabled = u.isSystem && u.roleId === 1;
      }
    }

    const total = all.length;
    let start = (page - 1) * size;
    if (start > total) start = total;
    let end = start + size;
    if (end > total) end = total;
    const pageList = all.slice(start, end);

    const resp: PageResult<RoleUserResp> = {
      list: pageList,
      total,
    };
    return ok(resp);
  }

  /** POST /system/role/:id/user 分配用户 */
  @Post('/system/role/:id/user')
  async assignToUsers(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() userIds: number[],
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const roleId = Number(idParam);
    if (!Number.isFinite(roleId) || roleId <= 0) {
      return fail('400', 'ID 参数不正确');
    }
    if (!Array.isArray(userIds) || !userIds.length) {
      return fail('400', '用户ID列表不能为空');
    }

    const tx = this.prisma.$transaction;
    try {
      await tx(
        userIds
          .filter((uid) => Number.isFinite(uid) && uid > 0)
          .map((uid) =>
            this.prisma.$executeRawUnsafe(
              `
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
`,
              nextId(),
              BigInt(uid),
              BigInt(roleId),
            ),
          ),
      );
    } catch {
      return fail('500', '分配用户失败');
    }

    return ok(true);
  }

  /** DELETE /system/role/user 取消分配用户（body: userRoleId 数组） */
  @Delete('/system/role/user')
  async unassignFromUsers(
    @Headers('authorization') authorization: string | undefined,
    @Body() ids: number[],
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    if (!Array.isArray(ids) || !ids.length) {
      return fail('400', '用户角色ID列表不能为空');
    }
    const idList = ids.filter((v) => Number.isFinite(v) && v > 0);
    if (!idList.length) {
      return fail('400', '用户角色ID列表不能为空');
    }

    try {
      await this.prisma.$executeRawUnsafe(
        `DELETE FROM sys_user_role WHERE id = ANY($1::bigint[])`,
        idList.map((v) => BigInt(v)),
      );
    } catch {
      return fail('500', '取消分配失败');
    }
    return ok(true);
  }

  /** GET /system/role/:id/user/id 查询关联用户 ID 列表 */
  @Get('/system/role/:id/user/id')
  async listUserId(@Param('id') idParam: string) {
    const roleId = Number(idParam);
    if (!Number.isFinite(roleId) || roleId <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const rows = await this.prisma.$queryRaw<
      { user_id: bigint }[]
    >`SELECT user_id FROM sys_user_role WHERE role_id = ${BigInt(roleId)};`;
    const ids = rows.map((r) => Number(r.user_id));
    return ok(ids);
  }
}

