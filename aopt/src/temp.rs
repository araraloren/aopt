use aopt::prelude::*;
use aopt::Error;
use tokio::spawn;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut loader = APreParser::default();
    let config_options = vec![];

    loader
        .add_opt("--depth=b")?
        .add_alias("-d")
        .set_help("Print debug message");

    // load config name to loader
    getopt!(std::env::args().skip(1), &mut loader)?;

    let ret_value = loader.take_retval().unwrap_or_default();
    let debug = *loader.find_val("--debug")?;
    let display_help = *loader.find_val("--help")?;
    let verbose = *loader.find_val("--verbose")?;
    let mut finder = AFwdParser::default();

    finder
        .add_opt("--whole=s")?
        .add_alias("-w")
        .set_help("Extension category: match whole filename");
    finder
        .add_opt("path=p@*")?
        .set_hint("[file or directory]+");

    let (sender, mut receiver) = channel::<Option<String>>(23);
    let ret = getopt!(ret_value.into_args().into_iter(), &mut finder); // parsing option from left arguments
    let has_sepcial_error = if let Err(e) = &ret {
        e.is_failure()
    } else {
        false
    };
    let no_option_matched = if let Ok(opt) = &ret {
        opt.is_none()
    } else {
        false
    };
    if has_sepcial_error || no_option_matched || display_help {
        if has_sepcial_error {
            eprintln!("{}\n", ret.unwrap_err());
        }
        return Ok(());
    }
    spawn(find_given_ext_in_directory(
        config_options,
        sender,
        finder,
        debug,
        verbose,
    ));
    while let Some(Some(data)) = receiver.recv().await {
        println!("{}", data);
    }

    Ok(())
}

async fn find_given_ext_in_directory(
    options: Vec<String>,
    sender: Sender<Option<String>>,
    parser: AFwdParser,
    debug: bool,
    verbose: bool,
) -> Result<(), Error> {
    let mut whos = HashSet::<String>::default();
    let mut exts = HashSet::<String>::default();

    let default_value = vec![];
    let only = parser.find_val::<String>("--only")?.as_str();
    let exclude = parser
        .find_vals::<String>("--exclude")
        .unwrap_or(&default_value);
    let ex_exts = parser.find_vals::<String>("--Extension");
    let ex_whos = parser.find_vals::<String>("--Whole");
    let whole = parser.find_vals::<String>("--whole");
    let extension = parser.find_vals::<String>("--extension");

    let ignore_case = *parser.find_val("--ignore-case")?;

    let only_checker = |name1: &str, name2: &str| -> bool { only.eq(name1) || only.eq(name2) };
    let exclude_checker = move |name1: &str, name2: &str| -> bool {
        exclude.iter().any(|v| v.eq(name1) || v.eq(name2))
    };

    if only_checker("whole", "w") && !exclude_checker("whole", "w") {
        if let Ok(whole) = whole {
            for ext in whole {
                whos.insert(ext.clone());
            }
        }
    }
    if only_checker("extension", "e") && !exclude_checker("extension", "e") {
        if let Ok(extension) = extension {
            for ext in extension {
                whos.insert(ext.clone());
            }
        }
    }
    for opt in options {
        if only_checker(opt.as_str(), "") && !exclude_checker(opt.as_str(), "") {
            if let Ok(opt_exts) = parser.find_vals::<String>(opt.as_str()) {
                for ext in opt_exts {
                    exts.insert(ext.clone());
                }
            }
        }
    }
    if let Ok(ex_exts) = ex_exts {
        for ext in ex_exts {
            exts.remove(ext);
        }
    }
    if let Ok(ex_whos) = ex_whos {
        for ext in ex_whos {
            whos.remove(ext);
        }
    }
    if ignore_case {
        exts = exts.into_iter().map(|v| v.to_lowercase()).collect();
        whos = whos.into_iter().map(|v| v.to_lowercase()).collect();
    }
    if debug {
        eprintln!("match whole filename : {:?}", whos);
        eprintln!("match file extension : {:?}", exts);
    }
    if whos.is_empty() && exts.is_empty() {
        println!("What extension or filename do you want search, try command: fs -? or fs --help",);
        return Ok(());
    }
    sender.send(None).await.unwrap();
    Ok(())
}
