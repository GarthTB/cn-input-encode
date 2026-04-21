pub(crate) struct CostMap {
    map: rustc_hash::FxHashMap<u64, f32>,
    mean: f32,
}

impl CostMap {
    pub(crate) fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut map = rustc_hash::FxHashMap::with_capacity_and_hasher(2500, Default::default());
        let mut sum = 0f64;
        for l in std::fs::read_to_string(path)?.lines() {
            let Some((k, v)) = l.split_once('\t') else {
                continue;
            };
            let mut c = k.chars();
            if let (Some(a), Some(b), None, Ok(v)) = (c.next(), c.next(), c.next(), v.parse()) {
                if let Some(v0) = map.insert(pack(a, b), v) {
                    return Err(format!("键对'{a}{b}'开销重复：'{v0}'、'{v}'").into());
                }
                sum += v as f64;
            }
        }
        let mean = match map.len() {
            0 => return Err("无有效键对".into()),
            v => (sum / v as f64) as f32,
        };
        Ok(Self { map, mean })
    }

    #[inline]
    pub(crate) fn get_pair(&self, a: char, b: char) -> f32 {
        *self.map.get(&pack(a, b)).unwrap_or(&self.mean)
    }

    #[inline]
    pub(crate) fn get_seq(&self, code: &str) -> f32 {
        let mut chars = code.chars();
        let mut sum = 0.0;
        if let Some(mut c0) = chars.next() {
            for c1 in chars {
                sum += self.get_pair(c0, c1);
                c0 = c1;
            }
        }
        sum
    }
}

#[inline]
fn pack(a: char, b: char) -> u64 {
    (a as u64) << 32 | (b as u64)
}
