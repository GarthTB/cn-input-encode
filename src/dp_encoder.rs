use crate::{cost_map::CostMap, trie_dict::TrieDict};

#[derive(Default)]
struct State<'a> {
    /// 前驱状态距离、内层索引
    prev: (usize, usize),
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
        }
    }

    pub(crate) fn prepare(&mut self) {
        self.text.clear();
        self.states.clear();
        self.states.push(vec![State::default()]);
        self.base = 0;
        self.cur = 0;
        self.end = 0;
    }

    pub(crate) fn shrink(&mut self) {
        if self.base == 0 {
            return;
        }
        self.text.copy_within(self.base.., 0);
        self.text.truncate(self.text.len() - self.base);
        self.states.truncate(self.end + 1);
        self.states.drain(..self.base);
        self.states[0][0].prev.0 = 0;
        self.cur -= self.base;
        self.end -= self.base;
        self.base = 0;
    }

    pub(crate) fn feed(&mut self, text: &str) -> u64 {
        let bef = self.text.len();
        self.text.extend(text.chars());
        let aft = self.text.len();
        self.states.resize_with(aft + 1, Vec::new);
        (aft - bef) as u64
    }

    pub(crate) fn proc_chunk(&mut self, costs: &CostMap, break_on_trunc: bool) {
        let delta = |c0: char, c1: char, space: bool| match c0 {
            '\0' => 0.0,
            c0 if space => costs.get_pair(c0, ' ') + costs.get_pair(' ', c1),
            c0 => costs.get_pair(c0, c1),
        };
        while self.cur < self.text.len() {
            if self
                .dict
                .for_each_head(&self.text[self.cur..], |wl, code, cost, c_node_i| {
                    let mut c = code.chars();
                    let c1 = c.next().unwrap();
                    let key = (c.next_back().unwrap_or(c1), c_node_i);
                    for i in 0..self.states[self.cur].len() {
                        let s = &self.states[self.cur][i];
                        let space = s.key.1 > 0 && self.dict.need_space(s.key.1, c1);
                        let cost = s.cost + delta(s.key.0, c1, space) + cost;
                        let target = self.states[self.cur + wl].iter_mut().find(|s| s.key == key);
                        if let Some(s) = target {
                            if s.cost > cost {
                                s.prev = (wl, i);
                                s.key = key;
                                s.code = code;
                                s.space = space;
                                s.cost = cost;
                            }
                        } else {
                            self.states[self.cur + wl].push(State {
                                prev: (wl, i),
                                key,
                                code,
                                space,
                                cost,
                            });
                        }
                    }
                    self.end = self.end.max(self.cur + wl)
                })
                && break_on_trunc
            {
                break;
            }
            if self.cur == self.end {
                let c1 = self.text[self.cur];
                let mut best_i = 0;
                let mut space = false;
                let mut min_cost = f64::INFINITY;
                for (i, s) in self.states[self.cur].iter().enumerate() {
                    let sp = s.key.1 > 0 && self.dict.need_space(s.key.1, c1);
                    let cost = s.cost + delta(s.key.0, c1, sp);
                    if cost < min_cost {
                        best_i = i;
                        space = sp;
                        min_cost = cost;
                    }
                }
                self.states[self.cur + 1].push(State {
                    prev: (1, best_i),
                    key: (c1, 0),
                    code: "",
                    space,
                    cost: min_cost,
                });
                self.end += 1;
            }
            self.cur += 1;
            if self.cur == self.end && self.states[self.cur].len() == 1 {
                self.base = self.cur;
            }
        }
    }

    pub(crate) fn proc_end(&mut self, costs: &CostMap) -> f64 {
        let states = &mut self.states[self.cur];
        for s in states.iter_mut() {
            if !s.key.0.is_ascii_digit() && s.key.1 > 0 {
                s.cost += costs.get_pair(s.key.0, ' ');
            }
        }
        states.select_nth_unstable_by(0, |a, b| a.cost.total_cmp(&b.cost));
        states.truncate(1);
        let s = &mut states[0];
        if !s.key.0.is_ascii_digit() && s.key.1 > 0 {
            s.code = Box::leak(format!("{} ", s.code).into());
        }
        self.base = self.cur;
        s.cost
    }

    pub(crate) fn build_encoding(&self) -> Vec<u8> {
        if self.base == 0 {
            return vec![];
        }
        let mut buf = Vec::with_capacity(crate::CHUNK_SIZE);
        let mut tmp = [0; 4];
        let (mut i, mut j) = (self.base, 0);
        while self.states[i][j].prev.0 > 0 {
            let state = &self.states[i][j];
            let mut code = state.code;
            if code.len() == 0 {
                code = self.text[i - 1].encode_utf8(&mut tmp);
            };
            buf.extend(code.as_bytes().iter().rev());
            if state.space {
                buf.push(b' ');
            }
            i -= state.prev.0;
            j = state.prev.1;
        }
        buf.reverse();
        buf
    }
}
