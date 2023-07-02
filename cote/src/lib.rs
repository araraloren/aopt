#![doc = include_str!("../README.md")]
pub mod _reference;
pub mod help;
pub mod meta;
pub mod parser;
pub mod rctx;
pub mod valid;
pub mod value;

use std::marker::PhantomData;

pub use aopt;
pub use aopt_help;
pub use cote_derive;

pub use aopt::ext::ctx;
pub use aopt::opt::Cmd;
pub use aopt::opt::Main;
pub use aopt::opt::Pos;
pub use aopt::parser::UserStyle;
pub use aopt::prelude::ctor_default_name;
pub use aopt::prelude::APolicyExt;
pub use aopt::prelude::ARef;
pub use aopt::prelude::ASer;
pub use aopt::prelude::ASet;
pub use aopt::prelude::Action;
pub use aopt::prelude::Args;
pub use aopt::prelude::Commit;
pub use aopt::prelude::Config;
pub use aopt::prelude::ConfigValue;
pub use aopt::prelude::Ctor;
pub use aopt::prelude::Ctx;
pub use aopt::prelude::DefaultSetChecker;
pub use aopt::prelude::ErasedTy;
pub use aopt::prelude::ErasedValue;
pub use aopt::prelude::Extract;
pub use aopt::prelude::FilterMatcher;
pub use aopt::prelude::Handler;
pub use aopt::prelude::HandlerCollection;
pub use aopt::prelude::Index;
pub use aopt::prelude::Infer;
pub use aopt::prelude::Information;
pub use aopt::prelude::InitializeValue;
pub use aopt::prelude::Invoker;
pub use aopt::prelude::Match;
pub use aopt::prelude::Opt;
pub use aopt::prelude::OptParser;
pub use aopt::prelude::OptValidator;
pub use aopt::prelude::OptValueExt;
pub use aopt::prelude::Policy;
pub use aopt::prelude::PolicyParser;
pub use aopt::prelude::PolicySettings;
pub use aopt::prelude::Process;
pub use aopt::prelude::RawValParser;
pub use aopt::prelude::ReturnVal;
pub use aopt::prelude::ServicesExt;
pub use aopt::prelude::ServicesValExt;
pub use aopt::prelude::Set;
pub use aopt::prelude::SetCfg;
pub use aopt::prelude::SetChecker;
pub use aopt::prelude::SetExt;
pub use aopt::prelude::SetValueFindExt;
pub use aopt::prelude::Store;
pub use aopt::prelude::ValInitializer;
pub use aopt::prelude::ValStorer;
pub use aopt::prelude::ValValidator;
pub use aopt::raise_error;
pub use aopt::raise_failure;
pub use aopt::Error as CoteError;
pub use aopt::GetoptRes;
pub use aopt::RawVal;
pub use aopt::Uid;
pub use cote_derive::Cote;
pub use aopt::prelude::VecStore;

pub use help::display_set_help;
pub use help::HelpDisplayCtx;
pub use meta::IntoConfig;
pub use meta::OptionMeta;
pub use parser::Parser;
pub use rctx::FailedInfo;
pub use rctx::RunningCtx;
pub use value::InferValueMut;

pub trait IntoParserDerive<'inv, Set, Ser>
where
    Ser: ServicesValExt + Default,
    SetCfg<Set>: Config + ConfigValue + Default,
    Set: crate::Set + OptParser + OptValidator + Default,
{
    fn into_parser() -> Result<Parser<'inv, Set, Ser>, aopt::Error> {
        let mut parser = Parser::default();
        Self::update(&mut parser)?;
        Ok(parser)
    }
    fn update(parser: &mut Parser<'inv, Set, Ser>) -> Result<(), aopt::Error>;
}

