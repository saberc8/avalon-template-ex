use chrono::{DateTime, Utc};

pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}
