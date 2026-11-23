use gpui_kit::{App, AppContext, Entity, Render, Window, div};

pub struct WallView;
impl WallView {
    pub fn new(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|_cx| WallView)
    }
}
impl Render for WallView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        _cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        div()
    }
}
