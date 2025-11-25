import { Body, Controller, Get, Headers, Post, Req } from '@nestjs/common';
import { AuthService } from './auth.service';
import { LoginDto } from './dto/login.dto';
import { ok, fail } from '../../shared/api-response/api-response';
import { TokenService } from './jwt/jwt.service';
import { OnlineStoreService } from './online.store';

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
  ) {}

  @Post('/auth/login')
  async login(@Body() dto: LoginDto, @Req() req: any) {
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
      // 对前端仅返回 token 字段，保持与原有协议兼容。
      return ok({ token: resp.token });
    } catch (e: any) {
      const msg = e?.message || '登录失败';
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
}
