import { Injectable } from '@nestjs/common';

/**
 * 在线会话记录结构，对齐 Go 版 OnlineSession。
 */
interface OnlineSession {
  userId: number;
  username: string;
  nickname: string;
  token: string;
  clientType: string;
  clientId: string;
  ip: string;
  address: string;
  browser: string;
  os: string;
  loginTime: Date;
  lastActiveTime: Date;
}

/**
 * 在线用户响应结构，对齐前端 OnlineUserResp。
 */
export interface OnlineUserResp {
  id: number;
  token: string;
  username: string;
  nickname: string;
  clientType: string;
  clientId: string;
  ip: string;
  address: string;
  browser: string;
  os: string;
  loginTime: string;
  lastActiveTime: string;
}

/**
 * 在线用户查询条件。
 */
export interface OnlineUserQuery {
  page?: string;
  size?: string;
  nickname?: string;
  loginTime?: string | string[];
}

function pad(num: number): string {
  return num < 10 ? `0${num}` : `${num}`;
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

/**
 * 在线会话内存存储，生命周期与 Node 进程一致。
 */
@Injectable()
export class OnlineStoreService {
  private readonly sessions = new Map<string, OnlineSession>();

  /**
   * 记录登录会话信息，在登录成功后调用。
   */
  recordLogin(params: {
    userId: number;
    username: string;
    nickname: string;
    clientId: string;
    token: string;
    ip?: string;
    userAgent?: string;
  }): void {
    const { userId, username, nickname, clientId, token } = params;
    if (!userId || !token) {
      return;
    }
    const now = new Date();
    const ip = (params.ip || '').trim();
    const ua = (params.userAgent || '').trim();

    const session: OnlineSession = {
      userId,
      username,
      nickname,
      token,
      clientType: 'PC',
      clientId,
      ip,
      address: '',
      browser: ua,
      os: '',
      loginTime: now,
      lastActiveTime: now,
    };
    this.sessions.set(token, session);
  }

  /**
   * 根据 token 移除在线会话，用于强退或登出。
   */
  removeByToken(token: string): void {
    const t = token.trim();
    if (!t) {
      return;
    }
    this.sessions.delete(t);
  }

  /**
   * 查询在线用户列表，按登录时间倒序分页。
   */
  list(params: {
    nickname?: string;
    loginStart?: Date;
    loginEnd?: Date;
    page: number;
    size: number;
  }): { list: OnlineUserResp[]; total: number } {
    const page = params.page > 0 ? params.page : 1;
    const size = params.size > 0 ? params.size : 10;
    const nickname = (params.nickname || '').trim();
    const { loginStart, loginEnd } = params;

    const all: OnlineSession[] = Array.from(this.sessions.values());

    const filtered: OnlineSession[] = [];
    for (const sess of all) {
      if (
        nickname &&
        !sess.username.includes(nickname) &&
        !sess.nickname.includes(nickname)
      ) {
        continue;
      }
      if (loginStart && sess.loginTime < loginStart) {
        continue;
      }
      if (loginEnd && sess.loginTime > loginEnd) {
        continue;
      }
      filtered.push(sess);
    }

    if (filtered.length > 1) {
      filtered.sort((a, b) => b.loginTime.getTime() - a.loginTime.getTime());
    }

    const total = filtered.length;
    const start = (page - 1) * size;
    const end = start + size;
    const pageItems = filtered.slice(
      start < 0 ? 0 : start,
      end > filtered.length ? filtered.length : end,
    );

    const list: OnlineUserResp[] = pageItems.map((sess) => ({
      id: sess.userId,
      token: sess.token,
      username: sess.username,
      nickname: sess.nickname,
      clientType: sess.clientType,
      clientId: sess.clientId,
      ip: sess.ip,
      address: sess.address,
      browser: sess.browser,
      os: sess.os,
      loginTime: formatDateTime(sess.loginTime),
      lastActiveTime: formatDateTime(sess.lastActiveTime),
    }));

    return { list, total };
  }
}

