use gpui_kit::{App, Window, assets::IconName, component::Theme};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// 浅色。
    Light,
    /// 深色。
    Dark,
    /// 跟随系统。
    #[default]
    System,
}

impl ThemeMode {
    pub fn next(&mut self) -> Self {
        *self = match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::System,
        };
        *self
    }

    pub fn resolve(&self, window: &Window) -> gpui_kit::component::ThemeMode {
        match self {
            Self::Light => gpui_kit::component::ThemeMode::Light,
            Self::Dark => gpui_kit::component::ThemeMode::Dark,
            Self::System => {
                let appearance = window.appearance();
                gpui_kit::component::ThemeMode::from(appearance)
            }
        }
    }

    pub fn apply(&self, window: &mut Window, cx: &mut App) {
        Theme::change(self.resolve(window), Some(window), cx);
    }

    pub fn icon(&self) -> IconName {
        match self {
            Self::System => IconName::Monitor,
            Self::Light => IconName::Sun,
            Self::Dark => IconName::Moon,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::System => "跟随系统",
            Self::Light => "浅色",
            Self::Dark => "深色",
        }
    }
}
