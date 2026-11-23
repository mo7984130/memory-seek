use std::fmt::Debug;

use common_core::util::DeferredFileError;
use gpui_kit::{
    App, Window,
    component::{
        WindowExt,
        notification::{Notification, NotificationType},
    },
};
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("启动时错误: {0}")]
    Start(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("{0}")]
    Api(#[from] api::error::ApiError),
    #[error("保存文件错误: {0}")]
    Storage(#[from] DeferredFileError),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Info,
    Success,
    Warning,
    Error,
}
impl From<Severity> for NotificationType {
    fn from(s: Severity) -> Self {
        match s {
            Severity::Info => NotificationType::Info,
            Severity::Success => NotificationType::Success,
            Severity::Warning => NotificationType::Warning,
            Severity::Error => NotificationType::Error,
        }
    }
}
pub struct Notice {
    pub severity: Severity,
    pub title: Option<String>,
    pub message: String,
    pub source: Box<dyn std::error::Error + Send + Sync>,
}
impl Notice {
    pub fn with_title(self, title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..self
        }
    }

    pub fn to_note(&self) -> Notification {
        let mut note = Notification::new()
            .with_type(self.severity.into())
            .message(self.message.clone());
        if let Some(title) = self.title.clone() {
            note = note.title(title)
        };
        note
    }

    pub fn error<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self {
            severity: Severity::Error,
            title: None,
            message: e.to_string(),
            source: Box::new(e),
        }
    }
}

pub trait NotifyExt {
    fn notify(self, msg: impl AsRef<str>, window: &mut Window, cx: &mut App);
}
impl<T, E: Into<Error>> NotifyExt for std::result::Result<T, E> {
    fn notify(self, msg: impl AsRef<str>, window: &mut Window, cx: &mut App) {
        if let Err(e) = self {
            let e = e.into();
            error!("{e}");
            window.push_notification((NotificationType::Error, msg.as_ref()), cx);
        }
    }
}
