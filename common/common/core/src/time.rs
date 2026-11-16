/// 统一整个项目的时间使用
use std::time::Duration;

pub type DateTime = chrono::DateTime<chrono::Utc>;
pub fn now() -> DateTime {
    chrono::Utc::now()
}
pub fn now_after(duration: Duration) -> DateTime {
    now() + chrono::Duration::from_std(duration).expect("duration must fit chrono range")
}
