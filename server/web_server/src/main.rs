use std::{net::SocketAddr, process};

use axum::Router;
use common_core::contextual::ContextualError;
use common_web::app_router::AppRouter;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

use crate::config::WebServerConfig;

mod config;
mod domains;
mod middlewares;

#[tokio::main]
async fn main() {
    match start_server().await {
        Ok(_) => {
            info!("服务正常退出");
            process::exit(0)
        }
        Err(e) => {
            error!("服务错误: {:?}", e);
            process::exit(1)
        }
    }
}

pub async fn start_server() -> Result<(), ContextualError> {
    let init_bundle = server::init(None).await?;
    let router = domains::init(&init_bundle.registry)?;

    let cfg = WebServerConfig::load(&init_bundle.config_source)?;

    debug!("尝试监听 {}", cfg.server.server_addr());
    let listener = TcpListener::bind(cfg.server.server_addr()).await?;
    info!("监听 {}", cfg.server.server_addr());

    let mut router = router;
    // 构建router
    router.add_public(
        Router::new()
            .route("/health", axum::routing::get(|| async { "ok" }))
            .route(
                "/how/are/you",
                axum::routing::get(|| async { "I am fine, thank you" }),
            ),
    );
    let AppRouter { public, protected } = router;
    let protected = protected.layer(axum::middleware::from_fn_with_state(
        init_bundle.state,
        middlewares::auth_middleware,
    ));

    let router = public
        .merge(protected)
        .layer(axum::middleware::from_fn(
            middlewares::tracing_span_middleware,
        ))
        .layer(axum::middleware::from_fn(middlewares::trace_id_middleware))
        .layer(axum::middleware::from_fn(middlewares::client_ip_middleware));

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_trigger())
    .await?;

    tracing::info!("服务已完全关闭");
    Ok(())
}

async fn shutdown_trigger() {
    let sigint = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(unix)]
    tokio::select! {
        _ = sigint => {},
        _ = sigterm => {},
    }

    #[cfg(not(unix))]
    sigint.await;

    tracing::info!("收到关闭信号，开始排空在途请求");
}
