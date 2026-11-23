use crate::{
    error::{Notice, Result},
    libs::error_enter::ErrorCenter,
};
use gpui_kit::{AsyncApp, AsyncWindowContext, Context, Task, WeakEntity, Window};

pub trait SpawnTaskExt<T: 'static> {
    fn spawn_task<F>(&self, task: F) -> Task<()>
    where
        F: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> Result<()> + 'static;

    // /// 窗口无关任务：`finish` 自己决定写状态、内联展示或交给 ErrorCenter
    // #[must_use]
    // fn spawn_task_with_finish<F, C>(&self, task: F, finish: C) -> Task<()>
    // where
    //     F: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> Result<()> + 'static,
    //     C: FnOnce(&mut T, &mut Context<T>, bool) + 'static;

    /// 窗口绑定任务：`finish` 里可以直接 toast
    fn spawn_task_in<F, C>(&self, window: &Window, task: F, finish: C) -> Task<()>
    where
        F: AsyncFnOnce(WeakEntity<T>, &mut AsyncWindowContext) -> Result<()> + 'static,
        C: FnOnce(&mut T, &mut Window, &mut Context<T>, bool) + 'static;
}

impl<T: 'static> SpawnTaskExt<T> for Context<'_, T> {
    fn spawn_task<F>(&self, task: F) -> Task<()>
    where
        F: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> Result<()> + 'static,
    {
        self.spawn(async move |this, cx| {
            let result = task(this.clone(), cx).await;
            if let Err(e) = result {
                cx.update(|cx| ErrorCenter::report(cx, Notice::error(e)));
            }
        })
    }

    // fn spawn_task_with_finish<F, C>(&self, task: F, finish: C) -> Task<()>
    // where
    //     F: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> Result<()> + 'static,
    //     C: FnOnce(&mut T, &mut Context<T>, bool) + 'static,
    // {
    //     self.spawn(async move |this, cx| {
    //         let result = task(this.clone(), cx).await;
    //         let mut is_err = false;
    //         if let Err(e) = result {
    //             cx.update(|cx| ErrorCenter::report(cx, Notice::error(e)));
    //             is_err = true
    //         }
    //         if this
    //             .update(cx, |this, cx| finish(this, cx, is_err))
    //             .is_err()
    //         {
    //             tracing::debug!("异步任务完成时实体已销毁，跳过收尾");
    //         }
    //     })
    // }

    fn spawn_task_in<F, C>(&self, window: &Window, task: F, finish: C) -> Task<()>
    where
        F: AsyncFnOnce(WeakEntity<T>, &mut AsyncWindowContext) -> Result<()> + 'static,
        C: FnOnce(&mut T, &mut Window, &mut Context<T>, bool) + 'static,
    {
        self.spawn_in(window, async move |this, cx| {
            let result = task(this.clone(), cx).await;
            let mut is_err = false;
            if let Err(e) = result {
                if cx
                    .update(|window, cx| ErrorCenter::report_in(window, cx, Notice::error(e)))
                    .is_err()
                {
                    tracing::debug!("异步任务完成时实体或窗口已销毁，跳过收尾");
                }
                is_err = true
            }
            if this
                .update_in(cx, |this, window, cx| finish(this, window, cx, is_err))
                .is_err()
            {
                tracing::debug!("异步任务完成时实体或窗口已销毁，跳过收尾");
            }
        })
    }
}
