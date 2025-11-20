import { Module } from '@nestjs/common';
import { PrismaModule } from '../shared/prisma/prisma.module';
import { AuthModule } from '../modules/auth/auth.module';
import { CaptchaModule } from '../modules/captcha/captcha.module';
import { CommonModule } from '../modules/common/common.module';

/**
 * 根模块，聚合各业务模块。
 */
@Module({
  imports: [PrismaModule, AuthModule, CaptchaModule, CommonModule],
})
export class AppModule {}

