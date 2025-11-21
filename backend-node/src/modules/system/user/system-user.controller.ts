import {
  Body,
  Controller,
  Delete,
  Get,
  Headers,
  Param,
  Patch,
  Post,
  Put,
  Query,
  UploadedFile,
  UseInterceptors,
} from '@nestjs/common';
import { FileInterceptor } from '@nestjs/platform-express';
import { PrismaService } from '../../../shared/prisma/prisma.service';
import { ok, fail } from '../../../shared/api-response/api-response';
import { TokenService } from '../../auth/jwt/jwt.service';
import { RSADecryptor } from '../../auth/security/rsa.service';
import { PasswordService } from '../../auth/security/password.service';
import {
  SystemUserDetailResp,
  SystemUserListQuery,
  SystemUserReq,
  SystemUserPasswordResetReq,
  SystemUserResp,
  SystemUserRoleUpdateReq,
  PageResult,
  IdsRequest,
  UserImportParseResp,
  UserImportResultResp,
} from './dto';
import { nextId } from '../../../shared/id/id';

/**
 * 用户管理接口集合，路径前缀 /system/user*。
 * 参考 Go 版 SystemUserHandler 与 Java UserController 设计，
 * 确保与前端协议完全兼容。
 */
@Controller()
export class SystemUserController {
  constructor(
    private readonly prisma: PrismaService,
    private readonly tokenService: TokenService,
    private readonly rsa: RSADecryptor,
    private readonly pwdService: PasswordService,
  ) {}

