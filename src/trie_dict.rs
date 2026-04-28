use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Default)]
struct WNode {
    /// 键为字，值为w_pool索引
    next: FxHashMap<char, usize>,
    /// (编码, 开销, c_pool索引)
    info: Vec<(String, f64, usize)>,
}

#[derive(Default)]
struct CNode {
    /// 键为码元，值为c_pool索引
    next: FxHashMap<char, usize>,
}

pub(crate) struct TrieDict {
    w_pool: Vec<WNode>,
    c_pool: Vec<CNode>,
}

impl TrieDict {
    pub(crate) fn new(path: &str, costs: &crate::cost_map::CostMap) -> crate::DynResult<Self> {
        let s = std::fs::read_to_string(path)?;
        let mut entries = parse_entries(&s)?;
        entries.sort_by(|a, b| b.2.total_cmp(&a.2));

        let mut w_pool = Vec::with_capacity(262144);
        let next = FxHashMap::with_capacity_and_hasher(8192, Default::default());
        w_pool.push(WNode { next, info: vec![] });
        let mut c_pool = Vec::with_capacity(262144);
        let next = FxHashMap::with_capacity_and_hasher(32, Default::default());
        c_pool.push(CNode { next });

        let mut codes = FxHashSet::with_capacity_and_hasher(entries.len(), Default::default());
        for (word, code, _) in entries {
            let code = distinct_and_record(code, &mut codes);
            let mut w_i = 0;
            for c in word.chars() {
                let i = w_pool.len();
                w_i = *w_pool[w_i].next.entry(c).or_insert(i);
                if w_i == i {
                    w_pool.push(WNode::default());
                }
            }
            let mut c_i = 0;
            for c in code.chars() {
                let i = c_pool.len();
                c_i = *c_pool[c_i].next.entry(c).or_insert(i);
                if c_i == i {
                    c_pool.push(CNode::default());
                }
            }
            let cost = costs.get_seq(&code);
            w_pool[w_i].info.push((code, cost, c_i));
        }

        Ok(Self { w_pool, c_pool })
    }

    /// 返回：是否可能有词在s末截断
    pub(crate) fn for_each_head<F>(&self, s: &[char], mut f: F) -> crate::DynResult<bool>
    where
        F: FnMut(usize, &str, f64, usize) -> crate::DynResult<()>,
    {
        let mut node = &self.w_pool[0];
        let mut wl = 0;
        while wl < s.len()
            && let Some(&next) = node.next.get(&s[wl])
        {
            node = &self.w_pool[next];
            wl += 1;
            for (code, cost, c_i) in &node.info {
                f(wl, code, *cost, *c_i)?;
            }
        }
        Ok(wl == s.len() && node.next.len() > 0)
    }

    #[inline]
    pub(crate) fn need_space(&self, c_node_i: usize, next_char: char) -> bool {
        self.c_pool[c_node_i].next.contains_key(&next_char)
    }
}

fn parse_entries(s: &str) -> Result<Vec<(&str, &str, f64)>, &'static str> {
    let entries: Vec<_> = s
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.len() == 0 || l.starts_with('#') {
                return None;
            }
            let mut parts = l.splitn(4, '\t');
            let word = parts.next()?;
            let code = parts.next()?;
            let weight = parts.next().map_or(0.0, |p| {
                let (s, d) = p.strip_suffix('%').map_or((p, 1.0), |p| (p, 100.0));
                s.parse::<f64>().ok().map_or(0.0, |v| v / d)
            });
            (word.len() > 0 && code.len() > 0).then_some((word, code, weight))
        })
        .collect();
    if entries.len() == 0 {
        return Err("无有效词条");
    }
    Ok(entries)
}

fn distinct_and_record(code: &str, codes: &mut FxHashSet<String>) -> String {
    let mut code = code.to_string();
    if codes.contains(&code) {
        for c in ('2'..='9').chain(['=']).cycle() {
            code.push(c);
            if !codes.contains(&code) {
                break;
            }
            if c != '=' {
                code.pop();
            }
        }
    }
    codes.insert(code.clone());
    code
}
