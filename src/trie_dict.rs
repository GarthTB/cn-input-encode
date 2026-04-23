use crate::cost_map::CostMap;
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Default)]
struct WordNode {
    /// 键为字，值为w_pool索引
    next: FxHashMap<char, usize>,
    /// (编码, 开销, c_pool索引)
    info: Vec<(String, f64, usize)>,
}

#[derive(Default)]
struct CodeNode {
    /// 键为码元，值为c_pool索引
    next: FxHashMap<char, usize>,
}

pub(crate) struct TrieDict {
    /// 字词Trie：键为字，值为词的信息
    w_pool: Vec<WordNode>,
    /// 编码Trie：键为码元
    c_pool: Vec<CodeNode>,
}

impl TrieDict {
    pub(crate) fn new(path: &str, costs: &CostMap) -> crate::DynResult<Self> {
        let s = std::fs::read_to_string(path)?;
        let mut entries = parse_entries(&s)?;
        entries.sort_by(|a, b| b.2.total_cmp(&a.2));

        let mut w_pool = Vec::with_capacity(262144);
        let next = crate::utils::fx_hash_map_with_capacity(8192);
        let info = Vec::new();
        w_pool.push(WordNode { next, info });
        let mut c_pool = Vec::with_capacity(262144);
        let next = crate::utils::fx_hash_map_with_capacity(32);
        c_pool.push(CodeNode { next });

        let mut codes = FxHashSet::with_capacity_and_hasher(entries.len(), Default::default());
        for (word, code, _) in entries {
            let code = distinct_and_record(code, &mut codes);
            let mut w_idx = 0;
            for c in word.chars() {
                let i = w_pool.len();
                w_idx = *w_pool[w_idx].next.entry(c).or_insert(i);
                if w_idx == i {
                    w_pool.push(WordNode::default());
                }
            }
            let mut c_idx = 0;
            for c in code.chars() {
                let i = c_pool.len();
                c_idx = *c_pool[c_idx].next.entry(c).or_insert(i);
                if c_idx == i {
                    c_pool.push(CodeNode::default());
                }
            }
            let cost = costs.get_seq(&code);
            w_pool[w_idx].info.push((code, cost, c_idx));
        }

        Ok(Self { w_pool, c_pool })
    }

    pub(crate) fn for_each_head<F>(&self, s: &[char], mut f: F) -> Option<bool>
    where
        F: FnMut(u16, &str, f64, &FxHashMap<char, usize>),
    {
        let mut node = &self.w_pool[0];
        let mut w_len = 0;
        for c in s {
            let Some(&next) = node.next.get(&c) else {
                return Some(w_len > 0);
            };
            node = &self.w_pool[next];
            w_len += 1;
            for (code, cost, c_idx) in &node.info {
                f(w_len, code, *cost, &self.c_pool[*c_idx].next);
            }
        }
        (node.next.len() == 0).then_some(w_len > 0)
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
            let (word, code) = (parts.next()?, parts.next()?);
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
