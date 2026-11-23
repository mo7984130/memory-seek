use std::borrow::Cow;

use gpui_kit::{
    AppContext, WindowOptions,
    component::{Theme, TitleBar},
};

use crate::{
    assets::AppAssets,
    libs::{client::Client, error_enter::ErrorCenter},
    root::Root,
    state::{router::NavState, session::SessionState},
};

mod assets;
mod component;
mod config;
pub mod error;
mod ext;
mod libs;
mod root;
mod state;
mod util;
mod view;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    libs::log::init();

    gpui_kit::application()
        .with_assets(AppAssets::default())
        .run(move |cx| {
            gpui_kit::init(cx);

            // 添加字体
            cx.text_system()
                .add_fonts(vec![
                    Cow::Borrowed(
                        include_bytes!("../assets/fonts/MapleMono-NF-CN-Regular.ttf").as_slice(),
                    ),
                    Cow::Borrowed(
                        include_bytes!("../assets/fonts/MapleMono-NF-CN-Bold.ttf").as_slice(),
                    ),
                ])
                .unwrap();
            Theme::update(cx, |theme| {
                theme.font_family = "Maple Mono NF CN".into();
            });

            // libs 初始化
            Client::init(cx).unwrap();
            ErrorCenter::init(cx);
            SessionState::load(cx).unwrap();

            let options = WindowOptions {
                ..TitleBar::window_options()
            };
            gpui_kit::open_window(options, cx, move |window, cx| {
                ErrorCenter::flush(window, cx);
                NavState::init(window, cx);

                let root = Root::new(window, cx).unwrap();
                cx.new(|_cx| root)
            })
            .unwrap();
        });

    Ok(())
}
