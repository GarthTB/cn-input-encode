#[inline]
pub(crate) fn pack(a: char, b: char) -> u64 {
    (a as u64) << 32 | (b as u64)
}

#[inline]
pub(crate) fn fx_hash_map_with_capacity<K, V>(c: usize) -> rustc_hash::FxHashMap<K, V> {
    rustc_hash::FxHashMap::with_capacity_and_hasher(c, Default::default())
}
