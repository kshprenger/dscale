//! Global key-value storage for simulation state and metrics.
//!
//! This module provides a global key-value store that can hold values of any type.
//! It's useful for sharing state, metrics, or configuration between processes or
//! for passing data back to the host application after simulation completion.
//!
//! The storage is global and persists throughout the simulation lifetime.
//! All functions operate on a per-simulation basis and are reset when a new
//! simulation starts.

use std::any::Any;

use dashmap::DashMap;
use rustc_hash::FxBuildHasher;

type Map<K, V> = DashMap<K, V, FxBuildHasher>;

static KV: std::sync::LazyLock<Map<String, Box<dyn Any + Send + Sync>>> =
    std::sync::LazyLock::new(|| DashMap::with_hasher(FxBuildHasher));

/// Stores a value of any type in the global key-value store.
///
/// This function allows you to store values that can be retrieved later using
/// [`get`] or modified using [`modify`]. The value can be of any type that
/// implements `'static`.
///
/// # Type Parameters
///
/// * `T` - The type of the value to store. Must be `'static`.
///
/// # Arguments
///
/// * `key` - A string key to identify the stored value
/// * `value` - The value to store
///
/// # Panics
///
/// This function does not panic under normal circumstances.
pub fn set<T: 'static + Send + Sync>(key: &str, value: T) {
    KV.insert(key.to_string(), Box::new(value));
}

/// Retrieves a cloned copy of a value from the global key-value store.
///
/// This function returns a clone of the stored value. The original value
/// remains in the store and can be retrieved again.
///
/// # Type Parameters
///
/// * `T` - The expected type of the stored value. Must be `'static` and `Clone`.
///
/// # Arguments
///
/// * `key` - The string key identifying the value to retrieve
///
/// # Returns
///
/// A cloned copy of the stored value.
///
/// # Panics
///
/// This function panics if:
/// * The key does not exist in the store
/// * The stored value cannot be downcast to type `T`
pub fn get<T: 'static + Clone + Send + Sync>(key: &str) -> T {
    KV.get(key)
        .expect("No key")
        .downcast_ref::<T>()
        .cloned()
        .expect("Wrong type cast")
}

/// Modifies a value in the global key-value store in-place.
///
/// This function allows you to modify a stored value without retrieving and
/// re-storing it. The modification is performed through a closure that receives
/// a mutable reference to the stored value.
///
/// # Type Parameters
///
/// * `T` - The expected type of the stored value. Must be `'static`.
///
/// # Arguments
///
/// * `key` - The string key identifying the value to modify
/// * `f` - A closure that receives a mutable reference to the stored value
///
/// # Panics
///
/// This function panics if:
/// * The key does not exist in the store
/// * The stored value cannot be downcast to type `T`
pub fn modify<T: 'static + Send + Sync>(key: &str, f: impl FnOnce(&mut T)) {
    let mut entry = KV.get_mut(key).expect("No key");
    f(entry.downcast_mut::<T>().expect("Wrong type cast"));
}

pub(crate) fn drop() {
    KV.clear();
}
