import { api } from "@/lib/api";
import type {
  AvatarResp,
  BasicInfoCommand,
  EmailCommand,
  PasswordCommand,
  PhoneCommand,
  SocialAccountResp
} from "@/types/profile";

const BASE_URL = "/user/profile";

export function uploadAvatar(data: FormData) {
  return api.patch<AvatarResp>(`${BASE_URL}/avatar`, data);
}

export function updateUserBaseInfo(data: BasicInfoCommand) {
  return api.patch<boolean>(`${BASE_URL}/basic/info`, data);
}

export function updateUserPassword(data: PasswordCommand) {
  return api.patch<boolean>(`${BASE_URL}/password`, data);
}

export function updateUserPhone(data: PhoneCommand) {
  return api.patch<boolean>(`${BASE_URL}/phone`, data);
}

export function updateUserEmail(data: EmailCommand) {
  return api.patch<boolean>(`${BASE_URL}/email`, data);
}

export function listUserSocial() {
  return api.get<SocialAccountResp[]>(`${BASE_URL}/social`);
}

export function bindSocialAccount(source: string, data: unknown = {}) {
  return api.post<boolean>(`${BASE_URL}/social/${source}`, data);
}

export function unbindSocialAccount(source: string) {
  return api.delete<boolean>(`${BASE_URL}/social/${source}`);
}