  /** 从 Authorization 头解析当前登录用户 ID，失败时返回 0 并直接输出失败响应。 */
  private currentUserId(authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) {
      return 0;
    }
    return claims.userId;
  }

  /** 分页查询用户列表：GET /system/user */
  @Get('/system/user')
  async listUserPage(@Query() query: SystemUserListQuery) {
    let page = Number(query.page ?? 1);
    let size = Number(query.size ?? 10);
    if (!Number.isFinite(page) || page <= 0) page = 1;
    if (!Number.isFinite(size) || size <= 0) size = 10;

    const desc = (query.description || '').trim();
    const statusStr = (query.status || '').trim();
    const deptStr = (query.deptId || '').trim();

    let statusFilter = 0;
    let deptId = 0;
    if (statusStr) {
      statusFilter = Number(statusStr) || 0;
    }
    if (deptStr) {
      deptId = Number(deptStr) || 0;
    }

    let where = 'WHERE 1=1';
    const args: any[] = [];
    let argPos = 1;
    if (desc) {
      where += ` AND (u.username ILIKE $${argPos} OR u.nickname ILIKE $${argPos} OR COALESCE(u.description,'') ILIKE $${argPos})`;
      args.push(`%${desc}%`);
      argPos++;
    }
    if (statusFilter !== 0) {
      where += ` AND u.status = $${argPos}`;
      args.push(statusFilter);
      argPos++;
    }
    if (deptId !== 0) {
      where += ` AND u.dept_id = $${argPos}`;
      args.push(deptId);
      argPos++;
    }

    const countSql = `SELECT COUNT(*)::bigint AS cnt FROM sys_user AS u ${where}`;
    const countRows = await this.prisma.$queryRawUnsafe<{ cnt: bigint }[]>(
      countSql,
      ...args,
    );
    const total = countRows[0]?.cnt ? Number(countRows[0].cnt) : 0;
    if (!total) {
      const empty: PageResult<SystemUserResp> = { list: [], total: 0 };
      return ok(empty);
    }

    const offset = BigInt((page - 1) * size);
    const limitPos = argPos;
    const offsetPos = argPos + 1;
    const argsWithPage = [...args, BigInt(size), offset];

    const listSql = `
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, ''),
       u.create_time,
       COALESCE(cu.nickname, ''),
       u.update_time,
       COALESCE(uu.nickname, '')
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
${where}
ORDER BY u.id DESC
LIMIT $${limitPos} OFFSET $${offsetPos};
`;

    const rows = await this.prisma.$queryRawUnsafe<
      {
        id: bigint;
        username: string;
        nickname: string;
        avatar: string | null;
        gender: number;
        email: string | null;
        phone: string | null;
        description: string | null;
        status: number;
        is_system: boolean;
        dept_id: bigint | null;
        dept_name: string;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >(listSql, ...argsWithPage);

    const users: SystemUserResp[] = rows.map((r) => ({
      id: Number(r.id),
      username: r.username,
      nickname: r.nickname,
      avatar: r.avatar ?? '',
      gender: r.gender,
      email: r.email ?? '',
      phone: r.phone ?? '',
      description: r.description ?? '',
      status: r.status,
      isSystem: r.is_system,
      createUserString: r.create_user_string,
      createTime: r.create_time.toISOString(),
      updateUserString: r.update_user_string,
      updateTime: r.update_time ? r.update_time.toISOString() : '',
      deptId: r.dept_id ? Number(r.dept_id) : 0,
      deptName: r.dept_name,
      roleIds: [],
      roleNames: [],
      disabled: r.is_system,
    }));

    await this.fillUserRoles(users);

    const resp: PageResult<SystemUserResp> = {
      list: users,
      total,
    };
    return ok(resp);
  }

  /** 查询所有用户列表：GET /system/user/list */
  @Get('/system/user/list')
  async listAllUser(@Query('userIds') userIds?: string | string[]) {
    const idList: number[] = [];
    const arr = Array.isArray(userIds) ? userIds : userIds ? [userIds] : [];
    for (const s of arr) {
      const v = Number(s);
      if (Number.isFinite(v) && v > 0) idList.push(v);
    }

    let rows;
    if (idList.length > 0) {
      rows = await this.prisma.$queryRawUnsafe<
        {
          id: bigint;
          username: string;
          nickname: string;
          avatar: string | null;
          gender: number;
          email: string | null;
          phone: string | null;
          description: string | null;
          status: number;
          is_system: boolean;
          dept_id: bigint | null;
          dept_name: string;
          create_time: Date;
          create_user_string: string;
          update_time: Date | null;
          update_user_string: string;
        }[]
      >(
        `
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '') AS dept_name,
       u.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       u.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE u.id = ANY($1::bigint[])
ORDER BY u.id DESC;
`,
        idList,
      );
    } else {
      rows = await this.prisma.$queryRawUnsafe<
        {
          id: bigint;
          username: string;
          nickname: string;
          avatar: string | null;
          gender: number;
          email: string | null;
          phone: string | null;
          description: string | null;
          status: number;
          is_system: boolean;
          dept_id: bigint | null;
          dept_name: string;
          create_time: Date;
          create_user_string: string;
          update_time: Date | null;
          update_user_string: string;
        }[]
      >`
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '') AS dept_name,
       u.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       u.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
ORDER BY u.id DESC;
`;
    }

    const users: SystemUserResp[] = rows.map((r) => ({
      id: Number(r.id),
      username: r.username,
      nickname: r.nickname,
      avatar: r.avatar ?? '',
      gender: r.gender,
      email: r.email ?? '',
      phone: r.phone ?? '',
      description: r.description ?? '',
      status: r.status,
      isSystem: r.is_system,
      createUserString: r.create_user_string,
      createTime: r.create_time.toISOString(),
      updateUserString: r.update_user_string,
      updateTime: r.update_time ? r.update_time.toISOString() : '',
      deptId: r.dept_id ? Number(r.dept_id) : 0,
      deptName: r.dept_name,
      roleIds: [],
      roleNames: [],
      disabled: r.is_system,
    }));

    await this.fillUserRoles(users);
    return ok(users);
  }

  /** 详情：GET /system/user/:id */
  @Get('/system/user/:id')
  async getUserDetail(@Param('id') idParam: string) {
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const rows = await this.prisma.$queryRaw<
      {
        id: bigint;
        username: string;
        nickname: string;
        avatar: string | null;
        gender: number;
        email: string | null;
        phone: string | null;
        description: string | null;
        status: number;
        is_system: boolean;
        dept_id: bigint | null;
        dept_name: string;
        pwd_reset_time: Date | null;
        create_time: Date;
        create_user_string: string;
        update_time: Date | null;
        update_user_string: string;
      }[]
    >`
SELECT u.id,
       u.username,
       u.nickname,
       COALESCE(u.avatar, ''),
       u.gender,
       COALESCE(u.email, ''),
       COALESCE(u.phone, ''),
       COALESCE(u.description, ''),
       u.status,
       u.is_system,
       u.dept_id,
       COALESCE(d.name, '') AS dept_name,
       u.pwd_reset_time,
       u.create_time,
       COALESCE(cu.nickname, '') AS create_user_string,
       u.update_time,
       COALESCE(uu.nickname, '') AS update_user_string
FROM sys_user AS u
LEFT JOIN sys_dept AS d ON d.id = u.dept_id
LEFT JOIN sys_user AS cu ON cu.id = u.create_user
LEFT JOIN sys_user AS uu ON uu.id = u.update_user
WHERE u.id = ${BigInt(id)};
`;

    if (!rows.length) {
      return fail('404', '用户不存在');
    }
    const r = rows[0];
    const base: SystemUserResp = {
      id: Number(r.id),
      username: r.username,
      nickname: r.nickname,
      avatar: r.avatar ?? '',
      gender: r.gender,
      email: r.email ?? '',
      phone: r.phone ?? '',
      description: r.description ?? '',
      status: r.status,
      isSystem: r.is_system,
      createUserString: r.create_user_string,
      createTime: r.create_time.toISOString(),
      updateUserString: r.update_user_string,
      updateTime: r.update_time ? r.update_time.toISOString() : '',
      deptId: r.dept_id ? Number(r.dept_id) : 0,
      deptName: r.dept_name,
      roleIds: [],
      roleNames: [],
      disabled: r.is_system,
    };

    const detail: SystemUserDetailResp = {
      ...base,
      pwdResetTime: r.pwd_reset_time ? r.pwd_reset_time.toISOString() : undefined,
    };

    await this.fillUserRoles([base]);
    detail.roleIds = base.roleIds;
    detail.roleNames = base.roleNames;

    return ok(detail);
  }

  /** 新增用户：POST /system/user */
  @Post('/system/user')
  async createUser(
    @Headers('authorization') authorization: string | undefined,
    @Body() body: SystemUserReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }

    const username = (body.username ?? '').trim();
    const nickname = (body.nickname ?? '').trim();
    if (!username || !nickname) {
      return fail('400', '用户名和昵称不能为空');
    }
    if (!body.deptId) {
      return fail('400', '所属部门不能为空');
    }
    const status = body.status || 1;
    const encryptedPwd = (body.password ?? '').trim();
    if (!encryptedPwd) {
      return fail('400', '密码不能为空');
    }

    let rawPwd: string;
    try {
      rawPwd = this.rsa.decryptBase64(encryptedPwd);
    } catch {
      return fail('400', '密码解密失败');
    }
    if (rawPwd.length < 8 || rawPwd.length > 32) {
      return fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
    }
    let hasLetter = false;
    let hasDigit = false;
    for (const ch of rawPwd) {
      if (ch >= '0' && ch <= '9') hasDigit = true;
      if (
        (ch >= 'a' && ch <= 'z') ||
        (ch >= 'A' && ch <= 'Z')
      ) {
        hasLetter = true;
      }
    }
    if (!hasLetter || !hasDigit) {
      return fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
    }

    const encodedPwd = this.pwdService.hash(rawPwd);
    const now = new Date();
    const newId = nextId();

    const tx = this.prisma.$transaction;
    try {
      await tx([
        this.prisma.$executeRawUnsafe(
          `
INSERT INTO sys_user (
    id, username, nickname, password, gender, email, phone, avatar,
    description, status, is_system, pwd_reset_time, dept_id,
    create_user, create_time
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
        $9, $10, FALSE, $11, $12,
        $13, $14);
`,
          newId,
          username,
          nickname,
          encodedPwd,
          body.gender,
          body.email ?? '',
          body.phone ?? '',
          body.avatar ?? '',
          body.description ?? '',
          status,
          now,
          BigInt(body.deptId),
          BigInt(currentUserId),
          now,
        ),
        ...(body.roleIds || []).map((rid) =>
          this.prisma.$executeRawUnsafe(
            `
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
`,
            nextId(),
            newId,
            BigInt(rid),
          ),
        ),
      ]);
    } catch {
      return fail('500', '新增用户失败');
    }

    return ok({ id: Number(newId) });
  }

  /** 修改用户：PUT /system/user/:id */
  @Put('/system/user/:id')
  async updateUser(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: SystemUserReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const username = (body.username ?? '').trim();
    const nickname = (body.nickname ?? '').trim();
    if (!username || !nickname) {
      return fail('400', '用户名和昵称不能为空');
    }
    if (!body.deptId) {
      return fail('400', '所属部门不能为空');
    }
    const status = body.status || 1;

    const now = new Date();
    const userIdBig = BigInt(id);

    const tx = this.prisma.$transaction;
    try {
      await tx([
        this.prisma.$executeRawUnsafe(
          `
UPDATE sys_user
   SET username    = $1,
       nickname    = $2,
       gender      = $3,
       email       = $4,
       phone       = $5,
       avatar      = $6,
       description = $7,
       status      = $8,
       dept_id     = $9,
       update_user = $10,
       update_time = $11
 WHERE id          = $12;
`,
          username,
          nickname,
          body.gender,
          body.email ?? '',
          body.phone ?? '',
          body.avatar ?? '',
          body.description ?? '',
          status,
          BigInt(body.deptId),
          BigInt(currentUserId),
          now,
          userIdBig,
        ),
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_user_role WHERE user_id = $1`,
          userIdBig,
        ),
        ...(body.roleIds || []).map((rid) =>
          this.prisma.$executeRawUnsafe(
            `
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
`,
            nextId(),
            userIdBig,
            BigInt(rid),
          ),
        ),
      ]);
    } catch {
      return fail('500', '修改用户失败');
    }

    return ok(true);
  }

  /** 删除用户：DELETE /system/user */
  @Delete('/system/user')
  async deleteUser(@Body() body: IdsRequest) {
    if (!body?.ids?.length) {
      return fail('400', 'ID 列表不能为空');
    }
    const idList = body.ids.filter((v) => Number.isFinite(v) && v > 0);
    if (!idList.length) {
      return fail('400', 'ID 列表不能为空');
    }
    const idsBig = idList.map((v) => BigInt(v));

    const tx = this.prisma.$transaction;
    try {
      await tx([
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_user_role WHERE user_id = ANY($1::bigint[])`,
          idsBig,
        ),
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_user WHERE id = ANY($1::bigint[])`,
          idsBig,
        ),
      ]);
    } catch {
      return fail('500', '删除用户失败');
    }
    return ok(true);
  }

  /** 重置密码：PATCH /system/user/:id/password */
  @Patch('/system/user/:id/password')
  async resetPassword(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: SystemUserPasswordResetReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const enc = (body.newPassword ?? '').trim();
    if (!enc) {
      return fail('400', '密码不能为空');
    }

    let rawPwd: string;
    try {
      rawPwd = this.rsa.decryptBase64(enc);
    } catch {
      return fail('400', '密码解密失败');
    }
    if (rawPwd.length < 8 || rawPwd.length > 32) {
      return fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
    }
    let hasLetter = false;
    let hasDigit = false;
    for (const ch of rawPwd) {
      if (ch >= '0' && ch <= '9') hasDigit = true;
      if (
        (ch >= 'a' && ch <= 'z') ||
        (ch >= 'A' && ch <= 'Z')
      ) {
        hasLetter = true;
      }
    }
    if (!hasLetter || !hasDigit) {
      return fail('400', '密码长度为 8-32 个字符，至少包含字母和数字');
    }

    const encodedPwd = this.pwdService.hash(rawPwd);
    const now = new Date();

    try {
      await this.prisma.$executeRawUnsafe(
        `
UPDATE sys_user
   SET password = $1,
       pwd_reset_time = $2,
       update_user = $3,
       update_time = $4
 WHERE id = $5;
`,
        encodedPwd,
        now,
        BigInt(currentUserId),
        now,
        BigInt(id),
      );
    } catch {
      return fail('500', '重置密码失败');
    }

    return ok(true);
  }

  /** 分配角色：PATCH /system/user/:id/role */
  @Patch('/system/user/:id/role')
  async updateRole(
    @Headers('authorization') authorization: string | undefined,
    @Param('id') idParam: string,
    @Body() body: SystemUserRoleUpdateReq,
  ) {
    const currentUserId = this.currentUserId(authorization);
    if (!currentUserId) {
      return fail('401', '未授权，请重新登录');
    }
    const id = Number(idParam);
    if (!Number.isFinite(id) || id <= 0) {
      return fail('400', 'ID 参数不正确');
    }

    const roleIds = body.roleIds || [];
    const tx = this.prisma.$transaction;
    try {
      await tx([
        this.prisma.$executeRawUnsafe(
          `DELETE FROM sys_user_role WHERE user_id = $1`,
          BigInt(id),
        ),
        ...roleIds.map((rid) =>
          this.prisma.$executeRawUnsafe(
            `
INSERT INTO sys_user_role (id, user_id, role_id)
VALUES ($1, $2, $3)
ON CONFLICT (user_id, role_id) DO NOTHING;
`,
            nextId(),
            BigInt(id),
            BigInt(rid),
          ),
        ),
      ]);
    } catch {
      return fail('500', '分配角色失败');
    }

    return ok(true);
  }

  /** 导出用户：GET /system/user/export（简化为 CSV 文本） */
  @Get('/system/user/export')
  async exportUser() {
    const rows = await this.prisma.$queryRaw<
      { username: string; nickname: string; gender: number; email: string | null; phone: string | null }[]
    >`
SELECT username, nickname, gender, COALESCE(email,''), COALESCE(phone,'')
FROM sys_user
ORDER BY id;
`;
    const lines = ['username,nickname,gender,email,phone'];
    for (const r of rows) {
      const line = `${r.username},${r.nickname},${r.gender},${r.email ?? ''},${r.phone ?? ''}`;
      lines.push(line);
    }
    const content = lines.join('\n');
    // 这里直接返回字符串，由网关或上层框架设置正确的响应头。
    return content;
  }

  /** 下载导入模板：GET /system/user/import/template */
  @Get('/system/user/import/template')
  async downloadImportTemplate() {
    const content = 'username,nickname,gender,email,phone\n';
    return content;
  }

  /** 解析导入：POST /system/user/import/parse（占位实现，保持流程可用） */
  @Post('/system/user/import/parse')
  @UseInterceptors(FileInterceptor('file'))
  async parseImport(@UploadedFile() file?: Express.Multer.File) {
    if (!file) {
      return fail('400', '文件不能为空');
    }
    const resp: UserImportParseResp = {
      importKey: Date.now().toString(),
      totalRows: 0,
      validRows: 0,
      duplicateUserRows: 0,
      duplicateEmailRows: 0,
      duplicatePhoneRows: 0,
    };
    return ok(resp);
  }

  /** 导入用户：POST /system/user/import（占位实现） */
  @Post('/system/user/import')
  async importUser(@Body() _body: any) {
    const resp: UserImportResultResp = {
      totalRows: 0,
      insertRows: 0,
      updateRows: 0,
    };
    return ok(resp);
  }

  /** 为用户列表填充角色 ID/名称信息。 */
  private async fillUserRoles(users: SystemUserResp[]) {
    if (!users.length) return;
    const userIds = Array.from(
      new Set(users.map((u) => u.id).filter((id) => id > 0)),
    );
    if (!userIds.length) return;

    const rows = await this.prisma.$queryRaw<
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
    for (const r of rows) {
      const uid = Number(r.user_id);
      const rid = Number(r.role_id);
      const entry = map.get(uid) ?? { ids: [], names: [] };
      entry.ids.push(rid);
      entry.names.push(r.name);
      map.set(uid, entry);
    }

    for (const u of users) {
      const entry = map.get(u.id);
      if (entry) {
        u.roleIds = entry.ids;
        u.roleNames = entry.names;
      }
    }
  }
}

