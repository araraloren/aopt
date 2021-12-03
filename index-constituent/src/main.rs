mod spyder;

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
use spyder::cnindex::CNIndex;
use spyder::csindex::CSIndex;
use spyder::Spyder;
use spyder::SpyderConsData;
use spyder::SpyderIndexData;

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
            let noa = app.get_parser().get_noa();

            if !ret || (noa.len() < 2) {
                print_help(set)?;
            } else {
                let data = noa[1];
                let debug = value_of(set, "--debug")?.as_bool().unwrap_or(&false);

                if *debug {
                    dbg!(set);
                }
                if !print_help(set)? {
                    if *value_of(set, SEARCH_CMD)?.as_bool().unwrap_or(&false) {
                        let ret = search_keyword(set, data.as_str(), *debug).await?;

                        if *debug {
                            dbg!(&ret);
                        }
                        for item in ret.as_slice() {
                            println!("{}", item.code);
                        }
                    } else if *value_of(set, CONS_CMD)?.as_bool().unwrap_or(&false) {
                        let ret = display_cons_of(set, data.as_str(), *debug).await?;

                        if *debug {
                            dbg!(&ret);
                        }
                        for item in ret.as_slice() {
                            println!("{}", item.code);
                        }
                    }
                }
            }
            Ok(())
        })
        .await?)
}

async fn search_keyword(set: &dyn Set, arg: &str, debug: bool) -> Result<SpyderIndexData> {
    let string = String::default();
    let type_ = value_of(set, "--type")?
        .as_str()
        .unwrap_or(&string)
        .as_str();
    let page_size = value_of(set, "--page-size")?.as_int().unwrap_or(&30);
    let page_number = value_of(set, "--page-number")?.as_int().unwrap_or(&1);
    match type_ {
        "CS" => {
            let csspyder = CSIndex::new(debug, *page_size as usize)
                .map_err(|e| create_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = csspyder
                .search(arg, *page_number as usize)
                .await
                .map_err(|e| create_error(format!("Can not search {}: {:?}", arg, e)))?;

            Ok(ret)
        }
        "CN" => {
            let cnspyder = CNIndex::new(debug, *page_size as usize)
                .map_err(|e| create_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = cnspyder
                .search(arg, *page_number as usize)
                .await
                .map_err(|e| create_error(format!("Can not search {}: {:?}", arg, e)))?;

            Ok(ret)
        }
        _ => {
            panic!("Unknow search type {}", type_)
        }
    }
}

async fn display_cons_of(set: &dyn Set, arg: &str, debug: bool) -> Result<SpyderConsData> {
    let string = String::default();
    let type_ = value_of(set, "--type")?
        .as_str()
        .unwrap_or(&string)
        .as_str();
    let page_size = value_of(set, "--page-size")?.as_int().unwrap_or(&30);
    let page_number = value_of(set, "--page-number")?.as_int().unwrap_or(&1);
    match type_ {
        "CS" => {
            let csspyder = CSIndex::new(debug, *page_size as usize)
                .map_err(|e| create_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = csspyder
                .fetch_cons(arg, *page_number as usize)
                .await
                .map_err(|e| create_error(format!("Can not fetch cons {}: {:?}", arg, e)))?;

            Ok(ret)
        }
        "CN" => {
            let cnspyder = CNIndex::new(debug, *page_size as usize)
                .map_err(|e| create_error(format!("Can not init CSIndexSpyder: {:?}", e)))?;

            let ret = cnspyder
                .fetch_cons(arg, *page_number as usize)
                .await
                .map_err(|e| create_error(format!("Can not fetch cons {}: {:?}", arg, e)))?;

            Ok(ret)
        }
        _ => {
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
