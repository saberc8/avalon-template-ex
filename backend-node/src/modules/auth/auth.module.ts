import { Module } from '@nestjs/common';
import { AuthController } from './auth.controller';
import { AuthService } from './auth.service';
import { RSADecryptor } from './security/rsa.service';
import { PasswordService } from './security/password.service';
import { TokenService } from './jwt/jwt.service';

/**
 * 认证模块，聚合登录/当前用户相关能力。
 */
@Module({
  controllers: [AuthController],
  providers: [AuthService, RSADecryptor, PasswordService, TokenService],
})
export class AuthModule {}

