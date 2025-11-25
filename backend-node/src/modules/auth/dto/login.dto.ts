import { IsNotEmpty, IsOptional, IsString } from 'class-validator';

/**
 * 登录请求 DTO，与前端 LoginParams、Python/Go 版 LoginRequest 对齐。
 */
export class LoginDto {
  @IsOptional()
  @IsString()
  authType?: string;

  @IsString()
  @IsNotEmpty({ message: '客户端ID不能为空' })
  clientId!: string;

  @IsString()
  @IsNotEmpty({ message: '用户名不能为空' })
  username!: string;

  @IsString()
  @IsNotEmpty({ message: '密码不能为空' })
  password!: string;

  @IsOptional()
  @IsString()
  captcha?: string;

  @IsOptional()
  @IsString()
  uuid?: string;
}

/**
 * 登录成功响应数据结构。
 */
export interface LoginResp {
  token: string;
  userId: number;
  username: string;
  nickname: string;
}
