use std::{cell::RefCell, collections::VecDeque};

use gpui_kit::{AnyWindowHandle, App, Global, Window, component::WindowExt};
use tracing::{debug, error, warn};

use crate::error::{Notice, Severity};

const MAX_PENDING: usize = 32;

#[derive(Default)]
pub struct ErrorCenter {
    pending: RefCell<VecDeque<Notice>>,
}

impl Global for ErrorCenter {}

impl ErrorCenter {
    pub fn init(cx: &mut App) {
        let this = Self::default();
        cx.set_global(this);
    }

    pub fn report(cx: &mut App, notice: Notice) {
        let target = cx.active_window().or_else(|| cx.windows().first().copied());

        Self::report_to(cx, target, notice);
    }

    pub fn report_to(cx: &mut App, target: Option<AnyWindowHandle>, notice: Notice) {
        Self::log(&notice);

        let mut delivered = false;
        if let Some(target) = target {
            let note = notice.to_note();

            delivered = target
                .update(cx, |_, window, cx| {
                    if window.root::<gpui_kit::base::Root>().flatten().is_some() {
                        window.push_notification(note, cx);
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
        };

        if !delivered {
            warn!("未成功显示错误通知, 通知进入队列");
            Self::push(cx, notice);
        }
    }

    pub fn report_in(window: &mut Window, cx: &mut App, notice: Notice) {
        Self::log(&notice);
        window.push_notification(notice.to_note(), cx);
    }

    fn push(cx: &App, notice: Notice) {
        let center = cx.global::<Self>();
        let mut pending = center.pending.borrow_mut();

        if pending.len() >= MAX_PENDING {
            warn!("错误队列溢出, 抛弃队头");
            pending.pop_front();
        }

        pending.push_back(notice);
    }

    pub fn flush(window: &mut Window, cx: &mut App) {
        let notices: VecDeque<Notice> = {
            let center = cx.global::<Self>();
            let mut pending = center.pending.borrow_mut();
            if pending.is_empty() {
                return;
            }
            std::mem::take(&mut *pending)
        };

        for notice in notices {
            window.push_notification(notice.to_note(), cx);
        }
    }

    fn log(notice: &Notice) {
        match notice.severity {
            Severity::Error => {
                error!(error = ?&notice.source, "{}", notice.message)
            }
            Severity::Warning => {
                warn!(error = ?&notice.source, "{}", notice.message)
            }
            Severity::Info | Severity::Success => {
                debug!(error = ?&notice.source, "{}", notice.message)
            }
        }
    }
}
