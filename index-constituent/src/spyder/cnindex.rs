use std::collections::HashMap;

use reqwest::header;
use reqwest::Client;

use super::json_to_number;
use super::Item;
use super::SpyderIndexData;
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
            header::HeaderValue::from_static("gzip, deflate"),
        );
        headers.insert(
            "Accept-Language",
            header::HeaderValue::from_static(
                "zh-CN,zh;q=0.8,zh-TW;q=0.7,zh-HK;q=0.5,en-US;q=0.3,en;q=0.2",
            ),
        );
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json, text/javascript, */*; q=0.01"),
        );

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
        "http://www.cnindex.com.cn/index/search".to_string()
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
    async fn search(&self, keyword: &str, page_number: usize) -> reqwest::Result<SpyderIndexData> {
        let search_page_uri = self.get_search_page_uri(keyword);
        let res = self.client.get(search_page_uri).header("Accept", " text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8").send().await?;

        if res.status().is_success() {
            let search_uri = self.get_search_content_uri();
            let mut hash_map = HashMap::new();

            hash_map.insert("content", keyword.to_owned());
            hash_map.insert("rows", format!("{}", self.page_size));
            hash_map.insert("pageNum", format!("{}", page_number));

            let res = self.client.post(&search_uri).form(&hash_map).send().await?;

            if self.debug {
                eprintln!(
                    "In search, url = `{}`, status: {:?}， res: {:?}",
                    search_uri,
                    res.status(),
                    res,
                );
            }

            if res.status().is_success() {
                let text = res.text().await?;

                if let Ok(json) = json::parse(&text) {
                    let mut ret = SpyderIndexData::default();

                    for (name, value) in json.entries() {
                        if name == "data" {
                            for (name, value) in value.entries() {
                                if name == "rows" {
                                    for member in value.members() {
                                        let mut item = Item::default();

                                        for (name, inner_value) in member.entries() {
                                            match name {
                                                "indexcode" => {
                                                    item.code = String::from(
                                                        inner_value.as_str().unwrap_or(""),
                                                    );
                                                }
                                                "indexname" => {
                                                    item.name = String::from(
                                                        inner_value.as_str().unwrap_or(""),
                                                    );
                                                }
                                                "samplesize" => {
                                                    item.number =
                                                        json_to_number(inner_value).unwrap_or(0);
                                                }
                                                _ => {}
                                            }
                                        }
                                        ret.push(item);
                                    }
                                    return Ok(ret);
                                } else if name == "total" {
                                    ret.set_total(value.as_i64().unwrap_or(0) as _);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(SpyderIndexData::default())
    }

    async fn fetch_cons(&self, code: &str, page_number: usize) -> reqwest::Result<SpyderIndexData> {
        let index_uri = self.get_index_uri(code);
        let res = self.client.get(index_uri).send().await?;
        let now = Utc::now();
        let date = format!("{}-{:02}", now.year_ce().1, now.month() - 1);

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
                    let mut ret = SpyderIndexData::default();

                    for (name, value) in json.entries() {
                        if name == "data" {
                            for (name, value) in value.entries() {
                                if name == "rows" {
                                    for member in value.members() {
                                        let mut item = Item::default();

                                        for (name, inner_value) in member.entries() {
                                            match name {
                                                "seccode" => {
                                                    item.code = String::from(
                                                        inner_value.as_str().unwrap_or(""),
                                                    );
                                                }
                                                "secname" => {
                                                    item.name = String::from(
                                                        inner_value.as_str().unwrap_or(""),
                                                    );
                                                }
                                                "weight" => {
                                                    item.number =
                                                        json_to_number(inner_value).unwrap_or(0);
                                                }
                                                _ => {}
                                            }
                                        }
                                        ret.push(item);
                                    }
                                    return Ok(ret);
                                }
                            }
                        } else if name == "total" {
                            ret.set_total(value.as_i64().unwrap_or(0) as _);
                        }
                    }
                }
            }
        }

        Ok(SpyderIndexData::default())
    }
}
