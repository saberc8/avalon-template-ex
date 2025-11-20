import { Injectable } from '@nestjs/common';
import * as bcrypt from 'bcryptjs';

/**
 * 密码校验服务，使用 bcrypt，与 Java/Go/Python 一致。
 */
@Injectable()
export class PasswordService {
  verify(rawPassword: string, encodedPassword: string): boolean {
    if (!rawPassword || !encodedPassword) return false;
    return bcrypt.compareSync(rawPassword, encodedPassword);
  }
}

