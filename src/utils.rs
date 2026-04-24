use std::io::Read;

pub(crate) fn fx_hash_map_with_capacity<K, V>(c: usize) -> rustc_hash::FxHashMap<K, V> {
    rustc_hash::FxHashMap::with_capacity_and_hasher(c, Default::default())
}

pub(crate) fn for_each_chunk<F: FnMut(&str)>(path: &str, mut f: F) -> crate::DynResult<()> {
    let mut buf = vec![0; crate::CHUNK_SIZE].into_boxed_slice();
    let mut len = 0;
    let mut file = std::fs::File::open(path)?;
    loop {
        match file.read(&mut buf[len..]) {
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
            Ok(n) if n > 0 => len += n,
            _ => return Ok(()),
        }
        let s = match std::str::from_utf8(&buf[..len]) {
            Err(e) if e.error_len().is_some() => return Err(e.into()),
            Err(e) => unsafe { std::str::from_utf8_unchecked(&buf[..e.valid_up_to()]) },
            Ok(s) => s,
        };
        f(s);
        let s_len = s.len();
        if s_len > 0 && s_len < len {
            buf.copy_within(s_len..len, 0);
        }
        len -= s_len;
    }
}
