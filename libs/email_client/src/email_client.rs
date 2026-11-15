use std::sync::Arc;

use common_core::contextual::{
    ContextualResult,
    Reason::{self, EmailSendFail},
    ext::ResultExt,
};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct EmailClient {
    inner: Arc<InnerEmailClient>,
}

#[derive(Debug)]
pub struct InnerEmailClient {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
    from_name: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailClientConfig {
    server: String,
    #[serde(default = "default_port")]
    port: u16,
    username: String,
    password: String,
    from_email: String,
    from_name: String,
    // true时, 明文传输
    #[serde(default = "default_plain")]
    plain: bool,
}
fn default_port() -> u16 {
    465
}
fn default_plain() -> bool {
    false
}

impl EmailClient {
    /// 创建邮件客户端；`plain` 为真时明文传输,否则 465 端口走 SSL、其余端口走 STARTTLS；服务器地址无法解析时 panic
    pub fn new(cfg: &EmailClientConfig) -> ContextualResult<Self> {
        let credentials = Credentials::new(cfg.username.to_string(), cfg.password.to_string());

        // 根据端口与传输模式决定加密方式：
        let transport = if cfg.plain {
            // 明文：MailHog 等测试 SMTP
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&cfg.server)
                .port(cfg.port)
                .credentials(credentials)
                .build()
        } else if cfg.port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.server)
                .contextual_error(Reason::BuildFail, "构建邮件客户端失败")?
                .port(cfg.port)
                .credentials(credentials)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.server)
                .contextual_error(Reason::BuildFail, "构建邮件客户端失败")?
                .port(cfg.port)
                .credentials(credentials)
                .build()
        };

        Ok(Self {
            inner: Arc::new(InnerEmailClient {
                transport,
                from_email: cfg.from_email.to_string(),
                from_name: cfg.from_name.to_string(),
            }),
        })
    }

    /// 发送HTML邮件
    pub async fn send_message(
        &self,
        to: &str,
        subject: &str,
        body: String,
    ) -> ContextualResult<()> {
        let email = Message::builder()
            .from(
                format!("{} <{}>", self.inner.from_name, self.inner.from_email)
                    .parse::<Mailbox>()
                    .contextual_error(Reason::EmailSendFail, "发件人地址格式错误")?,
            )
            .to(to
                .parse::<Mailbox>()
                .contextual_warn(EmailSendFail, "目标邮箱格式错误")?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .contextual_error(Reason::EmailSendFail, "构建邮件消息失败")?;

        let send_result = self
            .inner
            .transport
            .send(email)
            .await
            .contextual_error(Reason::EmailSendFail, "邮件服务商发送失败");

        send_result?;

        Ok(())
    }
}
