use crate::Result;
use std::time::Duration;

pub const RETRY_DELAYS: [Duration; 4] = [
    Duration::from_millis(100),
    Duration::from_millis(500),
    Duration::from_millis(2000),
    Duration::from_millis(5000),
];

pub async fn retry_429<F, Fut, T>(mut f: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempts = 0usize;
    loop {
        match f().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if err.is_rate_limited() && attempts < RETRY_DELAYS.len() {
                    let delay = RETRY_DELAYS[attempts];
                    tokio::time::sleep(delay).await;
                    attempts += 1;
                } else {
                    return Err(err);
                }
            }
        }
    }
}
