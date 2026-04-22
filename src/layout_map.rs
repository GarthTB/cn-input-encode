pub(crate) struct LayoutMap {
    map: rustc_hash::FxHashMap<char, u8>,
}

impl LayoutMap {
    pub(crate) fn new(layout: &[String]) -> Result<Self, String> {
        if layout.len() != 14 {
            return Err("布局配置非14项".into());
        }
        let mut map = crate::utils::fx_hash_map_with_capacity(50);
        let (fingers, rows) = layout.split_at(9);
        for (i, f) in (1u8..10).zip(fingers) {
            let v = i << 4;
            for c in f.chars() {
                if map.insert(c, v).is_some() {
                    return Err(format!("键'{c}'手指冲突"));
                }
            }
        }
        for (i, r) in (1u8..6).zip(rows) {
            for c in r.chars() {
                if *map.entry(c).and_modify(|v| *v += i).or_insert(i) & 0xF != i {
                    return Err(format!("键'{c}'排冲突"));
                }
            }
        }
        Ok(Self { map })
    }

    #[inline]
    pub(crate) fn get(&self, c: char) -> (u8, u8) {
        let v = self.map.get(&c).unwrap_or(&0);
        (v >> 4, v & 0xF)
    }
}
