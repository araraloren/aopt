mod spyder;

use cote::prelude::*;
use cote::Error;
use spyder::cnindex::CNIndex;
use spyder::csindex::CSIndex;
use spyder::Spyder;
use spyder::SpyderIndexData;

#[derive(Debug, Cote)]
#[cote(help, aborthelp)]
pub struct Stock {
    /// Print debug message
    #[arg(alias = "-d")]
    debug: bool,

    /// Set the page number of results
    #[arg(alias = "-n", force = false, value = 1)]
    page_number: i64,

    /// Get all the results
    #[arg(alias = "-a")]
    all: bool,

    /// Display on id column
    #[arg(alias = "-i")]
    id_only: bool,

    /// Reverse the order of results
    #[arg(name = "--/reverse", alias = "-/r")]
    reverse: bool,

    /// Set the search type
    #[arg(name = "--type", alias = "-t")]
    type_: SearchType,

    /// Set page size of results
    #[arg(alias = "-s", value = 30,
    force = false, valid = valid!([5, 10, 20, 30]))]
    page_size: i64,

    /// Search keyword and list result
    #[arg(alias = "se")]
    search: Cmd,

    /// Get and list constituents of given code
    #[arg(alias = "co")]
    cons: Cmd,

    /// Argument of operator, such as keyword of search sub command
    #[pos(index = 2)]
    args: String,
}

#[derive(Debug, Clone, Copy, CoteOpt, CoteVal)]
pub enum SearchType {
    CS,
    CN,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    let stock = Stock::parse_env()?;

    let mut ctx = SearchCtx::new(&stock, &stock.args, stock.type_)
        .with_all(stock.all)
        .with_debug(stock.debug)
        .with_page_size(stock.page_size as _)
        .with_page_number(stock.page_number as _);
    let mut data = vec![];
    let id_only = stock.id_only;
    let reverse = stock.reverse;

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

    Ok(())
}

#[derive(Clone)]
struct SearchCtx<'a, 'b> {
    stock: &'a Stock,
    arg: &'b str,
    all: bool,
    type_: SearchType,
    debug: bool,
    page_size: usize,
    page_number: usize,
}

impl<'a, 'b> SearchCtx<'a, 'b> {
    pub fn new(cote: &'a Stock, arg: &'b str, type_: SearchType) -> Self {
        Self {
            stock: cote,
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

    pub fn get_type_(&self) -> SearchType {
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
        Ok(self.stock.search.0)
    }

    pub fn is_cons(&self) -> Result<bool, Error> {
        Ok(self.stock.cons.0)
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

async fn run_command(ctx: &SearchCtx<'_, '_>) -> Result<SpyderIndexData, Error> {
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

async fn search_keyword(ctx: &SearchCtx<'_, '_>) -> Result<SpyderIndexData, Error> {
    match ctx.get_type_() {
        SearchType::CS => {
            let csspyder = CSIndex::new(ctx.get_debug(), ctx.get_page_size())
                .map_err(|e| Error::raise_error(format!("can not init CSIndexSpyder: {:?}", e)))?;

            let ret = csspyder
                .search(ctx.get_arg(), ctx.get_page_number())
                .await
                .map_err(|e| {
                    Error::raise_error(format!("can not search {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
        SearchType::CN => {
            let cnspyder = CNIndex::new(ctx.get_debug(), ctx.get_page_size())
                .map_err(|e| Error::raise_error(format!("can not init CSIndexSpyder: {:?}", e)))?;

            let ret = cnspyder
                .search(ctx.get_arg(), ctx.get_page_number())
                .await
                .map_err(|e| {
                    Error::raise_error(format!("can not search {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
    }
}

async fn display_cons_of(ctx: &SearchCtx<'_, '_>) -> Result<SpyderIndexData, Error> {
    match ctx.get_type_() {
        SearchType::CS => {
            let csspyder = CSIndex::new(ctx.get_debug(), ctx.get_page_size())
                .map_err(|e| Error::raise_error(format!("can not init CSIndexSpyder: {:?}", e)))?;

            let ret = csspyder
                .fetch_cons(ctx.get_arg(), ctx.get_page_number())
                .await
                .map_err(|e| {
                    Error::raise_error(format!("can not fetch cons {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
        SearchType::CN => {
            let cnspyder = CNIndex::new(ctx.get_debug(), ctx.get_page_size())
                .map_err(|e| Error::raise_error(format!("can not init CSIndexSpyder: {:?}", e)))?;

            let ret = cnspyder
                .fetch_cons(ctx.get_arg(), ctx.get_page_number())
                .await
                .map_err(|e| {
                    Error::raise_error(format!("can not fetch cons {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
    }
}
