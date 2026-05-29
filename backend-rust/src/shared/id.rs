use std::{
    sync::atomic::{AtomicI64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static LAST_ID: AtomicI64 = AtomicI64::new(0);

pub fn next_id() -> i64 {
    loop {
        let last = LAST_ID.load(Ordering::Relaxed);
        let now = current_millis();
        let next = if now <= last { last + 1 } else { now };

        if LAST_ID
            .compare_exchange(last, next, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return next;
        }
    }
}

fn current_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_are_monotonic_in_process() {
        let first = next_id();
        let second = next_id();

        assert!(second > first);
    }
}
