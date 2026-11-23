use std::time::Duration;

use gpui_kit::{
    App, AppContext, Context, Entity, ParentElement, Render, Styled, Window,
    assets::IconName,
    base::{Disableable, h_flex, input::InputState, v_flex},
    component::{
        ActiveTheme, Sizable, WindowExt,
        button::{Button, ButtonVariants},
        input::Input,
        notification::Notification,
    },
    div,
};
use type_auth::{RegisterRequest, SendEmailRequest, validators::email::validate_email};
use validator::Validate;

use crate::{
    component::base::VisibilityButton,
    ext::SpawnTaskExt,
    libs::client::Client,
    state::router::{NavState, Route},
    util,
    view::auth::auth_shell::AuthShell,
};

const EMAIL_CODE_COOLDOWN_SECS: u8 = 60;

pub struct RegisterView {
    username: Entity<InputState>,
    username_error: Option<String>,
    email: Entity<InputState>,
    email_verify_code: Entity<InputState>,
    email_verify_code_error: Option<String>,
    email_error: Option<String>,
    password: Entity<InputState>,
    confirm_password: Entity<InputState>,
    password_error: Option<String>,
    nickname: Entity<InputState>,
    nickname_error: Option<String>,
    inviter_code: Entity<InputState>,
    inviter_code_error: Option<String>,

    sending_code: bool,
    email_code_cooldown: u8,

    client: Client,
    loading: bool,
}
impl RegisterView {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let username = util::new_input(window, cx, "用户名");
        let email = util::new_input(window, cx, "邮箱");
        let email_verify_code = util::new_input(window, cx, "邮箱验证码");
        let password = util::new_masked_input(window, cx, "密码");
        let confirm_password = util::new_masked_input(window, cx, "确认密码");
        let nickname = util::new_input(window, cx, "昵称");
        let inviter_code = util::new_input(window, cx, "邀请码");

        let client = cx.global::<Client>().clone();

        cx.new(|_| Self {
            username,
            email,
            email_verify_code,
            password,
            confirm_password,
            nickname,
            inviter_code,

            username_error: None,
            email_error: None,
            email_verify_code_error: None,
            password_error: None,
            nickname_error: None,
            inviter_code_error: None,

            sending_code: false,
            email_code_cooldown: 0,

            client,
            loading: false,
        })
    }

    fn validate(&mut self, cx: &mut Context<Self>) -> Option<RegisterRequest> {
        let request = RegisterRequest {
            username: self.username.read(cx).value().into(),
            email: self.email.read(cx).value().into(),
            password: self.password.read(cx).value().into(),
            confirm_password: self.confirm_password.read(cx).value().into(),
            nickname: self.nickname.read(cx).value().into(),
            inviter_code: self.inviter_code.read(cx).value().into(),
            email_verify_code: self.email_verify_code.read(cx).value().into(),
        };
        self.clear_errors();

        match request.validate() {
            Ok(()) => Some(request),
            Err(errors) => {
                self.apply_errors(errors);
                None
            }
        }
    }

    fn clear_errors(&mut self) {
        self.username_error = None;
        self.email_error = None;
        self.email_verify_code_error = None;
        self.password_error = None;
        self.nickname_error = None;
        self.inviter_code_error = None;
    }

    fn apply_errors(&mut self, errors: validator::ValidationErrors) {
        for (field, errors) in errors.field_errors() {
            let message = errors
                .first()
                .and_then(|error| error.message.as_ref())
                .map(ToString::to_string)
                .unwrap_or_else(|| "输入内容不合法".to_string());

            match field.as_ref() {
                "username" => self.username_error = Some(message),
                "email" => self.email_error = Some(message),
                "email_verify_code" => self.email_verify_code_error = Some(message),
                "password" => self.password_error = Some(message),
                "nickname" => self.nickname_error = Some(message),
                "inviter_code" => self.inviter_code_error = Some(message),
                _ => {}
            }
        }
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }

        let Some(req) = self.validate(cx) else {
            cx.notify();
            return;
        };

        self.loading = true;
        cx.notify();

        let client = self.client.clone();
        cx.spawn_task_in(
            window,
            async move |_this, _cx| {
                client.register(req).await?;
                Ok(())
            },
            |this, window, cx, is_err| {
                this.loading = false;
                cx.notify();

                if !is_err {
                    window.push_notification(Notification::info("注册成功"), cx);
                    NavState::go(cx, Route::Login);
                };
            },
        )
        .detach();
    }

    fn send_email_verify_code(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.sending_code || self.email_code_cooldown > 0 {
            return;
        }

        let email = self.email.read(cx).value().trim().to_owned();
        if let Err(e) = validate_email(&email) {
            self.email_error = Some(e.to_string());
            cx.notify();
            return;
        };

        self.email_error = None;
        self.sending_code = true;
        cx.notify();

        let client = self.client.clone();
        cx.spawn_task_in(
            window,
            async move |this, cx| {
                client.send_email_code(SendEmailRequest { email }).await?;
                let _ = this.update(cx, |_this, cx| Self::start_email_code_cooldown(cx));
                Ok(())
            },
            |this, window, cx, is_err| {
                this.sending_code = false;
                cx.notify();

                if !is_err {
                    window.push_notification(Notification::info("发送验证码成功"), cx);
                };
            },
        )
        .detach();
    }

    fn start_email_code_cooldown(cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.email_code_cooldown = EMAIL_CODE_COOLDOWN_SECS;
                cx.notify();
            });
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;

                let Ok(shoulid_contine) = this.update(cx, |this, cx| {
                    this.email_code_cooldown = this.email_code_cooldown.saturating_sub(1);
                    cx.notify();
                    this.email_code_cooldown > 0
                }) else {
                    break;
                };

                if !shoulid_contine {
                    break;
                }
            }
        })
        .detach();
    }
}

