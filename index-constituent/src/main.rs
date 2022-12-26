mod spyder;

use std::ops::Deref;

use aopt::prelude::*;
use aopt::set::SetCfg;
use aopt::Error;
use cote::prelude::*;
use spyder::cnindex::CNIndex;
use spyder::csindex::CSIndex;
use spyder::Spyder;
use spyder::SpyderIndexData;

const SEARCH_CMD: &str = "search";
const CONS_CMD: &str = "cons";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    let mut cote = Cote::<AFwdPolicy>::default();

    for (opt, alias, help, value) in [
        ("-d=b", "--debug", "Print debug message", None),
        (
            "-n=i",
            "--page-number",
            "Set page number of results",
            Some(ValInitiator::i64(1i64)),
        ),
        ("-a=b", "--all", "Get all the result", None),
        ("-i=b", "--id-only", "Display only id column", None),
        ("-/r=b", "--/reverse", "Reverse the order of result", None),
    ] {
        let pc = cote.add_opt(opt)?.add_alias(alias).set_help(help);

        if let Some(value) = value {
            pc.set_initiator(value);
        }
    }
    cote.add_opt("-t=s!")?
        .add_alias("--type")
        .set_help("Search keyword and list result")
        .on(
            |_: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>| match val.as_str() {
                "CS" | "CN" => Ok(Some(val.take())),
                _ => Err(Error::raise_failure(
                    "The type must be one of [\"CS\", \"CN\"]!".to_string(),
                )),
            },
        )?;
    cote.add_opt("-s=i")?
        .add_alias("--page-size")
        .set_help("Set page size of results")
        .set_value(30i64)
        .on(
            |_: &mut ASet, _: &mut ASer, val: ctx::Value<i64>| match val.deref() {
                5 | 10 | 20 | 30 => Ok(Some(val)),
                _ => Err(Error::raise_failure(
                    "The page size must be one of [5, 10, 20, 30]!".to_string(),
                )),
            },
        )?;
    cote.add_opt("search=c")?
        .set_help("Search keyword and list result");
    cote.add_opt("cons=c")?
        .set_help("Get and list constituents of given code");
    // add a pos to last, but don't use it
    cote.add_opt("args=p@2")?
        .set_assoc(Assoc::Str)
        .set_help("Argument of operate, such as keyword of search");
    Ok(cote
        .run_async_mut(|ret, app| async move {
            if ret.is_some() {
                let debug: bool = *app.find_val("--debug")?;
                let data = app
                    .find_val::<String>("args")
                    .expect("Which index do you want to list?");
                let all = *app.find_val::<bool>("--all")?;
                let type_ = app.find_val::<String>("--type")?;
                let page_size = *app.find_val::<i64>("--page-size")?;
                let page_number = *app.find_val::<i64>("--page-number")?;
                let mut ctx = SearchCtx::new(app, data, type_)
                    .with_all(all)
                    .with_debug(debug)
                    .with_page_size(page_size as usize)
                    .with_page_number(page_number as usize);
                let mut data = vec![];
                let id_only = *app.find_val::<bool>("--id-only")?;
                let reverse = *app.find_val::<bool>("--/reverse")?;

                loop {
                    let ret = run_command(&ctx).await?;

                    if ret.is_empty() || !ctx.get_all() {
                        for item in ret.iter() {
                            data.push(item.clone());
                        }
                        break;
                    }
                    if ctx.get_all() {
                        ctx.set_page_number(ctx.get_page_number() + 1);
                    }
                }
                if reverse {
                    data.reverse();
                }
                if id_only {
                    for data in data.iter() {
                        println!("{}", data.code);
                    }
                } else {
                    for data in data.iter() {
                        if ctx.is_search()? {
                            println!("{}\t\t{}\t\t{:02}", data.code, data.name, data.number);
                        } else if ctx.is_cons()? {
                            println!(
                                "{}\t\t{}\t\t{}",
                                data.code,
                                data.name,
                                data.number as f64 / 100.0
                            );
                        }
                    }
                }
            }
            Ok(())
        })
        .await?)
}

#[derive(Clone)]
struct SearchCtx<'a, 'b, 'c, P: Policy> {
    cote: &'a Cote<P>,
    arg: &'b str,
    all: bool,
    type_: &'c str,
    debug: bool,
    page_size: usize,
    page_number: usize,
}

