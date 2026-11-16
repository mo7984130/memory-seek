use std::sync::Arc;

use crate::{contextual::ContextualError, type_registry::TypeRegistry};

pub trait FromRegistry: Sized {
    fn from_registry(registry: &TypeRegistry) -> Result<Self, ContextualError>;

    fn new_arc(registry: &TypeRegistry) -> Result<Arc<Self>, ContextualError> {
        let state = Self::from_registry(registry)?;
        let state = Arc::new(state);
        Ok(state)
    }
}
