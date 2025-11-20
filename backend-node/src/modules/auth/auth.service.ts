import { Injectable } from '@nestjs/common';
import { PrismaService } from '../../shared/prisma/prisma.service';
import { RSADecryptor } from './security/rsa.service';
import { PasswordService } from './security/password.service';
import { TokenService } from './jwt/jwt.service';
import { LoginDto, LoginResp } from './dto/login.dto';
import { UserInfo } from './models/user-info.model';
import { RouteItem } from './models/route-item.model';

/**
 * 认证与当前用户相关业务逻辑。
 */
@Injectable()
export class AuthService {
  constructor(
    private readonly prisma: PrismaService,
    private readonly rsa: RSADecryptor,
    private readonly pwdService: PasswordService,
    private readonly tokenService: TokenService,
  ) {}

  /**
   * 登录流程，与 Go/Python 版保持一致。
   */
  async login(dto: LoginDto): Promise<LoginResp> {
    const authType = (dto.authType ?? '').trim().toUpperCase();
    if (authType && authType !== 'ACCOUNT') {
      throw new Error('暂不支持该认证方式');
    }
    if (!dto.clientId?.trim()) {
      throw new Error('客户端ID不能为空');
    }
    if (!dto.username?.trim()) {
      throw new Error('用户名不能为空');
    }
    if (!dto.password?.trim()) {
      throw new Error('密码不能为空');
    }

    let rawPassword: string;
    try {
      rawPassword = this.rsa.decryptBase64(dto.password);
    } catch {
      throw new Error('密码解密失败');
    }

    // 从 sys_user 查询用户
    const user = await this.prisma.sys_user.findFirst({
      where: { username: dto.username },
    });
    if (!user) {
      throw new Error('用户名或密码不正确');
    }

    const ok = this.pwdService.verify(rawPassword, user.password ?? '');
    if (!ok) {
      throw new Error('用户名或密码不正确');
    }
    if (user.status !== 1) {
      throw new Error('此账号已被禁用，如有疑问，请联系管理员');
    }

    const token = this.tokenService.generate(Number(user.id));
    return { token };
  }

  /**
   * 获取当前用户信息。
   */
  async getUserInfo(userId: number): Promise<UserInfo | null> {
    const user = await this.prisma.sys_user.findUnique({
      where: { id: BigInt(userId) },
    });
    if (!user) return null;

    // 角色
    const roles = await this.prisma.$queryRaw<
      { id: bigint; name: string; code: string; data_scope: number }[]
    >`
SELECT r.id, r.name, r.code, r.data_scope
FROM sys_role AS r
JOIN sys_user_role AS ur ON ur.role_id = r.id
WHERE ur.user_id = ${BigInt(userId)};
`;
    const roleCodes = roles.map((r) => r.code);

    // 权限
    const permsRows = await this.prisma.$queryRaw<{ permission: string }[]>`
SELECT DISTINCT m.permission
FROM sys_menu AS m
LEFT JOIN sys_role_menu AS rm ON rm.menu_id = m.id
LEFT JOIN sys_role AS r ON r.id = rm.role_id
LEFT JOIN sys_user_role AS ur ON ur.role_id = r.id
LEFT JOIN sys_user AS u ON u.id = ur.user_id
WHERE u.id = ${BigInt(userId)}
  AND m.status = 1
  AND m.permission IS NOT NULL;
`;
    const permissions = permsRows.map((p) => p.permission);

    // 部门名称
    let deptName = '';
    if (user.dept_id) {
      const dept = await this.prisma.sys_dept.findUnique({
        where: { id: user.dept_id },
        select: { name: true },
      });
      if (dept) {
        deptName = dept.name;
      }
    }

    const pwdResetTime = user.pwd_reset_time
      ? formatDateTime(user.pwd_reset_time)
      : '';
    const registrationDate = formatDateOnly(user.create_time);

    const opt = (v: string | null) => v ?? '';

    return {
      id: Number(user.id),
      username: user.username,
      nickname: user.nickname,
      gender: user.gender,
      email: opt(user.email),
      phone: opt(user.phone),
      avatar: opt(user.avatar),
      description: opt(user.description),
      pwdResetTime,
      pwdExpired: false,
      registrationDate,
      deptName,
      roles: roleCodes,
      permissions,
    };
  }

