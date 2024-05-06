use super::json_to_number;
use super::Item;
use super::SpyderIndexData;
use reqwest::header;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct CSIndex {
    client: Client,
    debug: bool,
    page_size: usize,
}

impl CSIndex {
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

    pub fn get_search_content_uri(&self, keyword: &str, page_number: usize) -> String {
        format!("https://www.csindex.com.cn/csindex-home/index-list/search-result-about-index?searchInput={}&pageNum={}&pageSize={}", keyword, page_number, self.page_size)
    }

    pub fn get_cons_content_uri(&self, code: &str) -> String {
        format!(
            "https://www.csindex.com.cn/csindex-home/index/weight/top10/{}",
            code
        )
    }

    pub fn get_search_page_uri(&self, keyword: &str) -> String {
        format!("https://www.csindex.com.cn/#/search?searchText={}", keyword)
    }

    pub fn get_index_uri(&self, code: &str) -> String {
        format!(
            "https://www.csindex.com.cn/#/indices/family/detail?indexCode={}",
            code
        )
    }

    // pub fn get_list_uri(&self) -> String {
    //     format!("https://www.csindex.com.cn/#/indices/family/list")
    // }

    // pub fn get_list_content_uri(&self) -> String {
    //     format!("https://www.csindex.com.cn/csindex-home/index-list/query-index-item")
    // }
}

#[async_trait::async_trait]
impl super::Spyder for CSIndex {
    async fn search(&self, keyword: &str, page_number: usize) -> reqwest::Result<SpyderIndexData> {
        let search_page_uri = self.get_search_page_uri(keyword);
        let res = self.client.get(search_page_uri).send().await?;

        if res.status().is_success() {
            let search_uri = self.get_search_content_uri(keyword, page_number);
            let res = self.client.get(&search_uri).send().await?;

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
                        match name {
                            "data" => {
                                for member in value.members() {
                                    let mut item = Item::default();

                                    for (name, inner_value) in member.entries() {
                                        match name {
                                            "indexCode" => {
                                                item.code = String::from(
                                                    inner_value.as_str().unwrap_or(""),
                                                );
                                            }
                                            "indexName" => {
                                                item.name = String::from(
                                                    inner_value.as_str().unwrap_or(""),
                                                );
                                            }
                                            "consNumber" => {
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
                            "total" => {
                                ret.set_total(value.as_i64().unwrap_or(0) as _);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(SpyderIndexData::default())
    }

    async fn fetch_cons(
        &self,
        code: &str,
        _page_number: usize,
    ) -> reqwest::Result<SpyderIndexData> {
        let index_uri = self.get_index_uri(code);
        let res = self.client.get(index_uri).send().await?;

        if res.status().is_success() {
            let cons_content_uri = self.get_cons_content_uri(code);
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

                if self.debug {
                    dbg!(&text);
                }

                if let Ok(json) = json::parse(&text) {
                    let mut ret = SpyderIndexData::default();

                    ret.set_total(10);
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
                        }
                    }
                }
            }
        }

        Ok(SpyderIndexData::default())
    }
}
