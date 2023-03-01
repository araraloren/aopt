use cote::prelude::aopt;
use cote::prelude::derive::*;

use aopt::opt::Pos;

fn main() -> Result<(), aopt::Error> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    #[derive(Debug, Cote)]
    pub struct Example {
        /// a flag argument
        foo: bool,

        /// a position argument
        #[arg(index = "1")]
        bar: Pos<usize>,
    }

    let example = Example::parse(ARef::new(Args::from_array(["app", "--foo", "42"])))?;

    assert_eq!(example.foo, true);
    assert_eq!(example.bar.0, 42);

    let parser : AFwdParser = Example::into_parser()?;

    assert_eq!(parser["--foo"].help(), &aopt::astr("a flag argument"));
    assert_eq!(parser["bar"].help(), &aopt::astr("a position argument"));

    Ok(())
}
