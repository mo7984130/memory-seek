use common_core::{contextual::ContextualError, type_registry::TypeRegistry};
use common_web::app_router::AppRouter;
#[cfg(feature = "auth")]
use common_web::web_domain::WebDomain;

pub fn init(
    #[allow(unused_variables, reason = "当无domain开启时, registry无用")] registry: &TypeRegistry,
) -> Result<AppRouter, ContextualError> {
    #[allow(unused_mut, reason = "当无domain开启时, router无须mut")]
    let mut router = AppRouter::new();

    #[cfg(feature = "auth")]
    auth::controller::AuthDomain::init("/auth", registry, &mut router)?;

    Ok(router)
}
