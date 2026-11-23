use crate::{
    config::theme_mode::ThemeMode,
    error::{NotifyExt, Result},
};
use common_core::util::DeferredFile;
use gpui_kit::{App, AppContext, Entity, Window};
use serde::{Deserialize, Serialize};

pub mod dir;
pub mod theme_mode;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    theme: ThemeMode,
}

pub struct Config {
    store: DeferredFile<AppConfig>,
}
impl Config {
    pub fn new(window: &mut Window, cx: &mut App) -> Result<Entity<Self>> {
        let config = Self {
            store: DeferredFile::load(dir::config_dir().join("config.json"))?,
        };
        config.theme().apply(window, cx);
        Ok(cx.new(|_cx| config))
    }

    pub fn theme(&self) -> ThemeMode {
        self.store.with(|cfg| -> ThemeMode { cfg.theme })
    }

    pub fn next_theme(&mut self, window: &mut Window, cx: &mut App) -> ThemeMode {
        let theme = self.store.update(|cfg| cfg.theme.next());
        theme.apply(window, cx);
        self.store.save().notify("保存主题失败", window, cx);
        theme
    }
}
