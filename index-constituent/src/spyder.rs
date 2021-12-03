pub mod cnindex;
pub mod csindex;

use std::ops::{Deref, DerefMut};

#[async_trait::async_trait]
pub trait Spyder {
    async fn list(&self, page_number: usize) -> reqwest::Result<SpyderIndexData>;

    async fn search(&self, keyword: &str, page_number: usize) -> reqwest::Result<SpyderIndexData>;

    async fn fetch_cons(&self, code: &str, page_number: usize) -> reqwest::Result<SpyderConsData>;
}

#[derive(Debug, Clone, Default)]
pub struct Item {
    pub code: String,
    pub name: String,
    pub number: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SpyderIndexData(Vec<Item>);

impl Deref for SpyderIndexData {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SpyderIndexData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct SpyderConsData(Vec<Item>);

impl Deref for SpyderConsData {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SpyderConsData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
