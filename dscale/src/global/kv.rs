use std::any::Any;

use dashmap::DashMap;
use rustc_hash::FxBuildHasher;

type Map<K, V> = DashMap<K, V, FxBuildHasher>;

static KV: std::sync::LazyLock<Map<String, Box<dyn Any + Send + Sync>>> =
    std::sync::LazyLock::new(|| DashMap::with_hasher(FxBuildHasher));

/// Stores a value under the given key, replacing any previous value.
pub fn set<T: 'static + Send + Sync>(key: &str, value: T) {
    KV.insert(key.to_string(), Box::new(value));
}

/// Retrieves a clone of the value stored under the given key.
/// Panics if the key is missing or the type does not match.
pub fn get<T: 'static + Clone + Send + Sync>(key: &str) -> T {
    KV.get(key)
        .expect("No key")
        .downcast_ref::<T>()
        .cloned()
        .expect("Wrong type cast")
}

/// Mutates the value stored under the given key in place.
/// Panics if the key is missing or the type does not match.
pub fn modify<T: 'static + Send + Sync>(key: &str, f: impl FnOnce(&mut T)) {
    let mut entry = KV.get_mut(key).expect("No key");
    f(entry.downcast_mut::<T>().expect("Wrong type cast"));
}

pub(crate) fn reset() {
    KV.clear();
}