impl<'a, 'b, 'c, P: Policy> SearchCtx<'a, 'b, 'c, P>
where
    P: Policy<Error = Error>,
    P::Set: OptValidator + Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    pub fn new(cote: &'a Cote<P>, arg: &'b str, type_: &'c str) -> Self {
        Self {
            cote,
            arg,
            all: false,
            type_,
            debug: false,
            page_size: 0,
            page_number: 0,
        }
    }

    pub fn with_all(mut self, all: bool) -> Self {
        self.all = all;
        self
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    pub fn with_page_number(mut self, page_number: usize) -> Self {
        self.page_number = page_number;
        self
    }

    pub fn get_arg(&self) -> &'b str {
        self.arg
    }

    pub fn get_all(&self) -> bool {
        self.all
    }

    pub fn get_type_(&self) -> &'c str {
        self.type_
    }

    pub fn get_debug(&self) -> bool {
        self.debug
    }

    pub fn get_page_size(&self) -> usize {
        self.page_size
    }

    pub fn get_page_number(&self) -> usize {
        self.page_number
    }

    pub fn is_search(&self) -> Result<bool, Error> {
        Ok(*self.cote.find_val::<bool>(SEARCH_CMD)?)
    }

    pub fn is_cons(&self) -> Result<bool, Error> {
        Ok(*self.cote.find_val::<bool>(CONS_CMD)?)
    }

    // pub fn set_set(&mut self, set: &'a dyn Set)  {
    //     self.set = set;
    // }

    // pub fn set_arg(&mut self, arg: &'b str)  {
    //     self.arg = arg;
    // }

    // pub fn set_all(&mut self, all: bool)  {
    //     self.all = all;
    // }

    // pub fn set_type_(&mut self, type_: &'c str)  {
    //     self.type_ = type_;
    // }

    // pub fn set_debug(&mut self, debug: bool)  {
    //     self.debug = debug;
    // }

    // pub fn set_page_size(&mut self, page_size: usize)  {
    //     self.page_size = page_size;
    // }

    pub fn set_page_number(&mut self, page_number: usize) {
        self.page_number = page_number;
    }
}

async fn run_command<'a, 'b, 'c, P: Policy>(
    ctx: &SearchCtx<'a, 'b, 'c, P>,
) -> Result<SpyderIndexData, Error>
where
    P: Policy<Error = Error>,
    P::Set: OptValidator + Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    if ctx.is_search()? {
        let ret = search_keyword(ctx).await?;

        if ctx.get_debug() {
            dbg!(&ret);
        }
        Ok(ret)
    } else if ctx.is_cons()? {
        let ret = display_cons_of(ctx).await?;

        if ctx.get_debug() {
            dbg!(&ret);
        }

        Ok(ret)
    } else {
        panic!("????????????")
    }
}

async fn search_keyword<'a, 'b, 'c, P: Policy>(
    ctx: &SearchCtx<'a, 'b, 'c, P>,
) -> Result<SpyderIndexData, Error>
where
    P: Policy<Error = Error>,
    P::Set: OptValidator + Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    match ctx.get_type_() {
        "CS" => {
            let csspyder = CSIndex::new(ctx.get_debug(), ctx.get_page_size() as usize)
                .map_err(|e| Error::raise_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = csspyder
                .search(ctx.get_arg(), ctx.get_page_number() as usize)
                .await
                .map_err(|e| {
                    Error::raise_error(format!("Can not search {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
        "CN" => {
            let cnspyder = CNIndex::new(ctx.get_debug(), ctx.get_page_size() as usize)
                .map_err(|e| Error::raise_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = cnspyder
                .search(ctx.get_arg(), ctx.get_page_number() as usize)
                .await
                .map_err(|e| {
                    Error::raise_error(format!("Can not search {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
        type_ => {
            panic!("Unknow search type {}", type_)
        }
    }
}

async fn display_cons_of<'a, 'b, 'c, P: Policy>(
    ctx: &SearchCtx<'a, 'b, 'c, P>,
) -> Result<SpyderIndexData, Error>
where
    P: Policy<Error = Error>,
    P::Set: OptValidator + Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    match ctx.get_type_() {
        "CS" => {
            let csspyder = CSIndex::new(ctx.get_debug(), ctx.get_page_size() as usize)
                .map_err(|e| Error::raise_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = csspyder
                .fetch_cons(ctx.get_arg(), ctx.get_page_number() as usize)
                .await
                .map_err(|e| {
                    Error::raise_error(format!("Can not fetch cons {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
        "CN" => {
            let cnspyder = CNIndex::new(ctx.get_debug(), ctx.get_page_size() as usize)
                .map_err(|e| Error::raise_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = cnspyder
                .fetch_cons(ctx.get_arg(), ctx.get_page_number() as usize)
                .await
                .map_err(|e| {
                    Error::raise_error(format!("Can not fetch cons {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
        type_ => {
            panic!("Unknow search type {}", type_)
        }
    }
}
