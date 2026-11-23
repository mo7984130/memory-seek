use gpui_kit::{
    AnyElement, IntoElement, ParentElement, RenderOnce, Styled,
    base::v_flex,
    component::{ActiveTheme, scroll::ScrollableElement},
    div, px,
};

#[derive(IntoElement)]
pub struct AuthShell {
    content: AnyElement,
    footer: AnyElement,
}

impl AuthShell {
    pub fn new(content: impl IntoElement, footer: impl IntoElement) -> Self {
        Self {
            content: content.into_any_element(),
            footer: footer.into_any_element(),
        }
    }
}

impl RenderOnce for AuthShell {
    fn render(self, _window: &mut gpui_kit::Window, cx: &mut gpui_kit::App) -> impl IntoElement {
        let theme = cx.theme();

        v_flex().overflow_y_scrollbar().child(
            v_flex()
                .w_full()
                .min_h_full()
                .items_center()
                .justify_center()
                .px_4()
                .py_8()
                .child(
                    v_flex()
                        .w_full()
                        .max_w(px(360.))
                        .gap_8()
                        .rounded(px(12.))
                        .border_1()
                        .border_color(theme.border)
                        .px_6()
                        .py_8()
                        // LOGO
                        .child(
                            v_flex()
                                .items_center()
                                .gap_2()
                                .child(div().text_3xl().child("寻忆"))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.muted_foreground)
                                        .child("让时光驻留,让记忆重现。"),
                                ),
                        )
                        .child(self.content)
                        .child(self.footer),
                ),
        )
    }
}
