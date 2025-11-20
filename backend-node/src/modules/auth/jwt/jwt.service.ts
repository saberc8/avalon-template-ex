import { Injectable } from '@nestjs/common';
import * as jwt from 'jsonwebtoken';

/**
 * JWT Claims，仅保留 userId 字段以匹配前端。
 */
export interface TokenClaims {
  userId: number;
  iat: number;
  exp: number;
}

@Injectable()
export class TokenService {
  private readonly secret: string;
  private readonly ttlSeconds: number;

  constructor() {
    // 与 Python/Go 版本配置保持一致
    this.secret = process.env.AUTH_JWT_SECRET || 'asdasdasifhueuiwyurfewbfjsdafjk';
    const hours = Number(process.env.AUTH_JWT_TTL_HOURS || '24');
    this.ttlSeconds = hours * 3600;
  }

  /**
   * 生成包含 userId、iat、exp 的 HS256 Token。
   */
  generate(userId: number): string {
    const now = Math.floor(Date.now() / 1000);
    const payload: TokenClaims = {
      userId,
      iat: now,
      exp: now + this.ttlSeconds,
    };
    return jwt.sign(payload, this.secret, { algorithm: 'HS256' });
  }

  /**
   * 解析 Authorization 头中的 Token，返回 TokenClaims 或 null。
   * 支持 `Bearer xxx` 形式。
   */
  parse(authHeader?: string): TokenClaims | null {
    if (!authHeader) return null;
    let token = authHeader.trim();
    if (token.toLowerCase().startsWith('bearer ')) {
      token = token.slice(7).trim();
    }
    try {
      const decoded = jwt.verify(token, this.secret) as TokenClaims;
      if (typeof decoded.userId !== 'number') {
        return null;
      }
      return decoded;
    } catch {
      return null;
    }
  }
}

