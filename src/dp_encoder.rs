use crate::{DynResult, cost_map::CostMap, trie_dict::TrieDict};

#[derive(Default)]
struct State<'a> {
    /// 前驱状态相对索引
    prev: Option<(usize, usize)>,
    /// 当前词的编码
    code: &'a str,
    /// code的末码元、编码池索引
    key: (char, usize),
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
    c_len: u64,
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
            c_len: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.text.clear();
        self.states.clear();
        self.states.push(vec![State::default()]);
        self.base = 0;
        self.cur = 0;
        self.end = 0;
        self.t_len = 0;
        self.c_len = 0;
    }

    pub(crate) fn shrink_and_feed(&mut self, text: &str) -> (u64, u64) {
        todo!()
    }

    pub(crate) fn proc_chunk(&mut self, costs: &CostMap) {}

    pub(crate) fn join_and_append(&self, file: &mut std::fs::File) -> DynResult<()> {
        if self.base == 0 {
            return Ok(());
        }
        todo!()
    }

    pub(crate) fn proc_end(&mut self, file: &mut std::fs::File) -> DynResult<(u64, u64, f64)> {
        todo!()
    }
}
