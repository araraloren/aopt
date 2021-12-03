use std::io::Stdout;

use aopt::app::SingleApp;
use aopt::err::create_error;
use aopt::err::Result;
use aopt::prelude::*;
use aopt_help::printer::Printer;
use aopt_help::store::OptStore;
use aopt_help::store::PosStore;
use aopt_help::AppHelp;
use aopt_help::DefaultFormat;
use reqwest::header;
use reqwest::Client;

const SEARCH_CMD: &'static str = "search";
const CONS_CMD: &'static str = "cons";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    let mut app = SingleApp::<SimpleSet, SimpleParser<UidGenerator>>::default();

    app = app.with_optset(
        SimpleSet::default()
            .with_default_creator()
            .with_default_prefix(),
    );

    for (opt, long, help, value) in [
        ("-h=b", "-help", "Print help message", None),
        ("-d=b", "--debug", "Print debug message", None),
        (
            "-n=i",
            "--page-number",
            "Set page number of results",
            Some(OptValue::from(1i64)),
        ),
    ] {
        let mut commit = app.add_opt(opt)?;

        if let Some(value) = value {
            commit.set_default_value(value);
        }
        commit.add_alias(long)?;
        commit.set_help(help);
        commit.commit()?;
    }

    let uid = app
        .add_opt("-t=s!")?
        .add_alias("--type")?
        .set_help("Search keyword and list result")
        .commit()?;

    app.add_callback(
        uid,
        simple_opt_cb!(|_, _, value| {
            match value.as_str().unwrap_or(&String::default()).as_str() {
                "CS" | "CN" => Ok(Some(value)),
                _ => Err(create_error(format!(
                    "The type must be one of [\"CS\", \"CN\"]!"
                ))),
            }
        }),
    );
    let uid = app
        .add_opt("-s=i")?
        .add_alias("--page-size")?
        .set_help("Set page size of results")
        .set_default_value(OptValue::from(30i64))
        .commit()?;

    app.add_callback(
        uid,
        simple_opt_cb!(|_, _, value| {
            match value.as_int().unwrap_or(&0) {
                5 | 10 | 20 | 30 => Ok(Some(value)),
                _ => Err(create_error(format!(
                    "The page size must be one of [5, 10, 20, 30]!"
                ))),
            }
        }),
    );
    app.add_opt("search=c")?
        .set_help("Search keyword and list result")
        .commit()?;

    app.add_opt("cons=c")?
        .set_help("Get and list constituents of given code")
        .commit()?;

    Ok(app
        .run_async(std::env::args().skip(1), |ret, app| async move {
            let set = app.get_optset();
            let data = app.get_parser().get_noa()[1];

            dbg!(&set);

            if !ret {
                print_help(set)?;
            } else {
                if !print_help(set)? {
                    if *value_of(set, SEARCH_CMD)?.as_bool().unwrap_or(&false) {
                        let ret = search_keyword(set, data.as_str()).await?;

                        dbg!(ret);
                    } else if *value_of(set, CONS_CMD)?.as_bool().unwrap_or(&false) {
                    }
                }
            }
            Ok(())
        })
        .await?)
}

async fn search_keyword(set: &dyn Set, arg: &str) -> Result<Vec<CommonRes>> {
    let string = String::default();
    let debug = value_of(set, "--debug")?.as_bool().unwrap_or(&false);
    let type_ = value_of(set, "--type")?
        .as_str()
        .unwrap_or(&string)
        .as_str();
    let page_size = value_of(set, "--page-size")?.as_int().unwrap_or(&30);
    let page_number = value_of(set, "--page-number")?.as_int().unwrap_or(&1);
    match type_ {
        "CS" => {
            let csspyder = CSIndexSpyder::new(*debug, *page_size as usize)
                .map_err(|e| create_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = csspyder
                .search(arg, *page_number as usize)
                .await
                .map_err(|e| create_error(format!("Can not search {}: {:?}", arg, e)))?;

            Ok(ret)
        }
        "CN" => {
            todo!()
        }
        _ => Ok(vec![]),
    }
}

fn value_of<'a>(set: &'a dyn Set, opt: &str) -> Result<&'a OptValue> {
    Ok(set
        .find(opt)?
        .ok_or(create_error(format!("can not get option {}", opt)))?
        .get_value())
}

#[derive(Debug, Clone, Default)]
pub struct CommonRes {
    pub code: String,
    pub name: String,
    pub data: usize,
}

#[async_trait::async_trait]
pub trait FundSpyder {
    async fn reset(&self, url: &str) -> reqwest::Result<bool>;

    async fn search(&self, keyword: &str, page_number: usize) -> reqwest::Result<Vec<CommonRes>>;

    async fn fetch_cons(&self, code: &str) -> reqwest::Result<Vec<CommonRes>>;
}

#[derive(Debug, Clone)]
pub struct CSIndexSpyder {
    client: Client,
    debug: bool,
    page_sized: usize,
}

