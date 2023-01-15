use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::evaluator::ParametersEnum;

pub struct ComputationHost {
    pub(crate) cache: HashMap<(TypeId, ParametersEnum), Box<dyn Any>>,
}

impl ComputationHost {
    pub(crate) fn get_or_insert_cache_entry<C: 'static, T: Any>(
        &mut self,
        key: ParametersEnum,
        default: impl FnOnce() -> T,
    ) -> &mut T {
        let cache_key = (TypeId::of::<C>(), key);
        self.cache
            .entry(cache_key)
            .or_insert_with(|| Box::new(default()))
            .downcast_mut()
            .expect("mismatched cache entry types")
    }
}
