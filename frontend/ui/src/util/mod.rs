use gpui_kit::{App, AppContext, Entity, Window, base::input::InputState};

pub fn new_input(
    window: &mut Window,
    cx: &mut App,
    placeholder: &'static str,
) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
}

pub fn new_masked_input(
    window: &mut Window,
    cx: &mut App,
    placeholder: &'static str,
) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder(placeholder)
            .masked(true)
    })
}
