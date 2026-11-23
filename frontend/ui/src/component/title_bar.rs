use gpui_kit::{
    Context, Entity, ParentElement, Render, Styled, StyledImage, Window,
    base::h_flex,
    component::{
        Sizable,
        button::{Button, ButtonVariants},
    },
};

use crate::{component::AppImage, config::Config};

pub struct TitleBar {
    config: Entity<Config>,
}

impl TitleBar {
    pub fn new(config: Entity<Config>) -> Self {
        Self { config }
    }

    fn cycle_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.config
            .update(cx, |config, cx| config.next_theme(window, cx));
    }
}

impl Render for TitleBar {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let theme = self.config.read(cx).theme();
        gpui_kit::component::TitleBar::new().child(
            h_flex()
                .w_full()
                .pl_1()
                .gap_1()
                .justify_between()
                .items_center()
                .child(
                    AppImage::Logo
                        .img()
                        .h_5()
                        .object_fit(gpui_kit::ObjectFit::Contain),
                )
                .child(
                    Button::new("title-bar-theme")
                        .ghost()
                        .small()
                        .icon(theme.icon())
                        .tooltip(theme.label())
                        .on_click(cx.listener(|this, _, window, cx| this.cycle_theme(window, cx))),
                ),
        )
    }
}
