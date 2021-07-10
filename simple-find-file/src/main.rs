use getopt_rs::opt::nonopt::*;
use getopt_rs::opt::opt::*;
use getopt_rs::parser::{ForwardParser, Parser};
use getopt_rs::set::{Set, SimpleSet};
use getopt_rs::uid::UidGenerator;

fn main() {
    let mut set = SimpleSet::default();
    let mut parser = ForwardParser::<UidGenerator>::default();

    set.add_creator(Box::new(IntCreator::default()));
    set.add_creator(Box::new(BoolCreator::default()));
    set.add_creator(Box::new(StrCreator::default()));
    set.add_creator(Box::new(FltCreator::default()));
    set.add_creator(Box::new(UintCreator::default()));
    set.add_creator(Box::new(ArrayCreator::default()));
    set.add_creator(Box::new(CmdCreator::default()));
    set.add_creator(Box::new(PosCreator::default()));
    set.add_creator(Box::new(MainCreator::default()));
    set.add_prefix(String::from("--"));
    set.add_prefix(String::from("-"));

    getopt_rs::tools::initialize_log().unwrap();

    if let Ok(mut commit) = set.add_opt("cpp=c") {
        commit.set_help("run in cpp mode".to_string());
        commit.commit().unwrap();
    }
    if let Ok(mut commit) = set.add_opt("c=c") {
        commit.set_help("run in c mode".to_string());
        commit.commit().unwrap();
    }
    if let Ok(mut commit) = set.add_opt("-S=b") {
        commit.set_help("pass -S to compiler.".to_string());
        commit.commit().unwrap();
    }
    if let Ok(mut commit) = set.add_opt("-E=b") {
        commit.set_help("pass -E to compiler.".to_string());
        commit.commit().unwrap();
    }
    if let Ok(mut commit) = set.add_opt("-D=s") {
        commit.set_help("pass -D<value> to compiler.".to_string());
        commit.commit().unwrap();
    }

    let mut args = &mut ["c", "a", "ops"].iter().map(|&v| String::from(v));

    let ret = parser.parse(set, &mut std::env::args().skip(1)).unwrap();

    if let Some(ret) = ret {
        dbg!(ret);
    }
}
