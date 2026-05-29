export interface AvatarResp {
  avatar: string;
}

export interface SocialAccountResp {
  source: string;
}

export interface BasicInfoCommand {
  nickname: string;
  gender: number;
}

export interface PasswordCommand {
  oldPassword: string;
  newPassword: string;
}

export interface PhoneCommand {
  phone: string;
  captcha: string;
  oldPassword: string;
}

export interface EmailCommand {
  email: string;
  captcha: string;
  oldPassword: string;
}
