import { Module } from '@nestjs/common';
import { AuthModule } from '../modules/auth/auth.module';
import { CaptchaModule } from '../modules/captcha/captcha.module';
import { CommonModule } from '../modules/common/common.module';
import { SystemModule } from '../modules/system/system.module';
import { PrismaModule } from '../shared/prisma/prisma.module';
import { SystemLogController } from './monitor/log/system-log.controller';

/**
 * 根模块，聚合各业务模块。
 */
@Module({
  imports: [PrismaModule, AuthModule, CaptchaModule, CommonModule, SystemModule],
  controllers: [SystemLogController],
})
export class AppModule {}
