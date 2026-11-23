use gpui_kit::{AnyElement, App, AppContext, Entity, Global, IntoElement as _, Render, Window};

use crate::{
    state::session::SessionState,
    view::{
        WallView,
        auth::{LoginView, RegisterView},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Login,
    Register,
    Wall,
}

pub struct History {
    routes: Vec<Route>,
}

impl History {
    pub fn new(root: Route) -> Self {
        Self { routes: vec![root] }
    }

    pub fn current(&self) -> Route {
        *self.routes.last().expect("页面栈应该始终非空")
    }

    pub fn go(&mut self, route: Route) -> bool {
        if route == self.current() {
            return false;
        }

        match self.routes.iter().position(|one| *one == route) {
            Some(i) => self.routes.truncate(i + 1),
            None => self.routes.push(route),
        }
        true
    }

    // pub fn can_back(&self) -> bool {
    //     self.routes.len() > 1
    // }

    //     pub fn back(&mut self) -> bool {
    //         if self.can_back() {
    //             self.routes.pop();
    //             true
    //         } else {
    //             false
    //         }
    //     }
    //
    //     pub fn reset(&mut self, root: Route) {
    //         self.routes.clear();
    //         self.routes.push(root);
    //     }
}
pub struct Nav {
    history: History,
    login: Entity<LoginView>,
    register: Entity<RegisterView>,
    wall: Entity<WallView>,
}
pub struct NavState {
    pub nav: Entity<Nav>,
}
impl Global for NavState {}

impl NavState {
    pub fn init(window: &mut Window, cx: &mut App) {
        let root = if cx.global::<SessionState>().session.read(cx).is_logged_in() {
            Route::Wall
        } else {
            Route::Login
        };

        let login = LoginView::new(window, cx);
        let register = RegisterView::new(window, cx);
        let wall = WallView::new(window, cx);
        let history = History::new(root);
        let nav = cx.new(|_cx| Nav {
            history,
            login,
            register,
            wall,
        });
        cx.set_global(Self { nav });
    }

    pub fn go(cx: &mut App, route: Route) {
        cx.global::<NavState>().nav.clone().update(cx, |nav, cx| {
            nav.history.go(route);
            cx.notify();
        })
    }

    //     pub fn back(cx: &mut App) -> bool {
    //         let nav = cx.global::<NavState>().nav.clone();
    //
    //         nav.update(cx, |nav, cx| {
    //             let changed = nav.history.back();
    //
    //             if changed {
    //                 cx.notify();
    //             }
    //
    //             changed
    //         })
    //     }
    //
    //     pub fn current(cx: &mut App) -> Route {
    //         cx.global::<NavState>().nav.read(cx).history.current()
    //     }
    //
    //     pub fn can_back(cx: &mut App) -> bool {
    //         cx.global::<NavState>().nav.read(cx).history.can_back()
    //     }
    //
    //     pub fn reset(cx: &mut App, root: Route) {
    //         let nav = cx.global::<NavState>().nav.clone();
    //
    //         nav.update(cx, |nav, cx| {
    //             nav.history.reset(root);
    //             cx.notify();
    //         });
    //     }
}

impl Render for Nav {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let content: AnyElement = match self.history.current() {
            Route::Login => self.login.clone().into_any_element(),
            Route::Register => self.register.clone().into_any_element(),
            Route::Wall => self.wall.clone().into_any_element(),
        };

        content
    }
}
