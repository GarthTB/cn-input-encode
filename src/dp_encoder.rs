use crate::{DynResult, cost_map::CostMap, trie_dict::TrieDict};

#[derive(Default)]
struct State<'a> {
    /// 前驱状态距离、内层索引
    prev: Option<(usize, usize)>,
    /// 前驱末词编码的末码元、池索引
    key: (char, usize),
    /// 前驱末词编码
    code: &'a str,
    /// code前是否需要加空格
    space: bool,
    /// 包括code在内的总开销
    cost: f64,
}

pub(crate) struct Encoder<'a> {
    dict: &'a TrieDict,
    text: Vec<char>,
    states: Vec<Vec<State<'a>>>,
    base: usize,
    cur: usize,
    end: usize,
    t_len: u64,
}

impl<'a> Encoder<'a> {
    pub(crate) fn new(dict: &'a TrieDict) -> Self {
        Self {
            dict,
            text: Vec::with_capacity(crate::CHUNK_SIZE),
            states: Vec::with_capacity(crate::CHUNK_SIZE),
            base: 0,
            cur: 0,
            end: 0,
            t_len: 0,
        }
    }

    pub(crate) fn prepare(&mut self) {
        self.text.clear();
        self.states.clear();
        self.states.push(vec![State::default()]);
        self.base = 0;
        self.cur = 0;
        self.end = 0;
        self.t_len = 0;
    }

    pub(crate) fn shrink(&mut self) {
        if self.base == 0 {
            return;
        }
        self.text.copy_within(self.base.., 0);
        self.text.truncate(self.text.len() - self.base);
        self.states.truncate(self.end + 1);
        self.states.drain(..self.base);
        self.states[0][0].prev = None;
        self.cur -= self.base;
        self.end -= self.base;
        self.base = 0;
    }

    pub(crate) fn feed(&mut self, text: &str) -> (u64, u64) {
        let bef = self.text.len();
        self.text.extend(text.chars());
        let aft = self.text.len();
        self.states.resize_with(aft + 1, Vec::new);
        let from = self.t_len + 1;
        self.t_len += (aft - bef) as u64;
        (from, self.t_len)
    }

    pub(crate) fn proc_chunk(&mut self, costs: &CostMap) -> DynResult<()> {
        Ok(while self.cur < self.text.len() {
            if self.cur == self.end && self.states[self.cur].len() == 1 {
                self.base = self.cur;
            }
            if self
                .dict
                .for_each_head(&self.text[self.cur..], |wl, code, cost, c_node_i| {
                    let mut c = code.chars();
                    let c0 = c.next().unwrap();
                    let key = (c.next_back().unwrap_or(c0), c_node_i);
                    for s in &self.states[self.cur] {
                        let space = s.key.1 > 0 && self.dict.need_space(s.key.1, c0);
                        let total_cost = if space {
                            s.cost + costs.get_pair(s.key.0, ' ') + costs.get_pair(' ', c0) + cost
                        } else {
                            s.cost + costs.get_pair(s.key.0, c0) + cost
                        };
                        todo!("更新self.states[self.cur + wl].iter_mut().find(|s| s.key == key)")
                    }
                    Ok(self.end = self.end.max(self.cur + wl))
                })?
            {
                break;
            }
            if self.states[self.cur + 1].len() == 0 {
                todo!("填入原字符")
            }
            self.cur += 1;
        })
    }

    pub(crate) fn proc_end(&mut self, costs: &CostMap) -> DynResult<(u64, f64)> {
        todo!("处理尾部、计算末尾空格、删除非最优编码、取出总字数和最小开销")
    }

    pub(crate) fn build_encoding(&self) -> Vec<u8> {
        if self.base == 0 {
            return vec![];
        }
        let mut buf = Vec::with_capacity(crate::CHUNK_SIZE);
        let mut tmp = [0; 4];
        let (mut i, mut j) = (self.base, 0);
        while let Some((wl, prev_j)) = self.states[i][j].prev {
            let state = &self.states[i][j];
            let s = match state.code {
                "" => self.text[i - 1].encode_utf8(&mut tmp),
                c => c,
            };
            buf.extend(s.as_bytes().iter().rev());
            if state.space {
                buf.push(b' ');
            }
            (i, j) = (i - wl, prev_j);
        }
        buf.reverse();
        buf
    }
}
