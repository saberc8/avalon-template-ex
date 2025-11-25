import { Injectable } from '@nestjs/common';
import * as bcrypt from 'bcryptjs';

/**
 * 密码加解密服务，使用 bcrypt，与 Java/Go/Python 一致。
 */
@Injectable()
export class PasswordService {
  /**
   * 校验明文密码与加密密码是否匹配，兼容 {bcrypt} 前缀。
   */
  verify(rawPassword: string, encodedPassword: string): boolean {
    if (!rawPassword || !encodedPassword) return false;
    let enc = encodedPassword;
    if (enc.startsWith('{bcrypt}')) {
      enc = enc.substring('{bcrypt}'.length);
    }
    try {
      return bcrypt.compareSync(rawPassword, enc);
    } catch {
      return false;
    }
  }

  /**
   * 生成带 {bcrypt} 前缀的密码哈希，用于入库。
   */
  hash(rawPassword: string): string {
    if (!rawPassword) {
      throw new Error('密码不能为空');
    }
    const hashed = bcrypt.hashSync(rawPassword, 10);
    return `{bcrypt}${hashed}`;
  }
}
