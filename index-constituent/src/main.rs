mod spyder;

use std::io::Stdout;

use aopt::app::SingleApp;
use aopt::err::create_error;
use aopt::err::Result;
use aopt::prelude::*;
use aopt_help::prelude::*;
use spyder::cnindex::CNIndex;
use spyder::csindex::CSIndex;
use spyder::Spyder;
use spyder::SpyderIndexData;

const SEARCH_CMD: &'static str = "search";
const CONS_CMD: &'static str = "cons";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    color_eyre::install()?;

    let mut app = SingleApp::<SimpleSet, DefaultService, ForwardPolicy>::default();

    for (opt, long, help, value) in [
        ("-h=b", "-help", "Print help message", None),
        ("-d=b", "--debug", "Print debug message", None),
        (
            "-n=i",
            "--page-number",
            "Set page number of results",
            Some(OptValue::from(1i64)),
        ),
        ("-a=b", "--all", "Get all the result", None),
        ("-i=b", "--id-only", "Display only id column", None),
        (
            "-r=b/",
            "--reverse",
            "Reverse the order of result",
            Some(OptValue::from(true)),
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

    // add a pos to last, but don't use it
    app.add_opt("args=p@-1")?
        .set_help("Argument of operate, such as keyword of search")
        .commit()?;

    Ok(app
        .run_async(std::env::args().skip(1), |ret, app| async move {
            let parser = app.get_parser();
            let noa = parser.get_service().get_noa();
            let set = parser.get_set();
            let debug = value_of(set, "--debug")?.as_bool().unwrap_or(&false);

            if *debug {
                dbg!(&noa);
            }
            if !ret || (noa.len() < 2) {
                print_help(set, true)?;
            } else {
                let string = String::default();
                let data = noa[1];
                let all = value_of(set, "--all")?.as_bool().unwrap_or(&false);
                let type_ = value_of(set, "--type")?.as_str().unwrap_or(&string);
                let mut ctx = SearchCtx::new(set, data.as_str(), type_)
                    .with_all(*all)
                    .with_debug(*debug);
                let page_size = value_of(set, "--page-size")?.as_int();
                let page_number = value_of(set, "--page-number")?.as_int();

                match type_.as_str() {
                    "CS" => {
                        ctx = ctx
                            .with_page_size(*page_size.unwrap_or(&30) as usize)
                            .with_page_number(*page_number.unwrap_or(&1) as usize);
                    }
                    "CN" => {
                        ctx = ctx
                            .with_page_size(*page_size.unwrap_or(&20) as usize)
                            .with_page_number(*page_number.unwrap_or(&1) as usize);
                    }
                    _ => {}
                }
                if *debug {
                    dbg!(&ctx);
                }
                if !print_help(set, false)? {
                    let mut data = vec![];
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

                    let id_only = *value_of(set, "--id-only")?.as_bool().unwrap_or(&false);
                    let reverse = *value_of(set, "--reverse")?.as_bool().unwrap_or(&false);

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
            }
            Ok(())
        })
        .await?)
}

#[derive(Debug, Clone)]
struct SearchCtx<'a, 'b, 'c> {
    set: &'a dyn Set,
    arg: &'b str,
    all: bool,
    type_: &'c str,
    debug: bool,
    page_size: usize,
    page_number: usize,
}

impl<'a, 'b, 'c> SearchCtx<'a, 'b, 'c> {
    pub fn new(set: &'a dyn Set, arg: &'b str, type_: &'c str) -> Self {
        Self {
            set,
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

    pub fn is_search(&self) -> Result<bool> {
        Ok(*value_of(self.set, SEARCH_CMD)?.as_bool().unwrap_or(&false))
    }

    pub fn is_cons(&self) -> Result<bool> {
        Ok(*value_of(self.set, CONS_CMD)?.as_bool().unwrap_or(&false))
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

async fn run_command<'a, 'b, 'c>(ctx: &SearchCtx<'a, 'b, 'c>) -> Result<SpyderIndexData> {
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

async fn search_keyword<'a, 'b, 'c>(ctx: &SearchCtx<'a, 'b, 'c>) -> Result<SpyderIndexData> {
    match ctx.get_type_() {
        "CS" => {
            let csspyder = CSIndex::new(ctx.get_debug(), ctx.get_page_size() as usize)
                .map_err(|e| create_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = csspyder
                .search(ctx.get_arg(), ctx.get_page_number() as usize)
                .await
                .map_err(|e| create_error(format!("Can not search {}: {:?}", ctx.get_arg(), e)))?;

            Ok(ret)
        }
        "CN" => {
            let cnspyder = CNIndex::new(ctx.get_debug(), ctx.get_page_size() as usize)
                .map_err(|e| create_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = cnspyder
                .search(ctx.get_arg(), ctx.get_page_number() as usize)
                .await
                .map_err(|e| create_error(format!("Can not search {}: {:?}", ctx.get_arg(), e)))?;

            Ok(ret)
        }
        type_ => {
            panic!("Unknow search type {}", type_)
        }
    }
}

async fn display_cons_of<'a, 'b, 'c>(ctx: &SearchCtx<'a, 'b, 'c>) -> Result<SpyderIndexData> {
    match ctx.get_type_() {
        "CS" => {
            let csspyder = CSIndex::new(ctx.get_debug(), ctx.get_page_size() as usize)
                .map_err(|e| create_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = csspyder
                .fetch_cons(ctx.get_arg(), ctx.get_page_number() as usize)
                .await
                .map_err(|e| {
                    create_error(format!("Can not fetch cons {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
        "CN" => {
            let cnspyder = CNIndex::new(ctx.get_debug(), ctx.get_page_size() as usize)
                .map_err(|e| create_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = cnspyder
                .fetch_cons(ctx.get_arg(), ctx.get_page_number() as usize)
                .await
                .map_err(|e| {
                    create_error(format!("Can not fetch cons {}: {:?}", ctx.get_arg(), e))
                })?;

            Ok(ret)
        }
        type_ => {
            panic!("Unknow search type {}", type_)
        }
    }
}

fn value_of<'a>(set: &'a dyn Set, opt: &str) -> Result<&'a OptValue> {
    Ok(set
        .find(opt)?
        .ok_or(create_error(format!("can not get option {}", opt)))?
        .get_value())
}

fn print_help(set: &dyn Set, force: bool) -> Result<bool> {
    let mut is_need_help = false;

    if let Ok(Some(opt)) = set.find("help") {
        if opt.get_value().as_bool() == Some(&true) {
            is_need_help = true;
        }
    }
    if is_need_help || force {
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

    let version = gstr(&format!(
        "Create by araraloren {}",
        env!("CARGO_PKG_VERSION")
    ));
    let global = help.store.get_global_mut();

    global.set_header(gstr("Search and list index constituents"));
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
                        opt.get_type_name(),
                        opt.get_optional(),
                    ));
                }
            }

            search_cmd.commit();
        }
    }

    help
}
