import { Controller, Get } from '@nestjs/common';
import { ok } from '../../shared/api-response/api-response';

interface CaptchaResp {
  uuid: string;
  img: string;
  expireTime: number;
  isEnabled: boolean;
}

/**
 * 验证码接口，当前行为与 Go/Python 版保持一致：始终返回 isEnabled=false。
 */
@Controller()
export class CaptchaController {
  @Get('/captcha/image')
  getImageCaptcha() {
    const now = Date.now();
    const resp: CaptchaResp = {
      uuid: '',
      img: '',
      expireTime: now,
      isEnabled: false,
    };
    return ok(resp);
  }
}

