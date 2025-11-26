import {
  Body,
  Controller,
  Delete,
  Get,
  Headers,
  Param,
  Post,
  Put,
  Req,
} from '@nestjs/common';
import { PrismaService } from '../../../shared/prisma/prisma.service';
import { ok, fail } from '../../../shared/api-response/api-response';
import { TokenService } from '../../auth/jwt/jwt.service';
import { MenuReq, MenuResp, IdsRequest } from './dto';
import { writeOperationLog } from '../../../shared/log/operation-log';

/**
 * 菜单管理接口集合，兼容前端 /system/menu* 请求。
 */
@Controller()
export class SystemMenuController {
  constructor(
    private readonly prisma: PrismaService,
    private readonly tokenService: TokenService,
  ) {}

  private currentUserId(authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) return 0;
    return claims.userId;
  }

  /** GET /system/menu/tree */
  @Get('/system/menu/tree')
  async listMenuTree() {
    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        title: string;
        parent_id: bigint;
        type: number;
        path: string | null;
        name: string | null;
        component: string | null;
        redirect: string | null;
        icon: string | null;
        is_external: boolean | null;
        is_cache: boolean | null;
        is_hidden: boolean | null;
        permission: string | null;
        sort: number | null;
        status: number | null;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >`
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, '')        AS path,
       COALESCE(m.name, '')        AS name,
       COALESCE(m.component, '')   AS component,
       COALESCE(m.redirect, '')    AS redirect,
       COALESCE(m.icon, '')        AS icon,
       COALESCE(m.is_external, FALSE) AS is_external,
       COALESCE(m.is_cache, FALSE)    AS is_cache,
       COALESCE(m.is_hidden, FALSE)   AS is_hidden,
       COALESCE(m.permission, '')  AS permission,
       COALESCE(m.sort, 0)         AS sort,
       COALESCE(m.status, 1)       AS status,
       m.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       m.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
ORDER BY m.sort ASC, m.id ASC;
`;

    const flat: MenuResp[] = rows.map((r) => ({
      id: Number(r.id),
      title: r.title,
      parentId: Number(r.parent_id),
      type: r.type,
      path: r.path ?? '',
      name: r.name ?? '',
      component: r.component ?? '',
      redirect: r.redirect ?? '',
      icon: r.icon ?? '',
      isExternal: !!r.is_external,
      isCache: !!r.is_cache,
      isHidden: !!r.is_hidden,
      permission: r.permission ?? '',
      sort: r.sort ?? 0,
      status: r.status ?? 1,
      createUserString: r.create_user_string,
      createTime: r.create_time.toISOString(),
      updateUserString: r.update_user_string,
      updateTime: r.update_time ? r.update_time.toISOString() : '',
      children: [],
    }));

    const nodeMap = new Map<number, MenuResp>();
    for (const item of flat) {
      nodeMap.set(item.id, item);
    }

    const roots: MenuResp[] = [];
    for (const item of flat) {
      if (item.parentId === 0) {
        roots.push(item);
        continue;
      }
      const parent = nodeMap.get(item.parentId);
      if (!parent) {
        roots.push(item);
        continue;
      }
      parent.children.push(item);
    }

    return ok(roots);
  }

  /** GET /system/menu/:id */
  @Get('/system/menu/:id')
  async getMenu(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        title: string;
        parent_id: bigint;
        type: number;
        path: string | null;
        name: string | null;
        component: string | null;
        redirect: string | null;
        icon: string | null;
        is_external: boolean | null;
        is_cache: boolean | null;
        is_hidden: boolean | null;
        permission: string | null;
        sort: number | null;
        status: number | null;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >`
SELECT m.id,
       m.title,
       m.parent_id,
       m.type,
       COALESCE(m.path, '')        AS path,
       COALESCE(m.name, '')        AS name,
       COALESCE(m.component, '')   AS component,
       COALESCE(m.redirect, '')    AS redirect,
       COALESCE(m.icon, '')        AS icon,
       COALESCE(m.is_external, FALSE) AS is_external,
       COALESCE(m.is_cache, FALSE)    AS is_cache,
       COALESCE(m.is_hidden, FALSE)   AS is_hidden,
       COALESCE(m.permission, '')  AS permission,
       COALESCE(m.sort, 0)         AS sort,
       COALESCE(m.status, 1)       AS status,
       m.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       m.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_menu AS m
LEFT JOIN sys_user AS cu ON cu.id = m.create_user
LEFT JOIN sys_user AS uu ON uu.id = m.update_user
WHERE m.id = ${BigInt(id)};
`;
    if (!rows.length) {
      return fail('404', '菜单不存在');
    }
    const r = rows[0];
    const item: MenuResp = {
      id: Number(r.id),
      title: r.title,
      parentId: Number(r.parent_id),
      type: r.type,
      path: r.path ?? '',
      name: r.name ?? '',
      component: r.component ?? '',
      redirect: r.redirect ?? '',
      icon: r.icon ?? '',
      isExternal: !!r.is_external,
      isCache: !!r.is_cache,
      isHidden: !!r.is_hidden,
      permission: r.permission ?? '',
      sort: r.sort ?? 0,
      status: r.status ?? 1,
      createUserString: r.create_user_string,
      createTime: r.create_time.toISOString(),
      updateUserString: r.update_user_string,
      updateTime: r.update_time ? r.update_time.toISOString() : '',
      children: [],
    };
    return ok(item);
  }

  /** POST /system/menu */
  @Post('/system/menu')
  async createMenu(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: MenuReq,
    @Req() req: any,
  ) {
    const begin = Date.now();
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }

    const type = body.type || 1;
    const title = (body.title ?? '').trim();
    if (!title) {
      return fail('400', '菜单标题不能为空');
    }

    let isExternal = body.isExternal ?? false;
    const isCache = body.isCache ?? false;
    const isHidden = body.isHidden ?? false;

    let path = (body.path ?? '').trim();
    let name = (body.name ?? '').trim();
    let component = (body.component ?? '').trim();

    if (isExternal) {
      if (!(path.startsWith('http://') || path.startsWith('https://'))) {
        return fail(
          '400',
          '路由地址格式不正确，请以 http:// 或 https:// 开头',
        );
      }
    } else {
      if (path.startsWith('http://') || path.startsWith('https://')) {
        return fail('400', '路由地址格式不正确');
      }
      if (path && !path.startsWith('/')) {
        path = `/${path}`;
      }
      if (name.startsWith('/')) {
        name = name.slice(1);
      }
      if (component.startsWith('/')) {
        component = component.slice(1);
      }
    }

    const sort = body.sort > 0 ? body.sort : 999;
    const status = body.status || 1;
    const now = new Date();
    const newId = BigInt(Date.now());

    try {
      await this.prisma.$executeRawUnsafe(
        `
INSERT INTO sys_menu (
    id, title, parent_id, type, path, name, component, redirect,
    icon, is_external, is_cache, is_hidden, permission, sort, status,
    create_user, create_time
)
VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8,
    $9, $10, $11, $12, $13, $14, $15,
    $16, $17
);
`,
        newId,
        title,
        BigInt(body.parentId || 0),
        type,
        path,
        name,
        component,
        body.redirect ?? '',
        body.icon ?? '',
        isExternal,
        isCache,
        isHidden,
        body.permission ?? '',
        sort,
        status,
        BigInt(currentUserId),
        now,
      );
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '菜单管理',
        description: `新增菜单[${title}]`,
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '菜单管理',
        description: `新增菜单[${title}]`,
        success: false,
        message: '新增菜单失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '新增菜单失败');
    }

    return ok({ id: Number(newId) });
  }

  /** PUT /system/menu/:id */
  @Put('/system/menu/:id')
  async updateMenu(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: MenuReq,
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

    const title = (body.title ?? '').trim();
    if (!title) {
      return fail('400', '菜单标题不能为空');
    }

    let isExternal = body.isExternal ?? false;
    const isCache = body.isCache ?? false;
    const isHidden = body.isHidden ?? false;

    let path = (body.path ?? '').trim();
    let name = (body.name ?? '').trim();
    let component = (body.component ?? '').trim();

    if (isExternal) {
      if (!(path.startsWith('http://') || path.startsWith('https://'))) {
        return fail(
          '400',
          '路由地址格式不正确，请以 http:// 或 https:// 开头',
        );
      }
    } else {
      if (path.startsWith('http://') || path.startsWith('https://')) {
        return fail('400', '路由地址格式不正确');
      }
      if (path && !path.startsWith('/')) {
        path = `/${path}`;
      }
      if (name.startsWith('/')) {
        name = name.slice(1);
      }
      if (component.startsWith('/')) {
        component = component.slice(1);
      }
    }

    const sort = body.sort > 0 ? body.sort : 999;
    const status = body.status || 1;

    try {
      await this.prisma.$executeRawUnsafe(
        `
UPDATE sys_menu
   SET title       = $1,
       parent_id   = $2,
       type        = $3,
       path        = $4,
       name        = $5,
       component   = $6,
       redirect    = $7,
       icon        = $8,
       is_external = $9,
       is_cache    = $10,
       is_hidden   = $11,
       permission  = $12,
       sort        = $13,
       status      = $14,
       update_user = $15,
       update_time = $16
 WHERE id          = $17;
`,
        title,
        BigInt(body.parentId || 0),
        body.type || 1,
        path,
        name,
        component,
        body.redirect ?? '',
        body.icon ?? '',
        isExternal,
        isCache,
        isHidden,
        body.permission ?? '',
        sort,
        status,
        BigInt(currentUserId),
        new Date(),
        BigInt(id),
      );
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '菜单管理',
        description: `修改菜单[${title}]`,
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '菜单管理',
        description: `修改菜单[${title}]`,
        success: false,
        message: '修改菜单失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '修改菜单失败');
    }

    return ok(true);
  }

  /** DELETE /system/menu */
  @Delete('/system/menu')
  async deleteMenu(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: IdsRequest,
    @Req() req: any,
  ) {
    const begin = Date.now();
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

    const nodes = await this.prisma.$queryRaw<
      { id: bigint; parent_id: bigint }[]
    >`SELECT id, parent_id FROM sys_menu;`;
    const childrenOf = new Map<number, number[]>();
    for (const n of nodes) {
      const pid = Number(n.parent_id);
      const id = Number(n.id);
      const arr = childrenOf.get(pid) ?? [];
      arr.push(id);
      childrenOf.set(pid, arr);
    }

    const seen = new Set<number>();
    const collect = (id: number) => {
      if (seen.has(id)) return;
      seen.add(id);
      const ch = childrenOf.get(id) || [];
      for (const c of ch) collect(c);
    };
    for (const id of ids) {
      collect(id);
    }
    if (!seen.size) {
      return ok(true);
    }
    const allIds = Array.from(seen);

    try {
      const statements = [
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_role_menu WHERE menu_id = ANY($1::bigint[])`,
          allIds.map((v) => BigInt(v)),
        ),
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_menu WHERE id = ANY($1::bigint[])`,
          allIds.map((v) => BigInt(v)),
        ),
      ];
      await (this.prisma as any).$transaction(statements as any);
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '菜单管理',
        description: '删除菜单',
        success: true,
        message: '',
        timeTakenMs: Date.now() - begin,
      });
    } catch {
      await writeOperationLog(this.prisma, {
        req,
        userId: currentUserId,
        module: '菜单管理',
        description: '删除菜单',
        success: false,
        message: '删除菜单失败',
        timeTakenMs: Date.now() - begin,
      });
      return fail('500', '删除菜单失败');
    }

    return ok(true);
  }

  /** DELETE /system/menu/cache 清除菜单缓存（目前为 no-op） */
  @Delete('/system/menu/cache')
  async clearCache() {
    return ok(true);
  }
}
