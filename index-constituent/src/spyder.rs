pub mod cnindex;
pub mod csindex;

use std::ops::{Deref, DerefMut};

use json::JsonValue;

#[async_trait::async_trait]
pub trait Spyder {
    async fn list(&self, page_number: usize) -> reqwest::Result<SpyderIndexData>;

    async fn search(&self, keyword: &str, page_number: usize) -> reqwest::Result<SpyderIndexData>;

    async fn fetch_cons(&self, code: &str, page_number: usize) -> reqwest::Result<SpyderIndexData>;
}

#[derive(Debug, Clone, Default)]
pub struct Item {
    pub code: String,
    pub name: String,
    pub number: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SpyderIndexData {
    total: usize,
    data: Vec<Item>,
}

impl SpyderIndexData {
    pub fn set_total(&mut self, total: usize) {
        self.total = total;
    }
}

impl Deref for SpyderIndexData {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for SpyderIndexData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub fn json_to_number(json: &JsonValue) -> Option<u64> {
    if json.is_number() {
        return Some(json.as_fixed_point_u64(2).unwrap_or(0));
    } else if json.is_string() {
        if let Some(json) = json.as_str() {
            if let Ok(v) = json.parse::<u64>() {
                return Some(v);
            }
        }
    }
    None
}
