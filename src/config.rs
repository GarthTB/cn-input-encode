use crate::{cost_map::CostMap, layout_map::LayoutMap, trie_dict::TrieDict};

#[derive(serde::Deserialize)]
struct RawConfig {
    text: Vec<String>,
    costs: String,
    dict: String,
    layout: Option<Vec<String>>,
}

pub(crate) struct Config {
    pub(crate) text: Vec<String>,
    pub(crate) costs: CostMap,
    pub(crate) dict: TrieDict,
    pub(crate) layout: Option<LayoutMap>,
}

impl Config {
    pub(crate) fn load() -> crate::DynResult<Self> {
        let s = std::fs::read_to_string("cfg/config.toml")?;
        let raw: RawConfig = toml::from_str(&s)?;
        let costs = CostMap::new(&raw.costs)?;
        let dict = TrieDict::new(&raw.dict, &costs)?;
        Ok(Self {
            text: raw.text,
            costs,
            dict,
            layout: raw.layout.map(|v| LayoutMap::new(&v)).transpose()?,
        })
    }
}
