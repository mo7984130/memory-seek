#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppImage {
    Logo,
}

impl AppImage {
    pub fn path(self) -> &'static str {
        match self {
            Self::Logo => "icons/logo.png",
        }
    }

    pub fn img(self) -> gpui_kit::Img {
        gpui_kit::img(self.path())
    }
}