impl CSIndexSpyder {
    pub fn new(debug: bool, page_sized: usize) -> reqwest::Result<Self> {
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
            page_sized: page_sized,
        })
    }

    pub fn format_search_uri(&self, keyword: &str, page_number: usize) -> String {
        format!("https://www.csindex.com.cn/csindex-home/index-list/search-result-about-index?searchInput={}&pageNum={}&pageSize={}", keyword, page_number, self.page_sized)
    }

    pub fn format_cons_uri(&self, code: &str) -> String {
        format!(
            "https://www.csindex.com.cn/csindex-home/index/weight/top10/{}",
            code
        )
    }

    pub fn format_search_reset(&self, keyword: &str) -> String {
        format!("https://www.csindex.com.cn/#/search?searchText={}", keyword)
    }
}

#[async_trait::async_trait]
impl FundSpyder for CSIndexSpyder {
    async fn reset(&self, url: &str) -> reqwest::Result<bool> {
        let res = self.client.get(url).send().await?;

        if self.debug {
            eprintln!("ACCESS `{}`, status: {:?}", url, res.status());
        }

        Ok(res.status().is_success())
    }

    async fn search(&self, keyword: &str, page_number: usize) -> reqwest::Result<Vec<CommonRes>> {
        let mut ret = vec![];
        
        if self.reset(&self.format_search_reset(keyword)).await? {
            let search_uri = self.format_search_uri(keyword, page_number);

            dbg!(&search_uri);

            let res = self.client.get(search_uri).send().await?;

            if self.debug {
                eprintln!("SERACH `{}`, status: {:?}", keyword, res.status());
            }
            if res.status().is_success() {
                let text = res.text().await?;

                if let Ok(json) = json::parse(&text) {
                    dbg!(json);
                }
            }
        }
        Ok(ret)
    }

    async fn fetch_cons(&self, code: &str) -> reqwest::Result<Vec<CommonRes>> {
        let mut ret = vec![];
        let search_uri = self.format_cons_uri(code);
        let res = self.client.get(search_uri).send().await?;

        if self.debug {
            eprintln!("FETCH CONS of `{}`, status: {:?}", code, res.status());
        }
        if res.status().is_success() {
            let text = res.text().await?;

            if let Ok(json) = json::parse(&text) {
                dbg!(json);
            }
        }
        Ok(ret)
    }
}

#[derive(Debug, Clone)]
pub struct CNIndexSpyder {
    client: Client,
    debug: bool,
}

impl CNIndexSpyder {
    pub fn new(debug: bool) -> reqwest::Result<Self> {
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

        Ok(Self { client, debug })
    }
}

fn print_help(set: &dyn Set) -> Result<bool> {
    let mut is_need_help = false;

    if let Ok(Some(opt)) = set.find("help") {
        if opt.get_value().as_bool() == Some(&true) {
            is_need_help = true;
        }
    }
    if is_need_help {
        let mut app_help = simple_help_generate(set);

        if *value_of(set, SEARCH_CMD)?.as_bool().unwrap_or(&false) {
            app_help
                .print_cmd_help(Some(SEARCH_CMD.into()))
                .map_err(|e| create_error(format!("can not write help to stdout: {:?}", e)))?;
        } else if *value_of(set, CONS_CMD)?.as_bool().unwrap_or(&false) {
            app_help
                .print_cmd_help(Some(CONS_CMD.into()))
                .map_err(|e| create_error(format!("can not write help to stdout: {:?}", e)))?;
        } else {
            app_help
                .print_cmd_help(None)
                .map_err(|e| create_error(format!("can not write help to stdout: {:?}", e)))?;
        }
    }
    Ok(is_need_help)
}

fn simple_help_generate(set: &dyn Set) -> AppHelp<Stdout, DefaultFormat> {
    let mut help = AppHelp::default();

    help.set_name("index".into());

    let version = Ustr::from("Create by araraloren, V0.1.0");
    let global = help.store.get_global_mut();

    global.set_header(Ustr::from("Search and list index constituents"));
    global.set_footer(version.clone());

    for cmd_name in [SEARCH_CMD, CONS_CMD] {
        if let Ok(Some(opt)) = set.find(cmd_name) {
            let mut search_cmd = help.store.new_cmd(cmd_name.into());

            search_cmd
                .set_footer(version.clone())
                .set_hint(opt.get_hint())
                .set_help(opt.get_help());

            for opt in set.opt_iter() {
                if opt.match_style(aopt::opt::Style::Pos) {
                    search_cmd.add_pos(PosStore::new(
                        opt.get_name(),
                        opt.get_hint(),
                        opt.get_help(),
                        opt.get_index().unwrap().to_string().into(),
                        opt.get_optional(),
                    ));
                } else if !opt.match_style(aopt::opt::Style::Main)
                    && !opt.match_style(aopt::opt::Style::Cmd)
                {
                    search_cmd.add_opt(OptStore::new(
                        opt.get_name(),
                        opt.get_hint(),
                        opt.get_help(),
                        opt.get_optional(),
                    ));
                }
            }

            search_cmd.commit();
        }
    }

    help
}
