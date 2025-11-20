import { Controller, Get, Param, Query } from '@nestjs/common';
import { PrismaService } from '../../shared/prisma/prisma.service';
import { ok } from '../../shared/api-response/api-response';

interface LabelValue<T = any> {
  label: string;
  value: T;
  extra?: string;
}

interface TreeNode {
  key: number;
  title: string;
  disabled: boolean;
  children: TreeNode[];
}

/**
 * 通用下拉/树形接口，与 Python 版本 /common/* 行为保持一致。
 */
@Controller()
export class CommonController {
  constructor(private readonly prisma: PrismaService) {}

  /**
   * 网站基础配置字典数据，来源于 sys_option SITE 类别。
   */
  @Get('/common/dict/option/site')
  async listSiteOptions() {
    const rows = await this.prisma.$queryRaw<{ code: string; value: string }[]>`
SELECT code,
       COALESCE(value, default_value) AS value
FROM sys_option
WHERE category = 'SITE'
ORDER BY id ASC;
`;
    const data: LabelValue<string>[] = rows.map((r) => ({
      label: r.code,
      value: r.value,
    }));
    return ok(data);
  }

  /**
   * 菜单树，仅包含目录/菜单（type in (1,2)）。
   */
  @Get('/common/tree/menu')
  async listMenuTree() {
    const rows = await this.prisma.$queryRaw<
      { id: bigint; title: string; parent_id: bigint; sort: number; status: number }[]
    >`
SELECT id, title, parent_id, sort, status
FROM sys_menu
WHERE type IN (1, 2)
ORDER BY sort ASC, id ASC;
`;
    const flat = rows.map((r) => ({
      id: Number(r.id),
      title: r.title,
      parentId: Number(r.parent_id),
      sort: Number(r.sort),
      status: Number(r.status),
    }));
    if (flat.length === 0) {
      return ok<TreeNode[]>([]);
    }

    const nodeMap = new Map<number, TreeNode>();
    for (const m of flat) {
      nodeMap.set(m.id, {
        key: m.id,
        title: m.title,
        disabled: m.status !== 1,
        children: [],
      });
    }

    const roots: TreeNode[] = [];
    for (const m of flat) {
      const node = nodeMap.get(m.id)!;
      if (m.parentId === 0) {
        roots.push(node);
        continue;
      }
      const parent = nodeMap.get(m.parentId);
      if (!parent) {
        roots.push(node);
        continue;
      }
      parent.children.push(node);
    }

    return ok(roots);
  }

  /**
   * 部门树。
   */
  @Get('/common/tree/dept')
  async listDeptTree() {
    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        name: string;
        parent_id: bigint;
        sort: number;
        status: number;
        is_system: boolean;
      }[]
    >`
SELECT id, name, parent_id, sort, status, is_system
FROM sys_dept
ORDER BY sort ASC, id ASC;
`;
    const flat = rows.map((r) => ({
      id: Number(r.id),
      name: r.name,
      parentId: Number(r.parent_id),
      sort: Number(r.sort),
      status: Number(r.status),
      isSystem: r.is_system,
    }));
    if (flat.length === 0) {
      return ok<TreeNode[]>([]);
    }

    const nodeMap = new Map<number, TreeNode>();
    for (const d of flat) {
      nodeMap.set(d.id, {
        key: d.id,
        title: d.name,
        disabled: false,
        children: [],
      });
    }

    const roots: TreeNode[] = [];
    for (const d of flat) {
      const node = nodeMap.get(d.id)!;
      if (d.parentId === 0) {
        roots.push(node);
        continue;
      }
      const parent = nodeMap.get(d.parentId);
      if (!parent) {
        roots.push(node);
        continue;
      }
      parent.children.push(node);
    }

    return ok(roots);
  }

  /**
   * 用户字典：label=昵称/用户名，value=用户ID，extra=用户名。
   */
  @Get('/common/dict/user')
  async listUserDict(@Query('status') status?: string) {
    let sql = `
SELECT id,
       COALESCE(nickname, username, '') AS nickname,
       COALESCE(username, '')           AS username
FROM sys_user
WHERE status = 1
`;
    const params: any[] = [];
    const s = Number(status ?? 0);
    if (s > 0) {
      sql = `
SELECT id,
       COALESCE(nickname, username, '') AS nickname,
       COALESCE(username, '')           AS username
FROM sys_user
WHERE status = $1
`;
      params.push(s);
    }
    sql += ' ORDER BY id ASC;';

    const rows = await this.prisma.$queryRawUnsafe<
      { id: bigint; nickname: string; username: string }[]
    >(sql, ...params);

    const data: LabelValue<number>[] = rows.map((r) => ({
      label: r.nickname,
      value: Number(r.id),
      extra: r.username,
    }));
    return ok(data);
  }

  /**
   * 角色字典：label=角色名，value=角色ID，extra=角色编码。
   */
  @Get('/common/dict/role')
  async listRoleDict() {
    const rows = await this.prisma.$queryRaw<
      { id: bigint; name: string; code: string }[]
    >`
SELECT id, name, code
FROM sys_role
ORDER BY sort ASC, id ASC;
`;
    const data: LabelValue<number>[] = rows.map((r) => ({
      label: r.name,
      value: Number(r.id),
      extra: r.code,
    }));
    return ok(data);
  }

  /**
   * 通用字典项查询，根据 sys_dict / sys_dict_item。
   */
  @Get('/common/dict/:code')
  async listDictByCode(@Param('code') code: string) {
    const c = (code || '').trim();
    if (!c) {
      return ok<LabelValue<string>[]>([]);
    }
    const rows = await this.prisma.$queryRaw<
      { label: string; value: string; extra: string }[]
    >`
SELECT t1.label,
       t1.value,
       COALESCE(t1.color, '') AS extra
FROM sys_dict_item AS t1
LEFT JOIN sys_dict AS t2 ON t1.dict_id = t2.id
WHERE t1.status = 1
  AND t2.code = ${c}
ORDER BY t1.sort ASC, t1.id ASC;
`;
    const data: LabelValue<string>[] = rows.map((r) => ({
      label: r.label,
      value: r.value,
      extra: r.extra,
    }));
    return ok(data);
  }
}