pub trait ExtractFromSetDerive<'set, Set: SetValueFindExt> {
    fn try_extract(set: &'set mut Set) -> Result<Self, aopt::Error>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoteRes<P, Policy>
where
    Policy: crate::Policy,
{
    pub policy: P,

    pub ret: Policy::Ret,

    pub parser: Policy::Set,
}

pub trait ReturnValStatus {
    fn status(&self) -> bool;
}

impl ReturnValStatus for ReturnVal {
    fn status(&self) -> bool {
        ReturnVal::status(&self)
    }
}

pub type PrePolicy<'inv, Set, Ser> = aopt::prelude::PrePolicy<
    Parser<'inv, Set, Ser>,
    Ser,
    DefaultSetChecker<Parser<'inv, Set, Ser>>,
>;

pub type FwdPolicy<'inv, Set, Ser> = aopt::prelude::FwdPolicy<
    Parser<'inv, Set, Ser>,
    Ser,
    DefaultSetChecker<Parser<'inv, Set, Ser>>,
>;

pub type DelayPolicy<'inv, Set, Ser> = aopt::prelude::DelayPolicy<
    Parser<'inv, Set, Ser>,
    Ser,
    DefaultSetChecker<Parser<'inv, Set, Ser>>,
>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NullPolicy<Set, Ser>(PhantomData<(Set, Ser)>);

impl<Set, Ser> Default for NullPolicy<Set, Ser> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Set, Ser> Policy for NullPolicy<Set, Ser> {
    type Ret = aopt::parser::ReturnVal;

    type Set = Set;

    type Inv<'a> = aopt::prelude::Invoker<'a, Set, Ser>;

    type Ser = Ser;

    type Error = aopt::Error;

    fn parse<'a>(
        &mut self,
        _: &mut Self::Set,
        _: &mut Self::Inv<'a>,
        _: &mut Self::Ser,
        _: ARef<Args>,
    ) -> Result<Self::Ret, Self::Error> {
        Ok(ReturnVal::default())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_example_simple() {
        use crate::*;
        use crate as cote;
        // macro generate the code depend on crate name
        use aopt::opt::Pos;

        #[derive(Debug, Cote)]
        pub struct Example {
            /// a flag argument
            foo: bool,

            /// a position argument
            #[arg(index = "1")]
            bar: Pos<usize>,
        }

        let example = Example::parse(Args::from_array(["app", "--foo", "42"]));

        assert!(example.is_ok());

        let example = example.unwrap();

        assert_eq!(example.foo, true);
        assert_eq!(example.bar.0, 42);

        let parser = Example::into_parser().unwrap();

        assert_eq!(parser["--foo"].help(), &aopt::astr("a flag argument"));
        assert_eq!(parser["bar"].help(), &aopt::astr("a position argument"));
    }

    #[test]
    fn test_multiple_pos_arguments() {
        use crate::*;
        // macro generate the code depend on crate name
        use crate as cote;
        use aopt::opt::Pos;
        use std::path::PathBuf;

        #[derive(Debug, Cote)]
        #[cote(help)]
        pub struct CopyTool {
            #[arg(alias = "-f")]
            force: bool,

            /// Enable the recursive mode
            #[arg(alias = "-r")]
            recursive: bool,

            #[arg(index = "1", help = "The copy destination")]
            destination: Pos<String>,

            /// Specify path to copy
            #[arg(index = "2..")]
            sources: Vec<Pos<PathBuf>>,
        }

        let example = CopyTool::parse(Args::from_array(["app", "--force"]));

        assert!(example.is_err());

        let example = CopyTool::parse(Args::from_array([
            "app", "--force", ".", "../foo", "../bar/", "other",
        ]))
        .unwrap();

        assert_eq!(example.force, true);
        assert_eq!(example.recursive, false);
        assert_eq!(example.destination.0, String::from("."));
        assert_eq!(
            example.sources,
            ["../foo", "../bar/", "other"]
                .into_iter()
                .map(|v| Pos::new(PathBuf::from(v)))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_fallback() {
        use crate::*;
        // macro generate the code depend on crate name
        use crate as cote;
        use aopt::opt::Pos;

        #[derive(Debug, Cote)]
        #[cote(policy = delay, help, on = find_main, name = "find")]
        pub struct Find {
            /// Do not follow symbolic link
            #[arg(name = "-H", nodelay)]
            hard: bool,

            /// Fllow symbolic link
            #[arg(name = "-L", nodelay)]
            symbol: bool,

            #[arg(name = "-P", nodelay, value = true)]
            never: bool,

            #[arg(name = "-name", help = "Search the file base on file name")]
            name: Option<String>,

            /// List the file large than the size
            #[arg(name = "-size")]
            size: Option<usize>,

            #[arg(index = "1", help = "Search starting point", fallback = search, then = VecStore)]
            destination: Vec<Pos<String>>,
        }

        fn search<Set, Ser>(
            _: &mut Set,
            _: &mut Ser,
        ) -> Result<Option<Vec<String>>, aopt::Error> {
            Ok(Some(
                ["file1", "file2", "dir1", "dir2"]
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect(),
            ))
        }

        fn find_main<Set, Ser>(set: &mut Set, _: &mut Ser) -> Result<Option<()>, aopt::Error>
        where Set: SetValueFindExt,
        {
            let tool = Find::try_extract(set)?;

            assert_eq!(tool.hard, true);
            assert_eq!(tool.symbol, false);
            assert_eq!(tool.never, true);
            assert_eq!(tool.name, Some("foo".to_owned()));
            assert_eq!(tool.size, Some(42));
            assert_eq!(
                tool.destination,
                ["file1", "file2", "dir1", "dir2"]
                    .into_iter()
                    .map(|v| Pos::new(v.to_string()))
                    .collect::<Vec<_>>()
            );

            Ok(Some(()))
        }

        let args = Args::from_array(["app", ".", "-H", "-name=foo", "-size", "42"]);

        let CoteRes { ret, .. } = Find::parse_args(args).unwrap();

        ret.ok().unwrap();
    }

    #[test]
    fn sub_test() {
        assert!(sub_test_impl().is_ok());
    }

    fn sub_test_impl() -> Result<(), aopt::Error> {
        use crate::*;
        // macro generate the code depend on crate name
        use crate as cote;
        use std::path::PathBuf;

        #[derive(Debug, Cote, PartialEq, Default)]
        #[cote(policy = pre, help, name = "app")]
        pub struct App {
            /// Set the count value
            #[arg(values = [1usize, 2, 3])]
            count: Option<Vec<usize>>,

            #[sub(alias = "ls", help = "list subcommand list file of given path")]
            list: Option<List>,

            #[sub(help = "find something under directory")]
            find: Option<Find>,
        }

        #[derive(Debug, Cote, PartialEq)]
        #[cote(help)]
        pub struct List {
            #[arg(help = "list all the file")]
            all: bool,

            #[arg(valid = valid!([1, 42, 68]))]
            depth: usize,

            #[arg(index = "1")]
            path: Pos<PathBuf>,
        }

        #[derive(Debug, Cote, PartialEq)]
        #[cote(help)]
        pub struct Find {
            recursive: bool,

            #[arg(index = "1")]
            path: Pos<PathBuf>,
        }

        let args = Args::from_array(["app", "ls", "--all", "--depth=42", "."]);

        let app = App::parse(args)?;

        assert_eq!(
            app,
            App {
                count: Some(vec![1, 2, 3]),
                list: Some(List {
                    all: true,
                    depth: 42,
                    path: Pos(PathBuf::from("."))
                }),
                find: None,
            }
        );

        let args = Args::from_array(["app", "list", "--all", "--depth=6", "."]);

        let app = App::parse(args);

        assert!(app.is_err());

        let args = Args::from_array(["app", "--count=8", "find", "something"]);

        let app = App::parse(args)?;

        assert_eq!(
            app,
            App {
                count: Some(vec![1, 2, 3, 8]),
                list: None,
                find: Some(Find {
                    recursive: false,
                    path: Pos(PathBuf::from("something")),
                }),
            }
        );

        let args = Args::from_array(["app", "--count", "42"]);

        let app = App::parse(args);

        assert!(app.is_err());

        let args = Args::from_array(["app", "--count=42", "list"]);

        let CoteRes {
            ret,
            parser: mut app,
            ..
        } = App::parse_args(args)?;

        assert_eq!(ret.status(), false);
        assert_eq!(
            app.extract_type::<App>()?,
            App {
                count: Some(vec![1, 2, 3, 42]),
                list: None,
                find: None,
            }
        );

        Ok(())
    }
}
