import { Body, Controller, Get, Headers, Post } from '@nestjs/common';
import { AuthService } from './auth.service';
import { LoginDto } from './dto/login.dto';
import { ok, fail } from '../../shared/api-response/api-response';
import { TokenService } from './jwt/jwt.service';

/**
 * 认证相关 HTTP 控制器。
 * 对外暴露 /auth/login、/auth/user/info、/auth/user/route，与 Java/Go/Python 一致。
 */
@Controller()
export class AuthController {
  constructor(
    private readonly authService: AuthService,
    private readonly tokenService: TokenService,
  ) {}

  @Post('/auth/login')
  async login(@Body() dto: LoginDto) {
    try {
      const resp = await this.authService.login(dto);
      return ok(resp);
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
