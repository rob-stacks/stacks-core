/// HashMap / HashSet type aliases for the Clarity VM execution path.
///
/// With the `bench` feature enabled these resolve to `FxHashMap`/`FxHashSet`
/// (fixed-seed, no OS randomness) so Callgrind instruction counts are fully
/// reproducible across runs.  In production the standard library types are
/// used unchanged.
#[cfg(feature = "bench")]
pub use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
#[cfg(not(feature = "bench"))]
pub use std::collections::{HashMap, HashSet};

/// Construct an empty `HashMap`.  Use this instead of `HashMap::new()` so the
/// call compiles with both the default `RandomState` and the `FxBuildHasher`
/// used when the `bench` feature is on.
#[inline(always)]
pub fn new_map<K, V>() -> HashMap<K, V> {
    HashMap::default()
}

/// Construct an empty `HashSet`.  Same reason as `new_map`.
#[inline(always)]
pub fn new_set<K>() -> HashSet<K> {
    HashSet::default()
}

/// Construct a `HashMap` with a given capacity hint.
#[inline(always)]
pub fn map_with_capacity<K, V>(n: usize) -> HashMap<K, V> {
    #[cfg(feature = "bench")]
    {
        use rustc_hash::FxBuildHasher;
        HashMap::with_capacity_and_hasher(n, FxBuildHasher::default())
    }
    #[cfg(not(feature = "bench"))]
    {
        HashMap::with_capacity(n)
    }
}
