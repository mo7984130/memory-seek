use common_core::util::DeferredFile;
use gpui_kit::{App, AppContext, Context, Entity, Global};
use serde::{Deserialize, Serialize};
use type_auth::{LoginResponse, UserInfo};

use crate::{config::dir, error::Error, ext::SpawnTaskExt, libs::client::Client};

#[derive(Debug, Default, Serialize, Deserialize)]
enum AuthStatus {
    #[default]
    LoggedOut,
    LoggedIn(LoggedInInfo),
}
impl AuthStatus {
    pub fn is_logged_in(&self) -> bool {
        matches!(self, Self::LoggedIn { .. })
    }

    pub fn set_user(&mut self, user: UserInfo) {
        let info = LoggedInInfo { user };

        *self = Self::LoggedIn(info);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LoggedInInfo {
    user: UserInfo,
}

pub struct Session {
    client: Client,

    status: DeferredFile<AuthStatus>,
}
impl Session {
    fn new(cx: &mut App) -> Result<Entity<Self>, Error> {
        let client = cx.global::<Client>().clone();
        let status = DeferredFile::load(dir::config_dir().join("info.json"))?;
        let this = cx.new(|_cx| Self { client, status });
        this.update(cx, |this, cx| this.refresh(cx));
        Ok(this)
    }

    // pub fn user(&self) -> Option<UserInfo> {
    //     self.status.with(|status| match status {
    //         AuthStatus::LoggedOut => None,
    //         AuthStatus::LoggedIn(info) => Some(info.user.clone()),
    //     })
    // }

    pub fn is_logged_in(&self) -> bool {
        self.status.with(AuthStatus::is_logged_in)
    }

    pub fn login(&mut self, cx: &mut Context<Self>, resp: LoginResponse) {
        self.status.update(|status| status.set_user(resp.user));

        cx.notify();
    }

    fn apply_user(&mut self, user: UserInfo) {
        self.status.update(|status| status.set_user(user));
    }

    pub fn refresh(&self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        cx.spawn_task(async move |this, cx| {
            let info = client.get_self_info().await?;
            let _ = this.update(cx, |this, cx| {
                this.apply_user(info);
                cx.notify();
            });
            Ok(())
        })
        .detach();
    }
}

pub struct SessionState {
    pub session: Entity<Session>,
}
impl Global for SessionState {}
impl SessionState {
    pub fn load(cx: &mut App) -> Result<(), Error> {
        let session = Session::new(cx)?;
        let this = Self { session };
        cx.set_global(this);
        Ok(())
    }
}
