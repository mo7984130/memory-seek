use std::sync::Arc;

use axum::Router;

use crate::controller_router::ControllerRouter;

pub struct AppRouter {
    pub public: Router,
    pub protected: Router,
}
impl Default for AppRouter {
    fn default() -> Self {
        Self::new()
    }
}
impl AppRouter {
    pub fn new() -> Self {
        Self {
            public: Router::new(),
            protected: Router::new(),
        }
    }

    /// 合并入公开路由
    #[allow(unused)]
    pub fn add_public(&mut self, router: Router) {
        self.public = std::mem::take(&mut self.public).merge(router);
    }

    #[allow(unused)]
    pub fn add_protected(&mut self, router: Router) {
        self.protected = std::mem::take(&mut self.protected).merge(router);
    }

    pub fn add<R: ControllerRouter>(&mut self, state: &Arc<R::State>) {
        self.add_public(R::public_routes().with_state(Arc::clone(state)));
        self.add_protected(R::protected_routes().with_state(Arc::clone(state)));
    }

    pub fn add_with_prefix<R: ControllerRouter>(&mut self, prefix: &str, state: &Arc<R::State>) {
        self.add_public(
            Router::new().nest(prefix, R::public_routes().with_state(Arc::clone(state))),
        );
        self.add_protected(
            Router::new().nest(prefix, R::protected_routes().with_state(Arc::clone(state))),
        );
    }
}
