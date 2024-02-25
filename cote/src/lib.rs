#![doc = include_str!("../README.md")]
pub mod _reference;
pub(crate) mod alter;
pub(crate) mod help;
pub(crate) mod meta;
pub(crate) mod parser;
pub(crate) mod rctx;
pub(crate) mod value;

pub mod valid;

use std::marker::PhantomData;

pub use aopt;
pub use aopt_help;
pub use cote_derive;

use aopt::prelude::OptStyleManager;

pub use aopt::ext::ctx;
pub use aopt::opt::Any;
pub use aopt::opt::Cmd;
pub use aopt::opt::Main;
pub use aopt::opt::MutOpt;
pub use aopt::opt::Pos;
pub use aopt::opt::RefOpt;
pub use aopt::parser::UserStyle;
pub use aopt::prelude::ctor_default_name;
pub use aopt::prelude::AOpt;
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
pub use aopt::prelude::Opt;
pub use aopt::prelude::OptParser;
pub use aopt::prelude::OptValidator;
pub use aopt::prelude::OptValueExt;
pub use aopt::prelude::Policy;
pub use aopt::prelude::PolicyParser;
pub use aopt::prelude::PolicySettings;
pub use aopt::prelude::PrefixOptValidator;
pub use aopt::prelude::RawValParser;
pub use aopt::prelude::ReturnVal;
pub use aopt::prelude::ServicesValExt;
pub use aopt::prelude::Set;
pub use aopt::prelude::SetCfg;
pub use aopt::prelude::SetChecker;
pub use aopt::prelude::SetExt;
pub use aopt::prelude::SetValueFindExt;
pub use aopt::prelude::Store;
pub use aopt::prelude::Style;
pub use aopt::prelude::ValInitializer;
pub use aopt::prelude::ValStorer;
pub use aopt::prelude::ValValidator;
pub use aopt::prelude::VecStore;
pub use aopt::raise_error;
pub use aopt::raise_failure;
pub use aopt::value::raw2str;
pub use aopt::value::Placeholder;
pub use aopt::Error as CoteError;
pub use aopt::GetoptRes;
pub use aopt::RawVal;
pub use aopt::Uid;
pub use cote_derive::Cote;
pub use cote_derive::CoteOpt;
pub use cote_derive::CoteVal;

pub use alter::Alter;
pub use alter::Hint;
pub use help::display_set_help;
pub use help::HelpDisplayCtx;
pub use meta::IntoConfig;
pub use meta::OptionMeta;
pub use parser::Parser;
pub use rctx::FailedInfo;
pub use rctx::RunningCtx;
pub use value::Fetch;

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

pub trait Status {
    fn status(&self) -> bool;
}

impl Status for ReturnVal {
    fn status(&self) -> bool {
        ReturnVal::status(self)
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

#[derive(Debug, Clone)]
pub struct NullPolicy<'inv, Set, Ser> {
    style_manager: OptStyleManager,

    marker: PhantomData<(Set, Ser, &'inv ())>,
}

impl<'inv, Set, Ser> Default for NullPolicy<'inv, Set, Ser> {
    fn default() -> Self {
        Self {
            style_manager: OptStyleManager::default(),
            marker: Default::default(),
        }
    }
}

impl<'inv, Set, Ser> Policy for NullPolicy<'inv, Set, Ser> {
    type Ret = ReturnVal;

    type Set = Parser<'inv, Set, Ser>;

    type Inv<'a> = Invoker<'a, Parser<'inv, Set, Ser>, Ser>;

    type Ser = Ser;

    type Error = aopt::Error;

    fn parse(
        &mut self,
        _: &mut Self::Set,
        _: &mut Self::Inv<'_>,
        _: &mut Self::Ser,
        _: ARef<Args>,
    ) -> Result<Self::Ret, Self::Error> {
        Ok(ReturnVal::default())
    }
}

impl<'inv, Set, Ser> PolicySettings for NullPolicy<'inv, Set, Ser> {
    fn style_manager(&self) -> &OptStyleManager {
        &self.style_manager
    }

    fn style_manager_mut(&mut self) -> &mut OptStyleManager {
        &mut self.style_manager
    }

    fn strict(&self) -> bool {
        false
    }

    fn styles(&self) -> &[UserStyle] {
        &self.style_manager
    }

    fn no_delay(&self) -> Option<&[aopt::AStr]> {
        None
    }

    fn overload(&self) -> bool {
        false
    }

    fn set_strict(&mut self, _: bool) -> &mut Self {
        self
    }

    fn set_styles(&mut self, _: Vec<UserStyle>) -> &mut Self {
        self
    }

    fn set_no_delay(&mut self, _: impl Into<aopt::AStr>) -> &mut Self {
        self
    }

    fn set_overload(&mut self, _: bool) -> &mut Self {
        self
    }
}

impl<'inv, Set, Ser> APolicyExt<NullPolicy<'inv, Set, Ser>> for NullPolicy<'inv, Set, Ser>
where
    Set: Default,
    Ser: Default,
{
    fn default_ser(&self) -> <NullPolicy<'inv, Set, Ser> as Policy>::Ser {
        Ser::default()
    }

    fn default_set(&self) -> <NullPolicy<'inv, Set, Ser> as Policy>::Set {
        Parser::default()
    }

    fn default_inv<'a>(&self) -> <NullPolicy<'inv, Set, Ser> as Policy>::Inv<'a> {
        Invoker::default()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_example_simple() {
        use crate as cote;
        use crate::*;
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

        let example = Example::parse(Args::from(["app", "--foo", "42"]));

        assert!(example.is_ok());

        let example = example.unwrap();

        assert!(example.foo);
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
            #[arg(index = 2..)]
            sources: Vec<Pos<PathBuf>>,
        }

        let example = CopyTool::parse(Args::from(["app", "--force"]));

        assert!(example.is_err());

        let example = CopyTool::parse(Args::from([
            "app", "--force", ".", "../foo", "../bar/", "other",
        ]))
        .unwrap();

        assert!(example.force);
        assert!(!example.recursive);
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

        fn search<Set, Ser>(_: &mut Set, _: &mut Ser) -> Result<Option<Vec<String>>, aopt::Error> {
            Ok(Some(
                ["file1", "file2", "dir1", "dir2"]
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect(),
            ))
        }

        fn find_main<Set, Ser>(set: &mut Set, _: &mut Ser) -> Result<Option<()>, aopt::Error>
        where
            Set: SetValueFindExt,
        {
            let tool = Find::try_extract(set)?;

            assert!(tool.hard,);
            assert!(!tool.symbol);
            assert!(tool.never);
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

        let args = Args::from(["app", ".", "-H", "-name=foo", "-size", "42"]);

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

        let args = Args::from(["app", "ls", "--all", "--depth=42", "."]);

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

        let args = Args::from(["app", "list", "--all", "--depth=6", "."]);

        let app = App::parse(args);

        assert!(app.is_err());

        let args = Args::from(["app", "--count=8", "find", "something"]);

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

        let args = Args::from(["app", "--count", "42"]);

        let app = App::parse(args);

        assert!(app.is_err());

        let args = Args::from(["app", "--count=42", "list"]);

        let CoteRes {
            ret,
            parser: mut app,
            ..
        } = App::parse_args(args)?;

        assert!(!ret.status());
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
