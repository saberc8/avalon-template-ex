import { Module } from '@nestjs/common';
import { AuthController } from './auth.controller';
import { AuthService } from './auth.service';
import { RSADecryptor } from './security/rsa.service';
import { PasswordService } from './security/password.service';
import { TokenService } from './jwt/jwt.service';
import { OnlineStoreService } from './online.store';
import { OnlineUserController } from './online.controller';
import { OptionService } from '../../shared/option/option.service';

/**
 * 认证模块，聚合登录/当前用户与在线用户相关能力。
 */
@Module({
  controllers: [AuthController, OnlineUserController],
  providers: [
    AuthService,
    RSADecryptor,
    PasswordService,
    TokenService,
    OnlineStoreService,
    OptionService,
  ],
})
export class AuthModule {}
