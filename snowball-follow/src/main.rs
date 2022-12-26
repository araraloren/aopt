use std::borrow::Cow;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;

use aopt::ctx::VecStore;
use aopt::prelude::*;
use aopt::Error;
use aopt_help::prelude::Block;
use aopt_help::prelude::Store;
use reqwest::header;
use reqwest::Client;

const STOCK_NUMBER_LEN: usize = 6;
const STOCK_SHANGHAI: &str = "SH";
const STOCK_SHENZHEN: &str = "SZ";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;
    let parser = parser_command_line(Args::new(std::env::args().skip(1)))?;
    let debug = *parser.find_val::<bool>("--debug")?;
    let help = *parser.find_val::<bool>("--help")?;

    if help {
        display_help(parser.optset())?;
    } else {
        let mut ids: Vec<String> = vec![];

        for stock_id in parser.find_vals::<String>("stock_id")? {
            ids.push(stock_id.clone());
        }
        for stock_id in parser.find_vals::<String>("stock_file_list")? {
            ids.push(stock_id.clone());
        }
        if !ids.is_empty() {
            let start = *parser.find_val::<i64>("--start")?;
            let count = *parser.find_val::<i64>("--count")?;
            let interval = *parser.find_val::<u64>("--interval")?;

            let snowball = SnowBall::new(debug)?;

            if debug {
                eprintln!("Got ==> {:?}", ids);
            }
            if snowball
                .init(&format!("{}{}", STOCK_SHANGHAI, "000002"))
                .await?
            {
                for id in ids {
                    if let Ok(count) = snowball.get_snowball_follow(&id, start, count).await {
                        println!("{}: {}", id, count);
                    } else {
                        println!("{}: None", id);
                    }
                    tokio::time::sleep(Duration::from_millis(interval)).await;
                }
            }
        } else if debug {
            eprintln!("Stock list is empty: {:?}", ids);
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SnowBall {
    client: Client,
    debug: bool,
}

impl SnowBall {
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

    fn snowball_follow_uri(code: &str, start: i64, count: i64) -> String {
        format!(
            "https://www.xueqiu.com/recommend/pofriends.json?type=1&code={}&start={}&count={}",
            code, start, count
        )
    }

    fn snowball_home(code: &str) -> String {
        format!("https://www.xueqiu.com/S/{}", code)
    }

    pub async fn init(&self, code: &str) -> reqwest::Result<bool> {
        let home = Self::snowball_home(code);
        let res = self.client.get(&home).send().await?;

        if self.debug {
            eprintln!("ACCESS `{}` , status = {:?}", &home, res.status());
        }

        Ok(res.status().is_success())
    }

    pub async fn get_snowball_follow(
        &self,
        code: &str,
        start: i64,
        count: i64,
    ) -> reqwest::Result<i64> {
        let stock_follow_uri = Self::snowball_follow_uri(code, start, count);
        let res = self.client.get(&stock_follow_uri).send().await?;
        let mut ret = 0;

        if self.debug {
            eprintln!(
                "ACCESS `{}` , status = {:?}",
                &stock_follow_uri,
                res.status()
            );
        }

        if res.status().is_success() {
            let text = res.text().await?;

            if let Ok(json::JsonValue::Object(v)) = json::parse(&text) {
                if let Some(count) = v.get("totalcount") {
                    ret = count.as_i64().unwrap_or(0);
                }
            }
        }
        Ok(ret)
    }
}

fn parser_command_line(args: Args) -> Result<AFwdParser, Error> {
    let mut parser = AFwdParser::default();

    parser.policy_mut().set_strict(true);

    for (optstr, alias, help, value) in [
        ("-d=b", "--debug", "Print debug message", None),
        (
            "-i=u",
            "--interval",
            "Set access interval",
            Some(ValInitiator::u64(1000u64)),
        ),
        (
            "-s=i",
            "--start",
            "Set start parameter of request",
            Some(ValInitiator::i64(0i64)),
        ),
        (
            "-c=i",
            "--count",
            "Set count parameter of request",
            Some(ValInitiator::i64(14i64)),
        ),
    ] {
        if let Some(initiator) = value {
            parser
                .add_opt(optstr)?
                .set_initiator(initiator)
                .add_alias(alias)
                .set_help(help);
        } else {
            parser.add_opt(optstr)?.add_alias(alias).set_help(help);
        }
    }
    // process single stock id
    parser
        .add_opt("stock_id=p@*")?
        .set_help("Get follow from single stock id")
        .set_values(Vec::<String>::new())
        .on(|set: &mut ASet, ser: &mut ASer, val: ctx::Value<String>| {
            let id = convert_line_to_stock_number(val.deref());
            let debug = *ser.sve_val::<bool>(set["--debug"].uid())?;

            if debug {
                if id.is_none() {
                    eprintln!("{} is not a valid stock number!", val);
                } else {
                    eprintln!("Got a stock id: {:?}!", id);
                }
            }
            Ok(id)
        })?;

    // process single stock id
    parser
        .add_opt("stock_file_list=p@1")?
        .set_help("Get follow from stock list in file")
        .set_values(Vec::<String>::new())
        .on(
            |set: &mut ASet, ser: &mut ASer, file: ctx::Value<PathBuf>| {
                let mut ret = Ok(None);
                let debug = *ser.sve_val::<bool>(set["--debug"].uid())?;

                if file.is_file() {
                    let fh = File::open(file.as_path()).map_err(|e| {
                        Error::raise_error(format!("can not read file {:?}: {:?}", file, e))
                    })?;
                    let mut reader = BufReader::new(fh);
                    let mut ids = vec![];
                    let mut line = String::default();

                    loop {
                        let count = reader.read_line(&mut line).map_err(|e| {
                            Error::raise_error(format!(
                                "can not read line from file {:?}: {:?}",
                                file, e
                            ))
                        })?;

                        if count > 0 {
                            if let Some(stock_number) = convert_line_to_stock_number(line.trim()) {
                                ids.push(stock_number);
                            } else if debug {
                                eprintln!("{} is not a valid stock number!", line.trim());
                            }
                            line.clear();
                        } else {
                            break;
                        }
                    }
                    ret = Ok(Some(ids));
                }
                ret
            },
        )?
        .then(VecStore);

    parser.init()?;
    parser.parse(aopt::Arc::new(args))?;

    Ok(parser)
}

fn convert_line_to_stock_number(line: &str) -> Option<String> {
    if line.is_empty() && line.len() <= 6 || line.len() == 8 {
        if line.starts_with(STOCK_SHANGHAI) || line.starts_with(STOCK_SHENZHEN) {
            for char in line.chars().skip(2) {
                if !char.is_ascii_digit() {
                    return None;
                }
            }
        } else {
            for char in line.chars() {
                if !char.is_ascii_digit() {
                    return None;
                }
            }
        }

        if line.len() == 8 {
            return Some(line.to_owned());
        } else {
            return Some(normalize_stock_number(line));
        }
    } else if line.len() == 9
        && (line.ends_with(&format!(".{}", STOCK_SHANGHAI))
            || line.ends_with(&format!(".{}", STOCK_SHENZHEN)))
    {
        for char in line.chars().rev().skip(3) {
            if !char.is_ascii_digit() {
                return None;
            }
        }
        let splited: Vec<&str> = line.split('.').collect();

        return Some(format!("{}{}", splited[1], splited[0]));
    }
    None
}

fn normalize_stock_number(number: &str) -> String {
    let mut ret = format!("{}{}", "0".repeat(STOCK_NUMBER_LEN - number.len()), number);

    if let Some(header) = ret.get(0..2) {
        match header {
            "68" | "60" => {
                ret = format!("{}{}", STOCK_SHANGHAI, ret);
            }
            "00" | "30" => {
                ret = format!("{}{}", STOCK_SHENZHEN, ret);
            }
            _ => {
                panic!("{} is not a valid stock number", number);
            }
        }
    }
    ret
}

fn display_help<S: Set>(set: &S) -> Result<(), aopt_help::Error> {
    let foot = format!(
        "Create by {} v{}",
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_VERSION")
    );
    let mut app_help = aopt_help::AppHelp::new(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_DESCRIPTION"),
        &foot,
        aopt_help::prelude::Style::default(),
        std::io::stdout(),
    );
    let global = app_help.global_mut();

    global.add_block(Block::new("option", "[OPTION]", "", "OPTION:", ""))?;
    global.add_block(Block::new("args", "[ARGS]", "", "ARGS:", ""))?;
    for opt in set.iter() {
        if opt.mat_style(Style::Pos) {
            global.add_store(
                "args",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::from(opt.r#type().to_string()),
                    opt.force(),
                    true,
                ),
            )?;
        } else if opt.mat_style(Style::Argument)
            || opt.mat_style(Style::Boolean)
            || opt.mat_style(Style::Combined)
        {
            global.add_store(
                "option",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::from(opt.r#type().to_string()),
                    opt.force(),
                    false,
                ),
            )?;
        }
    }

    app_help.display(true)?;

    Ok(())
}
