use getopt_rs::opt::opt::BoolCreator;
use getopt_rs::opt::opt::IntCreator;
use getopt_rs::parser::{ForwardParser, Parser};
use getopt_rs::set::{Set, SimpleSet};
use getopt_rs::uid::UidGenerator;

fn main() {
    let mut set = SimpleSet::default();
    let mut parser = ForwardParser::<UidGenerator>::default();

    set.add_creator(Box::new(IntCreator::default()));
    set.add_creator(Box::new(BoolCreator::default()));
    set.add_prefix(String::from("--"));
    set.add_prefix(String::from("-"));

    getopt_rs::tools::initialize_log();

    if let Ok(mut commit) = set.add_opt("-c=b") {
        commit.commit().unwrap();
    }

    let ret = parser
        .parse(
            set,
            &mut ["-c", "a", "ops"].iter().map(|&v| String::from(v)),
        )
        .unwrap();

    if let Some(ret) = ret {
        dbg!(ret);
    }
}
