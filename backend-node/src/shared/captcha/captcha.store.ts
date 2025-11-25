/**
 * 简单验证码存储（内存版）。
 *
 * 说明：
 * - 仅在单个 Node 进程内生效，重启后数据会丢失；
 * - 用于替代 Java 版 Redis 中的验证码存储，实现基本的验证码校验流程。
 */

type CaptchaRecord = {
  code: string;
  expireTime: number;
};

const store = new Map<string, CaptchaRecord>();

/** 写入验证码记录。 */
export function setCaptcha(
  uuid: string,
  code: string,
  expireTime: number,
): void {
  if (!uuid || !code) return;
  store.set(uuid, { code, expireTime });
}

/** 读取并删除验证码记录。 */
export function takeCaptcha(uuid: string): CaptchaRecord | null {
  if (!uuid) return null;
  const rec = store.get(uuid);
  if (!rec) return null;
  store.delete(uuid);
  if (rec.expireTime <= Date.now()) {
    return null;
  }
  return rec;
}

