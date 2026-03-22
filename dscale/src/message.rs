
use std::{any::Any, sync::Arc};

pub trait Message: Any + Send + Sync {
    fn virtual_size(&self) -> usize {
        usize::default()
    }
}

#[derive(Clone)]
pub struct MessagePtr(pub(crate) Arc<dyn Message>);

impl MessagePtr {
    pub fn try_as_type<T: 'static>(&self) -> Option<&T> {
        (&*self.0 as &dyn Any).downcast_ref::<T>()
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.try_as_type::<T>().is_some()
    }

    pub fn as_type<T: 'static>(&self) -> &T {
        self.try_as_type::<T>().expect("Failed as_type")
    }
}
