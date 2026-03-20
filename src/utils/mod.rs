pub mod time {
    use chrono::{DateTime, Utc};

    pub fn current_timestamp() -> DateTime<Utc> {
        Utc::now()
    }

    pub fn timestamp_millis() -> i64 {
        Utc::now().timestamp_millis()
    }
}