  /**
   * 获取当前用户路由树。
   */
  async getUserRoutes(userId: number): Promise<RouteItem[]> {
    const roles = await this.prisma.$queryRaw<
      { id: bigint; name: string; code: string; data_scope: number }[]
    >`
SELECT r.id, r.name, r.code, r.data_scope
FROM sys_role AS r
JOIN sys_user_role AS ur ON ur.role_id = r.id
WHERE ur.user_id = ${BigInt(userId)};
`;
    if (roles.length === 0) {
      return [];
    }
    const roleCodes = roles.map((r) => r.code);
    const roleIds = roles.map((r) => r.id);

    const menus = await this.prisma.$queryRaw<
      {
        id: bigint;
        parent_id: bigint;
        title: string;
        type: number;
        path: string | null;
        name: string | null;
        component: string | null;
        redirect: string | null;
        icon: string | null;
        is_external: boolean;
        is_cache: boolean;
        is_hidden: boolean;
        permission: string | null;
        sort: number;
        status: number;
      }[]
    >`
SELECT
  m.id,
  m.parent_id,
  m.title,
  m.type,
  m.path,
  m.name,
  m.component,
  m.redirect,
  m.icon,
  COALESCE(m.is_external, false) AS is_external,
  COALESCE(m.is_cache, false)    AS is_cache,
  COALESCE(m.is_hidden, false)   AS is_hidden,
  m.permission,
  COALESCE(m.sort, 0)            AS sort,
  m.status
FROM sys_menu AS m
JOIN sys_role_menu AS rm ON rm.menu_id = m.id
WHERE rm.role_id = ANY(${roleIds as any});
`;

    // 过滤掉按钮类型（3）
    const filtered = menus.filter((m) => Number(m.type) !== 3);
    if (filtered.length === 0) {
      return [];
    }

    // 排序：sort ↑，id ↑
    filtered.sort((a, b) => {
      if (a.sort === b.sort) {
        return Number(a.id - b.id);
      }
      return a.sort - b.sort;
    });

    const nodeMap = new Map<number, RouteItem>();
    for (const m of filtered) {
      const id = Number(m.id);
      nodeMap.set(id, {
        id,
        title: m.title,
        parentId: Number(m.parent_id),
        type: Number(m.type),
        path: m.path ?? '',
        name: m.name ?? '',
        component: m.component ?? '',
        redirect: m.redirect ?? '',
        icon: m.icon ?? '',
        isExternal: m.is_external,
        isHidden: m.is_hidden,
        isCache: m.is_cache,
        permission: m.permission ?? '',
        roles: roleCodes,
        sort: Number(m.sort),
        status: Number(m.status),
        children: [],
        activeMenu: '',
        alwaysShow: false,
        breadcrumb: true,
        showInTabs: true,
        affix: false,
      });
    }

    const roots: RouteItem[] = [];
    for (const item of nodeMap.values()) {
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

    const sortChildren = (nodes: RouteItem[]) => {
      for (const node of nodes) {
        if (node.children.length > 0) {
          node.children.sort((a, b) => {
            if (a.sort === b.sort) {
              return a.id - b.id;
            }
            return a.sort - b.sort;
          });
          sortChildren(node.children);
        }
      }
    };
    sortChildren(roots);

    return roots;
  }
}

function pad(num: number): string {
  return num < 10 ? `0${num}` : `${num}`;
}

function formatDateOnly(date: Date): string {
  const y = date.getFullYear();
  const m = pad(date.getMonth() + 1);
  const d = pad(date.getDate());
  return `${y}-${m}-${d}`;
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