impl Render for RegisterView {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let danger = cx.theme().danger;

        let email_code_label = if self.sending_code {
            "发送中…".to_owned()
        } else if self.email_code_cooldown > 0 {
            format!("{} 秒", self.email_code_cooldown)
        } else {
            "获取验证码".to_owned()
        };

        AuthShell::new(
            v_flex()
                .w_full()
                .gap_3()
                .child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .child(
                            Input::new(&self.username)
                                .large()
                                .w_full()
                                .prefix(IconName::User),
                        )
                        .children(
                            self.username_error
                                .clone()
                                .map(|msg| div().w_full().text_sm().text_color(danger).child(msg)),
                        ),
                )
                .child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .child(
                            Input::new(&self.email)
                                .large()
                                .w_full()
                                .prefix(IconName::Mail),
                        )
                        .children(
                            self.email_error
                                .clone()
                                .map(|msg| div().w_full().text_sm().text_color(danger).child(msg)),
                        ),
                )
                .child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .child(
                            h_flex()
                                .w_full()
                                .gap_2()
                                .child(
                                    Input::new(&self.email_verify_code)
                                        .large()
                                        .flex_1()
                                        .prefix(IconName::ShieldCheck),
                                )
                                .child(
                                    Button::new("send-email-code")
                                        .large()
                                        .outline()
                                        .label(email_code_label)
                                        .disabled(self.sending_code || self.email_code_cooldown > 0)
                                        .loading(self.sending_code)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.send_email_verify_code(window, cx);
                                        })),
                                ),
                        )
                        .children(
                            self.email_verify_code_error
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
                                    "register-password-button",
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
                    v_flex().w_full().gap_1().child(
                        Input::new(&self.confirm_password)
                            .large()
                            .w_full()
                            .prefix(IconName::Lock)
                            .suffix(VisibilityButton::from_input(
                                "register-confirm-password-button",
                                &self.confirm_password,
                                cx,
                            )),
                    ),
                )
                .child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .child(
                            Input::new(&self.nickname)
                                .large()
                                .w_full()
                                .prefix(IconName::UserRound),
                        )
                        .children(
                            self.nickname_error
                                .clone()
                                .map(|msg| div().w_full().text_sm().text_color(danger).child(msg)),
                        ),
                )
                .child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .child(
                            Input::new(&self.inviter_code)
                                .large()
                                .w_full()
                                .prefix(IconName::Ticket),
                        )
                        .children(
                            self.inviter_code_error
                                .clone()
                                .map(|msg| div().w_full().text_sm().text_color(danger).child(msg)),
                        ),
                )
                .child(
                    Button::new("register")
                        .primary()
                        .large()
                        .w_full()
                        .label("注册")
                        .loading(self.loading)
                        .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                ),
            h_flex()
                .gap_1()
                .text_sm()
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child("已有账号?"),
                )
                .child(
                    Button::new("register-open-login")
                        .ghost()
                        .small()
                        .label("立即登录")
                        .text_color(cx.theme().primary)
                        .on_click(|_, _, cx| NavState::go(cx, Route::Login)),
                ),
        )
    }
}
