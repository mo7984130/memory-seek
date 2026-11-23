use crate::config::Config;
use crate::error::Result;
use crate::state::router::{Nav, NavState};
use gpui_kit::div;
use gpui_kit::{
    App, AppContext, Context, Entity, ParentElement, Render, Styled, Window, base::v_flex,
};

use crate::component::TitleBar;

pub struct Root {
    title_bar: Entity<TitleBar>,
    nav: Entity<Nav>,
    // _nav_subscription: Subscription,
}

impl Root {
    pub fn new(window: &mut Window, cx: &mut App) -> Result<Self> {
        let config = Config::new(window, cx)?;
        let title_bar = cx.new(|_| TitleBar::new(config));
        let nav = cx.global::<NavState>().nav.clone();
        // let _nav_subscription = cx.observe(&nav, |this, cx| {
        //     cx.notify();
        // });

        Ok(Self {
            title_bar,
            nav,
            // _nav_subscription,
        })
    }
}

impl Render for Root {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        v_flex()
            .size_full()
            .child(self.title_bar.clone())
            .child(div().flex_1().min_h_0().child(self.nav.clone()))
    }
}
