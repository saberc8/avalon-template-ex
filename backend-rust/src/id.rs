use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 简单的本地 ID 生成器。
/// 逻辑与 Go 版 internal/infrastructure/id/ids.go 一致：
/// 使用当前毫秒时间戳，保证在单进程内单调递增。
static LAST_ID: AtomicI64 = AtomicI64::new(0);

/// 生成下一个唯一 ID（i64），用于 sys_* 表的主键。
pub fn next_id() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    loop {
        let last = LAST_ID.load(Ordering::Relaxed);
        let candidate = if now <= last { last + 1 } else { now };
        match LAST_ID.compare_exchange(
            last,
            candidate,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => return candidate,
            Err(_) => continue,
        }
    }
}

