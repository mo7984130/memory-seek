use gpui_kit::{
    App, Entity, IntoElement,
    base::input::InputState,
    component::{
        IconName, Sizable,
        button::{Button, ButtonVariants},
    },
};

pub struct VisibilityButton;
impl VisibilityButton {
    pub fn from_input(
        id: &'static str,
        input: &Entity<InputState>,
        cx: &mut App,
    ) -> impl IntoElement {
        let masked = input.read(cx).presentation().is_masked();
        let input = input.clone();

        Button::new(id)
            .text()
            .xsmall()
            .icon(if masked {
                IconName::EyeOff
            } else {
                IconName::Eye
            })
            .tooltip(if masked { "显示" } else { "隐藏" })
            .on_click(move |_, window, cx| {
                input.update(cx, |input, cx| {
                    input.toggle_masked(window, cx);
                })
            })
    }
}
