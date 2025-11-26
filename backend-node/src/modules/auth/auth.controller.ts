import { Body, Controller, Get, Headers, Post, Req } from '@nestjs/common';
import { AuthService } from './auth.service';
import { LoginDto } from './dto/login.dto';
import { ok, fail } from '../../shared/api-response/api-response';
import { TokenService } from './jwt/jwt.service';
import { OnlineStoreService } from './online.store';
import { PrismaService } from '../../shared/prisma/prisma.service';
import { nextId } from '../../shared/id/id';

/**
 * 认证相关 HTTP 控制器。
 * 对外暴露 /auth/login、/auth/user/info、/auth/user/route，与 Java/Go/Python 一致。
 */
@Controller()
export class AuthController {
  constructor(
    private readonly authService: AuthService,
    private readonly tokenService: TokenService,
    private readonly onlineStore: OnlineStoreService,
    private readonly prisma: PrismaService,
  ) {}

  @Post('/auth/login')
  async login(@Body() dto: LoginDto, @Req() req: any) {
    const begin = Date.now();
    try {
      const resp = await this.authService.login(dto);
      // 记录在线用户信息，兼容 /monitor/online 在线用户列表。
      const ipHeader =
        (req.headers?.['x-forwarded-for'] as string | undefined) || '';
      const realIp =
        (ipHeader.split(',')[0] || '').trim() ||
        (req.ip as string | undefined) ||
        '';
      const ua =
        (req.headers?.['user-agent'] as string | undefined) || '';
      this.onlineStore.recordLogin({
        userId: resp.userId,
        username: resp.username,
        nickname: resp.nickname,
        clientId: dto.clientId,
        token: resp.token,
        ip: realIp,
        userAgent: ua,
      });
      // 写入登录成功日志，便于系统日志模块展示登录行为。
      await this.writeLoginLog({
        req,
        success: true,
        message: '',
        userId: resp.userId,
        ip: realIp,
        userAgent: ua,
        timeTakenMs: Date.now() - begin,
      });
      // 对前端仅返回 token 字段，保持与原有协议兼容。
      return ok({ token: resp.token });
    } catch (e: any) {
      const msg = e?.message || '登录失败';
      // 登录失败同样写入系统日志。
      await this.writeLoginLog({
        req,
        success: false,
        message: msg,
        userId: 0,
        ip:
          (req.headers?.['x-forwarded-for'] as string | undefined) || '' ||
          (req.ip as string | undefined) ||
          '',
        userAgent:
          (req.headers?.['user-agent'] as string | undefined) || '',
        timeTakenMs: Date.now() - begin,
      });
      return fail('400', msg);
    }
  }

  /**
   * 登出接口，对应 Java 版 /auth/logout。
   * Node 采用无状态 JWT，这里仅解析当前 Token 并返回登录 ID，
   * 前端收到成功响应后自行丢弃 Token 即完成登出。
   */
  @Post('/auth/logout')
  async logout(@Headers('authorization') authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) {
      return fail('401', '未授权，请重新登录');
    }
    // 从 Authorization 中提取原始 token，并移除在线记录。
    const authz = authorization || '';
    let rawToken = authz.trim();
    if (rawToken.toLowerCase().startsWith('bearer ')) {
      rawToken = rawToken.slice(7).trim();
    }
    if (rawToken) {
      this.onlineStore.removeByToken(rawToken);
    }
    return ok(claims.userId);
  }

  @Get('/auth/user/info')
  async getUserInfo(@Headers('authorization') authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) {
      return fail('401', '未授权，请重新登录');
    }
    const user = await this.authService.getUserInfo(claims.userId);
    if (!user) {
      return fail('401', '未授权，请重新登录');
    }
    return ok(user);
  }

  @Get('/auth/user/route')
  async getUserRoute(@Headers('authorization') authorization?: string) {
    const claims = this.tokenService.parse(authorization || undefined);
    if (!claims) {
      return fail('401', '未授权，请重新登录');
    }
    const routes = await this.authService.getUserRoutes(claims.userId);
    return ok(routes);
  }

  /**
   * 写入 sys_log 登录相关记录，字段兼容 Java/Go 版本。
   * 失败时不影响主流程。
   */
  private async writeLoginLog(params: {
    req: any;
    success: boolean;
    message: string;
    userId: number;
    ip: string;
    userAgent: string;
    timeTakenMs: number;
  }): Promise<void> {
    try {
      const id = nextId();
      const now = new Date();
      const req = params.req || {};
      const url: string =
        (req.originalUrl as string | undefined) ||
        (req.url as string | undefined) ||
        '/auth/login';
      const method: string = (req.method as string | undefined) || 'POST';
      const headers = req.headers || {};
      const reqHeadersJson = (() => {
        try {
          return JSON.stringify(headers);
        } catch {
          return '';
        }
      })();
      // 为避免记录密码等敏感信息，这里不记录请求体。
      const statusCode = params.success ? 200 : 400;
      const status = params.success ? 1 : 2;
      const errorMsg = params.success ? '' : params.message || '';

      await this.prisma.$executeRawUnsafe(
        `
INSERT INTO sys_log (
    id, trace_id, description, module,
    request_url, request_method, request_headers, request_body,
    status_code, response_headers, response_body,
    time_taken, ip, address, browser, os,
    status, error_msg, create_user, create_time
) VALUES (
    $1, $2, $3, $4,
    $5, $6, $7, $8,
    $9, $10, $11,
    $12, $13, $14, $15, $16,
    $17, $18, $19, $20
);
`,
        id,
        '',
        '用户登录',
        '登录',
        url,
        method,
        reqHeadersJson,
        '',
        statusCode,
        '',
        '',
        BigInt(params.timeTakenMs || 0),
        (params.ip || '').slice(0, 100),
        '',
        (params.userAgent || '').slice(0, 100),
        '',
        status,
        errorMsg,
        params.userId ? BigInt(params.userId) : null,
        now,
      );
    } catch {
      // 忽略日志写入失败，不影响登录流程。
    }
  }
}
