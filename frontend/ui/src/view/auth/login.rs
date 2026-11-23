use gpui_kit::{
    App, AppContext, Context, Entity, ParentElement, Render, Styled, Window,
    assets::IconName,
    base::{h_flex, input::InputState, v_flex},
    component::{
        ActiveTheme, Sizable, WindowExt,
        button::{Button, ButtonVariants},
        input::Input,
        notification::Notification,
    },
    div,
};
use type_auth::{LoginRequest, validators};

use crate::{
    component::base::VisibilityButton,
    ext::SpawnTaskExt,
    libs::client::Client,
    state::{
        router::{NavState, Route},
        session::{Session, SessionState},
    },
    util,
    view::auth::auth_shell::AuthShell,
};

pub struct LoginView {
    account: Entity<InputState>,
    account_error: Option<String>,
    password: Entity<InputState>,
    password_error: Option<String>,

    client: Client,
    session: Entity<Session>,
    loading: bool,
}
impl LoginView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let session = cx.global::<SessionState>().session.clone();
        let client = cx.global::<Client>().clone();

        let account = util::new_input(window, cx, "用户名 / 邮箱");
        let password = util::new_masked_input(window, cx, "密码");

        cx.new(|_cx| Self {
            account,
            account_error: None,
            password,
            password_error: None,

            client,
            session,
            loading: false,
        })
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }

        let account = self.account.read(cx).value().to_string();
        let password = self.password.read(cx).value().to_string();
        let mut validate_fail = false;
        match validators::account::validate_account(&account) {
            Ok(_) => self.account_error = None,
            Err(e) => {
                self.account_error = Some(e.to_string());
                validate_fail = true;
            }
        }
        match validators::password::validate_password(&password) {
            Ok(_) => self.password_error = None,
            Err(e) => {
                self.password_error = Some(e.to_string());
                validate_fail = true;
            }
        }

        if validate_fail {
            self.loading = false;
            cx.notify();
            return;
        }

        self.loading = true;
        cx.notify();

        let client = self.client.clone();
        let session = self.session.clone();
        cx.spawn_task_in(
            window,
            async move |_this, cx| {
                let resp = client.login(LoginRequest { account, password }).await?;
                session.update(cx, |session, cx| session.login(cx, resp));
                let _ = cx.update(|_window, cx| NavState::go(cx, Route::Wall));
                Ok(())
            },
            |this, window, cx, is_err| {
                this.loading = false;
                cx.notify();

                if !is_err {
                    window.push_notification(Notification::info("登录成功"), cx);
                }
            },
        )
        .detach();
    }
}

impl Render for LoginView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let theme = cx.theme();
        let (danger, muted_foreground, primary) =
            (theme.danger, theme.muted_foreground, theme.primary);

        AuthShell::new(
            v_flex()
                .w_full()
                .gap_3()
                .child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .child(
                            Input::new(&self.account)
                                .large()
                                .w_full()
                                .prefix(IconName::User),
                        )
                        .children(
                            self.account_error
                                .clone()
                                .map(|msg| div().w_full().text_sm().text_color(danger).child(msg)),
                        ),
                )
                .child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .child(
                            Input::new(&self.password)
                                .large()
                                .w_full()
                                .prefix(IconName::Lock)
                                .suffix(VisibilityButton::from_input(
                                    "login-password-button",
                                    &self.password,
                                    cx,
                                )),
                        )
                        .children(
                            self.password_error
                                .clone()
                                .map(|msg| div().w_full().text_sm().text_color(danger).child(msg)),
                        ),
                )
                .child(
                    Button::new("login")
                        .primary()
                        .large()
                        .w_full()
                        .label("登录")
                        .loading(self.loading)
                        .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                ),
            h_flex()
                .gap_1()
                .text_sm()
                .child(div().text_color(muted_foreground).child("还没有账号?"))
                .child(
                    Button::new("login-open-register")
                        .ghost()
                        .small()
                        .label("立即注册")
                        .text_color(primary)
                        .on_click(|_, _, cx| NavState::go(cx, Route::Register)),
                ),
        )
    }
}
