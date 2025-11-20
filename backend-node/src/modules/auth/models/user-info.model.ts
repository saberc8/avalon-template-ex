/**
 * /auth/user/info 返回的用户信息结构。
 * 与前端 UserInfo、Go/Python 版本保持字段一致。
 */
export interface UserInfo {
  id: number;
  username: string;
  nickname: string;
  gender: number;
  email: string;
  phone: string;
  avatar: string;
  description: string;
  pwdResetTime: string;
  pwdExpired: boolean;
  registrationDate: string;
  deptName: string;
  roles: string[];
  permissions: string[];
}

