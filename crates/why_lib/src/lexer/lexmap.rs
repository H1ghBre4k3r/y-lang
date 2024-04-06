use std::collections::HashMap;

use super::Terminal;

/// Struct for storing terminal symbols with their respective "key".
#[derive(Debug, Clone, Default)]
pub struct LexMap {
    map: HashMap<&'static str, Terminal>,
}

impl LexMap {
    pub fn insert(&mut self, key: &'static str, value: Terminal) {
        self.map.insert(key, value);
    }

    pub fn can_match(&self, key: &str) -> bool {
        for map_key in self.map.keys() {
            if map_key.starts_with(key) {
                return true;
            }
        }
        false
    }

    pub fn get(&self, key: &str) -> Option<Terminal> {
        self.map.get(key).cloned()
    }
}
