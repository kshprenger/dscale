use std::{any::Any, sync::Arc};

/// Trait for values that can be sent between simulated processes.
///
/// The optional [`Message::virtual_size`] method controls bandwidth simulation.
/// If bandwidth is unbounded, the default (zero) is fine.
pub trait Message: Any + Send + Sync {
    /// Size in bytes used for bandwidth simulation.
    /// Can exceed the real memory footprint to model heavy payloads.
    /// Defaults to 0 (no bandwidth cost).
    fn virtual_size(&self) -> usize {
        usize::default()
    }
}

/// Reference-counted wrapper around a [`Message`].
///
/// Provides type-safe downcasting to recover the concrete message type.
#[derive(Clone)]
pub struct MessagePtr(pub Arc<dyn Message>);

impl MessagePtr {
    /// Attempts to downcast to `T`. Returns `None` if the type does not match.
    pub fn try_as_type<T: 'static>(&self) -> Option<&T> {
        (&*self.0 as &dyn Any).downcast_ref::<T>()
    }

    /// Returns `true` if the contained message is of type `T`.
    pub fn is<T: 'static>(&self) -> bool {
        self.try_as_type::<T>().is_some()
    }

    /// Downcasts to `T`, panicking if the type does not match.
    pub fn as_type<T: 'static>(&self) -> &T {
        self.try_as_type::<T>().expect("Failed as_type")
    }
}
