/**
 * 简单的本地 ID 生成器。
 * 逻辑与 Go 版 internal/infrastructure/id/ids.go 一致：
 * 使用当前毫秒时间戳，保证在单进程内单调递增。
 */
let lastId = BigInt(0);

/**
 * 生成下一个唯一 ID（bigint），用于 sys_* 表的主键。
 */
export function nextId(): bigint {
  const now = BigInt(Date.now());
  if (now <= lastId) {
    lastId += BigInt(1);
  } else {
    lastId = now;
  }
  return lastId;
}
