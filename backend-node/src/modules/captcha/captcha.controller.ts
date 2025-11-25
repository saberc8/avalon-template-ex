import { Controller, Get } from '@nestjs/common';
import { randomUUID } from 'crypto';
import { ok } from '../../shared/api-response/api-response';
import { OptionService } from '../../shared/option/option.service';
import { setCaptcha } from '../../shared/captcha/captcha.store';

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
  constructor(private readonly optionService: OptionService) {}

  @Get('/captcha/image')
  async getImageCaptcha() {
    // 读取 LOGIN_CAPTCHA_ENABLED 配置：1 开启验证码，其余视为关闭
    const enabled = await this.optionService.getIntValueByCode(
      'LOGIN_CAPTCHA_ENABLED',
    );
    const now = Date.now();
    // 未启用验证码时，仅返回 isEnabled=false，保持兼容。
    if (enabled !== 1) {
      const resp: CaptchaResp = {
        uuid: '',
        img: '',
        expireTime: now,
        isEnabled: false,
      };
      return ok(resp);
    }

    // 生成 4 位数字验证码
    const code = String(Math.floor(1000 + Math.random() * 9000));
    const uuid = randomUUID();
    const expireMinutes = 2;
    const expireTime = now + expireMinutes * 60 * 1000;

    // 构造简单 SVG 图片，并转为 Base64 Data URL
    const svg = `<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"40\">\n  <rect width=\"100\" height=\"40\" fill=\"#f5f5f5\"/>\n  <text x=\"50\" y=\"26\" text-anchor=\"middle\" font-size=\"20\" fill=\"#333\" font-family=\"Arial, sans-serif\">${code}</text>\n</svg>`;
    const img =
      'data:image/svg+xml;base64,' +
      Buffer.from(svg, 'utf8').toString('base64');

    // 将验证码写入内存存储，供登录时校验
    setCaptcha(uuid, code, expireTime);

    const resp: CaptchaResp = {
      uuid,
      img,
      expireTime,
      isEnabled: true,
    };
    return ok(resp);
  }
}
