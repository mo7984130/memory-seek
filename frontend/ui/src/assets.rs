use std::borrow::Cow;

use gpui_kit::{AssetSource, SharedString};

#[derive(rust_embed::Embed)]
#[folder = "./assets"]
struct LocalAssets;

pub struct AppAssets {
    kit: gpui_kit::assets::AllAssets,
}
impl Default for AppAssets {
    fn default() -> Self {
        Self {
            kit: gpui_kit::assets::AllAssets,
        }
    }
}

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        // 先查自己的资源
        if let Some(file) = LocalAssets::get(path) {
            return Ok(Some(file.data));
        }
        // 回退到 GPUI Kit
        self.kit.load(path)
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        let mut paths: Vec<SharedString> = LocalAssets::iter()
            .filter(|p| p.starts_with(path))
            .map(|p| p.into())
            .collect();
        paths.extend(self.kit.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
