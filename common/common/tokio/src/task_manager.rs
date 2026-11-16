use std::sync::Mutex;
use std::time::Duration;
use std::{borrow::Cow, sync::Arc};

use chrono::{DateTime, Local};
use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Copy)]
pub enum TaskSchedule {
    Daily { hour: u32, minute: u32 },
}

#[derive(Clone)]
pub struct TaskManager {
    inner: Arc<Inner>,
}
impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Inner {
    tasks: DashMap<Cow<'static, str>, TaskTracker>,
    cancel_token: CancellationToken,
    waiting: Mutex<Option<Cow<'static, str>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self::with_token(CancellationToken::new())
    }

    pub fn with_token(cancel_token: CancellationToken) -> Self {
        Self {
            inner: Arc::new(Inner {
                cancel_token,
                tasks: DashMap::new(),
                waiting: Mutex::new(None),
            }),
        }
    }

    pub fn spawn<F, Fut>(
        &self,
        name: impl Into<Cow<'static, str>>,
        task: F,
    ) -> Option<CancellationToken>
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if self.inner.cancel_token.is_cancelled() {
            return None;
        }

        let name = name.into();
        let child_token = self.inner.cancel_token.child_token();

        let task_token = child_token.clone();
        let task_name = name.clone();

        debug!("启动任务: '{}'", name);
        self.inner.tasks.entry(name).or_default().spawn(async move {
            task(task_token).await;
            debug!("任务'{}'已结束", task_name);
        });

        Some(child_token)
    }

    pub fn spawn_schedule<F, Fut>(
        &self,
        name: impl Into<Cow<'static, str>>,
        schedule: TaskSchedule,
        task: F,
    ) -> Option<CancellationToken>
    where
        F: Fn(CancellationToken) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        if self.inner.cancel_token.is_cancelled() {
            return None;
        }

        let name = name.into();
        let child_token = self.inner.cancel_token.child_token();

        let task_name = name.clone();
        let task_token = child_token.clone();

        debug!("启动定时任务: {name}");
        self.inner.tasks.entry(name).or_default().spawn(async move {
            loop {
                let delay = delay_until_next(schedule);
                tokio::select! {
                    biased;
                    _ = tokio::time::sleep(delay) => {}
                    _ = task_token.cancelled() => {
                        debug!("定时任务取消: {task_name}");
                        return;
                    }
                }
                task(task_token.clone()).await;
            }
        });

        Some(child_token)
    }

    pub fn spawn_interval<F, Fut>(
        &self,
        name: impl Into<Cow<'static, str>>,
        interval: Duration,
        task: F,
    ) -> Option<CancellationToken>
    where
        F: Fn(CancellationToken) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let name = name.into();
        let child_token = self.inner.cancel_token.child_token();

        let task_name = name.clone();
        let task_token = child_token.clone();

        debug!("启动间隔任务: {name}");
        self.inner.tasks.entry(name).or_default().spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                tokio::select! {
                    biased;
                    _ = interval_timer.tick() => {}
                    _ = task_token.cancelled() => {
                        debug!("间隔任务 '{}' 被取消", task_name);
                        return;
                    }
                }
                task(task_token.clone()).await;
            }
        });

        Some(child_token)
    }

    pub fn running_names(&self) -> Vec<Cow<'static, str>> {
        let mut names: Vec<Cow<'_, str>> = self
            .inner
            .tasks
            .iter()
            .filter(|e| !e.value().is_empty()) // 只报告还有任务的组
            .map(|e| e.key().clone())
            .collect();
        if let Some(waiting) = self.inner.waiting.lock().unwrap().as_ref()
            && !names.iter().any(|n| n == waiting)
        {
            names.push(waiting.clone());
        }
        names
    }

    async fn wait_all(&self) {
        loop {
            // 收集当前所有组名
            let names: Vec<Cow<'static, str>> =
                self.inner.tasks.iter().map(|e| e.key().clone()).collect();
            if names.is_empty() {
                break;
            }

            for name in &names {
                // 先 close，允许 wait 完成
                if let Some(tracker) = self.inner.tasks.get(name) {
                    tracker.close();
                }
            }

            // 等待这一批全部完成
            for name in &names {
                if let Some(tracker) = self.inner.tasks.get(name) {
                    *self.inner.waiting.lock().unwrap() = Some(name.clone());
                    tracker.wait().await;
                }
            }

            // 清理已完成的任务组
            for name in &names {
                self.inner.tasks.remove(name);
            }
            *self.inner.waiting.lock().unwrap() = None;

            // 若期间有新组创建，外层 loop 会再处理一轮
        }
    }

    pub fn shutdown(&self) {
        self.inner.cancel_token.cancel();
    }

    pub async fn wait_for_all(&self, timeout: Option<Duration>) -> bool {
        match timeout {
            None => {
                self.wait_all().await;
                info!("所有后台任务已完成");
                true
            }
            Some(timeout) => {
                match tokio::time::timeout(timeout, async { self.wait_all().await }).await {
                    Ok(_) => {
                        info!("所有后台任务已完成");
                        true
                    }
                    Err(_) => {
                        error!(
                            "等待后台任务超时 ({}s)，仍在运行的任务: {:?}，强制退出",
                            timeout.as_secs(),
                            self.running_names()
                        );
                        false
                    }
                }
            }
        }
    }
}

fn delay_until_next(schedule: TaskSchedule) -> Duration {
    let now = Local::now();
    let next = match schedule {
        TaskSchedule::Daily { hour, minute } => {
            // 提前校验，避免 and_hms_opt 返回 None 时才 panic，错误信息更明确
            assert!(
                hour < 24 && minute < 60,
                "无效的调度时刻: {hour:02}:{minute:02}"
            );

            // 计算"今天"的候选时刻；DST 跳过的不存在时刻返回 None
            let candidate = resolve_local(now.date_naive(), hour, minute);

            match candidate {
                Some(c) if c > now => c,
                // 今天已过，或今天该时刻在 DST 中被跳过 → 用明天
                _ => {
                    let tomorrow = now.date_naive() + chrono::Duration::days(1);
                    resolve_local(tomorrow, hour, minute).unwrap_or_else(|| {
                        // 极端 DST 情况：连续两天都不存在该时刻（几乎不可能，但给出明确错误）
                        panic!("无法解析调度时刻 {hour:02}:{minute:02}（可能处于夏令时跳变区间）")
                    })
                }
            }
        }
    };

    // now 与 next 都是同一时区的本地时间，差值直接相减即可（chrono 会正确处理 DST 偏移）
    let diff = next.signed_duration_since(now);
    match diff.to_std() {
        Ok(d) => d,
        Err(e) => {
            error!("解析日期错误: {e}");
            Duration::from_hours(24)
        }
    }
}

/// 把一个"朴素本地日期时间"解析为带时区的绝对时刻。
/// DST 跳变导致该时刻不存在时返回 None；若存在两个（回拨重叠），取最早的那个。
fn resolve_local(date: chrono::NaiveDate, hour: u32, minute: u32) -> Option<DateTime<Local>> {
    let naive = date.and_hms_opt(hour, minute, 0)?;
    naive.and_local_timezone(Local).earliest()
}
