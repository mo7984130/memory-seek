use std::io::{self, IsTerminal, Write};
use tracing::info;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Clone, Copy)]
struct LevelWriter;

struct LevelWriterGuard(bool); // true = stdout

impl Write for LevelWriterGuard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.0 {
            io::stdout().write(buf)
        } else {
            io::stderr().write(buf)
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.0 {
            io::stdout().flush()
        } else {
            io::stderr().flush()
        }
    }
}

impl<'a> MakeWriter<'a> for LevelWriter {
    type Writer = LevelWriterGuard;
    fn make_writer(&'a self) -> Self::Writer {
        LevelWriterGuard(true)
    }
    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        LevelWriterGuard(*meta.level() > tracing::Level::ERROR)
    }
}

pub fn init() {
    let layer = fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_writer(LevelWriter);
    let log_filter = match std::env::var("RUST_LOG") {
        Ok(log_level) => format!("{log_level},gpui=warn,wgpu_hal=warn"),
        Err(_) => "info,gpui=warn,gpui::text_system=error,wgpu_hal=warn".to_string(),
    };
    Registry::default()
        .with(EnvFilter::new(log_filter))
        .with(layer)
        .init();
    info!("日志系统初始化完成");
}
