import {
  Controller,
  Delete,
  Get,
  Headers,
  Param,
  Query,
} from '@nestjs/common';
import { ok, fail } from '../../shared/api-response/api-response';
import { TokenService } from './jwt/jwt.service';
import {
  OnlineStoreService,
  OnlineUserQuery,
  OnlineUserResp,
} from './online.store';
import { PageResult } from '../system/user/dto';

/**
 * 在线用户相关接口：
 * - GET /monitor/online
 * - DELETE /monitor/online/:token
 * 行为对齐 Java/Go 版本，兼容前端在线用户页面。
 */
@Controller()
export class OnlineUserController {
  constructor(
    private readonly store: OnlineStoreService,
    private readonly tokenService: TokenService,
  ) {}

  /** 分页查询在线用户列表：GET /monitor/online */
  @Get('/monitor/online')
  async pageOnline(@Query() query: OnlineUserQuery) {
    let page = Number(query.page ?? 1);
    let size = Number(query.size ?? 10);
    if (!Number.isFinite(page) || page <= 0) page = 1;
    if (!Number.isFinite(size) || size <= 0) size = 10;

    const nickname = (query.nickname || '').trim();

    let loginStart: Date | undefined;
    let loginEnd: Date | undefined;
    const range = query.loginTime;
    const arr = Array.isArray(range) ? range : range ? [range] : [];
    if (arr.length === 2) {
      const [startStr, endStr] = arr;
      const s = parseDateTime(startStr);
      const e = parseDateTime(endStr);
      if (s) loginStart = s;
      if (e) loginEnd = e;
    }

    const { list, total } = this.store.list({
      nickname,
      loginStart,
      loginEnd,
      page,
      size,
    });

    const resp: PageResult<OnlineUserResp> = {
      list,
      total,
    };
    return ok(resp);
  }

  /** 强退在线用户：DELETE /monitor/online/:token */
  @Delete('/monitor/online/:token')
  async kickout(
    @Headers('authorization') authorization: string | undefined,
    @Param('token') tokenParam: string,
  ) {
    const token = (tokenParam || '').trim();
    if (!token) {
      return fail('400', '令牌不能为空');
    }

    const authz = authorization || '';
    let rawCurrent = authz.trim();
    if (rawCurrent.toLowerCase().startsWith('bearer ')) {
      rawCurrent = rawCurrent.slice(7).trim();
    }

    if (rawCurrent && rawCurrent === token) {
      return fail('400', '不能强退自己');
    }

    if (!authz || !this.tokenService.parse(authz)) {
      return fail('401', '未授权，请重新登录');
    }

    this.store.removeByToken(token);
    return ok(true);
  }
}

function parseDateTime(value: string | undefined): Date | undefined {
  if (!value) return undefined;
  const s = value.trim();
  if (!s) return undefined;
  const iso = s.replace(' ', 'T');
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) {
    return undefined;
  }
  return d;
}

