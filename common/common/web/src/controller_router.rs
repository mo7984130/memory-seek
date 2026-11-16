use axum::Router;

pub trait ControllerRouter {
    type State: Send + Sync + 'static;

    /// 返回需要鉴权的路由集合
    fn protected_routes() -> Router<std::sync::Arc<Self::State>> {
        Router::new()
    }
    /// 返回无需鉴权的公开路由集合
    fn public_routes() -> Router<std::sync::Arc<Self::State>> {
        Router::new()
    }
}
