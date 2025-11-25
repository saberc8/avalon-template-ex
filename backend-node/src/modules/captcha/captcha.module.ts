import { Module } from '@nestjs/common';
import { CaptchaController } from './captcha.controller';
import { OptionService } from '../../shared/option/option.service';

@Module({
  controllers: [CaptchaController],
  providers: [OptionService],
})
export class CaptchaModule {}
