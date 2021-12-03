use reqwest::header;
use reqwest::Client;

use super::Item;
use super::{SpyderConsData, SpyderIndexData};
use chrono::{Datelike, Utc};

#[derive(Debug, Clone)]
pub struct CNIndex {
    client: Client,
    debug: bool,
    page_size: usize,
}

impl CNIndex {
    pub fn new(debug: bool, page_size: usize) -> reqwest::Result<Self> {
        let mut headers = header::HeaderMap::new();

        headers.insert(
            "Accept-Encoding",
            header::HeaderValue::from_static("gzip, deflate, br"),
        );
        headers.insert(
            "Accept-Language",
            header::HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6"),
        );
        headers.insert("Accept", header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"));

        let client = Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36 Edg/95.0.1020.44",
        )
        .default_headers(headers)
        .gzip(true)
        .cookie_store(true)
        .build()?;

        Ok(Self {
            client,
            debug,
            page_size,
        })
    }

    pub fn get_search_content_uri(&self) -> String {
        format!("http://www.cnindex.com.cn/index/search")
    }

    pub fn get_cons_content_uri(
        &self,
        code: &str,
        date: &str,
        page_number: usize,
        rows: usize,
    ) -> String {
        format!(
            "http://www.cnindex.com.cn/sample-detail/detail?indexcode={}&dateStr={}&pageNum={}&rows={}",
            code,
            date, page_number, rows
        )
    }

    pub fn get_search_page_uri(&self, keyword: &str) -> String {
        format!(
            "http://www.cnindex.com.cn/module/index-series.html?act_menu=1&index_type=0&search={}",
            keyword
        )
    }

    pub fn get_index_uri(&self, code: &str) -> String {
        format!(
            "http://www.cnindex.com.cn/module/index-detail.html?act_menu=1&indexCode={}",
            code
        )
    }

    // pub fn get_list_uri(&self) -> String {
    //     format!("http://www.cnindex.com.cn/zh_indices/sese/index.html?act_menu=1&index_type=-1")
    // }

    //  pub fn get_list_content_uri(&self, rows: usize, page_number: usize) -> String {
    //      format!("http://www.cnindex.com.cn/index/indexList?channelCode=-1&rows={}&pageNum={}", rows, page_number)
    //  }
}

#[async_trait::async_trait]
impl super::Spyder for CNIndex {
    async fn list(&self, _page_number: usize) -> reqwest::Result<SpyderIndexData> {
        todo!()
    }

    async fn search(&self, keyword: &str, page_number: usize) -> reqwest::Result<SpyderIndexData> {
        let search_page_uri = self.get_search_page_uri(keyword);
        let res = self.client.get(search_page_uri).send().await?;

        if res.status().is_success() {
            let search_uri = self.get_search_content_uri();
            let res = self
                .client
                .post(&search_uri)
                .body(format!(
                    "content={}&rows={}&pageNum={}",
                    keyword, self.page_size, page_number
                ))
                .send()
                .await?;

            if self.debug {
                eprintln!(
                    "In search, url = `{}`, status: {:?}",
                    search_uri,
                    res.status()
                );
            }

            if res.status().is_success() {
                let text = res.text().await?;

                if let Ok(json) = json::parse(&text) {
                    let mut ret = SpyderIndexData::default();

                    for (name, value) in json.entries() {
                        if name == "data" {
                            for (name, value) in value.entries() {
                                if name == "weightList" {
                                    for member in value.members() {
                                        let mut item = Item::default();

                                        for (name, inner_value) in member.entries() {
                                            match name {
                                                "securityCode" => {
                                                    item.code = String::from(
                                                        inner_value.as_str().unwrap_or(""),
                                                    );
                                                }
                                                "securityName" => {
                                                    item.name = String::from(
                                                        inner_value.as_str().unwrap_or(""),
                                                    );
                                                }
                                                "weight" => {
                                                    if let Some(v) = inner_value.as_str() {
                                                        if let Ok(num) = v.parse::<f64>() {
                                                            item.number = (num * 100.0) as u64;
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        ret.push(item);
                                    }
                                    return Ok(ret);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(SpyderIndexData::default())
    }

    async fn fetch_cons(&self, code: &str, page_number: usize) -> reqwest::Result<SpyderConsData> {
        let index_uri = self.get_index_uri(code);
        let res = self.client.get(index_uri).send().await?;
        let now = Utc::now();
        let date = format!("{}-{:02}-{:02}", now.year_ce().1, now.month(), now.day());

        if res.status().is_success() {
            let cons_content_uri =
                self.get_cons_content_uri(code, &date, page_number, self.page_size);
            let res = self.client.get(&cons_content_uri).send().await?;

            if self.debug {
                eprintln!(
                    " In cons, url = `{}`, status: {:?}",
                    cons_content_uri,
                    res.status()
                );
            }

            if res.status().is_success() {
                let text = res.text().await?;

                if let Ok(json) = json::parse(&text) {
                    let mut ret = SpyderConsData::default();

                    for (name, value) in json.entries() {
                        if name == "data" {
                            for (name, value) in value.entries() {
                                if name == "weightList" {
                                    for member in value.members() {
                                        let mut item = Item::default();

                                        for (name, inner_value) in member.entries() {
                                            match name {
                                                "securityCode" => {
                                                    item.code = String::from(
                                                        inner_value.as_str().unwrap_or(""),
                                                    );
                                                }
                                                "securityName" => {
                                                    item.name = String::from(
                                                        inner_value.as_str().unwrap_or(""),
                                                    );
                                                }
                                                "weight" => {
                                                    if let Some(v) = inner_value.as_str() {
                                                        if let Ok(num) = v.parse::<f64>() {
                                                            item.number = (num * 100.0) as u64;
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        ret.push(item);
                                    }
                                    return Ok(ret);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(SpyderConsData::default())
    }
}
