use common_core::{
    contextual::ContextualError,
    type_registry::{FromRegistry, TypeRegistry},
};
use tracing::{debug, info};

use crate::{app_router::AppRouter, controller_router::ControllerRouter};

pub trait WebDomain: Sized {
    type State: FromRegistry;
    type Controller: ControllerRouter<State = Self::State>;

    fn init(
        prefix: &str,
        registry: &TypeRegistry,
        router: &mut AppRouter,
    ) -> Result<(), ContextualError> {
        debug!("初始化{}", prefix);
        let state = Self::State::new_arc(registry)?;
        router.add_with_prefix::<Self::Controller>(prefix, &state);
        info!("初始化{}成功", prefix);
        Ok(())
    }
}
