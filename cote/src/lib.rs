#![doc = include_str!("../README.md")]
pub mod _reference;
pub(crate) mod help;
pub(crate) mod infer;
pub(crate) mod meta;
pub(crate) mod parser;
pub(crate) mod rctx;
pub(crate) mod value;

#[cfg(feature = "shell")]
pub mod shell {
    pub use crate::parser::shell::CompletionManager;
    pub use aopt::shell::*;
}

pub mod valid;

pub use aopt;
pub use aopt::Error;
pub use aopt_help;
pub use cote_derive;

pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    pub use aopt::error;
    pub use aopt::failure;
    pub use aopt::opt::AnyOpt;
    pub use aopt::opt::Cmd;
    pub use aopt::opt::Main;
    pub use aopt::opt::MutOpt;
    pub use aopt::opt::Pos;
    pub use aopt::parser::UserStyle;
    pub use aopt::prelude::ctor_default_name;
    pub use aopt::prelude::AOpt;
    pub use aopt::prelude::ARef;
    pub use aopt::prelude::Action;
    pub use aopt::prelude::AppServices;
    pub use aopt::prelude::AppStorage;
    pub use aopt::prelude::Args;
    pub use aopt::prelude::Commit;
    pub use aopt::prelude::ConfigBuild;
    pub use aopt::prelude::ConfigBuildInfer;
    pub use aopt::prelude::ConfigBuildWith;
    pub use aopt::prelude::ConfigBuilder;
    pub use aopt::prelude::ConfigBuilderWith;
    pub use aopt::prelude::ConfigValue;
    pub use aopt::prelude::Ctor;
    pub use aopt::prelude::Ctx;
    pub use aopt::prelude::DefaultSetChecker;
    pub use aopt::prelude::ErasedTy;
    pub use aopt::prelude::ErasedValue;
    pub use aopt::prelude::FilterMatcher;
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
    pub use aopt::prelude::PrefixedValidator;
    pub use aopt::prelude::RawValParser;
    pub use aopt::prelude::Return;
    pub use aopt::prelude::Set;
    pub use aopt::prelude::SetCfg;
    pub use aopt::prelude::SetChecker;
    pub use aopt::prelude::SetExt;
    pub use aopt::prelude::SetOpt;
    pub use aopt::prelude::SetValueFindExt;
    pub use aopt::prelude::Store;
    pub use aopt::prelude::Style;
    pub use aopt::prelude::ValInitializer;
    pub use aopt::prelude::ValStorer;
    pub use aopt::prelude::ValValidator;
    pub use aopt::prelude::VecStore;
    pub use aopt::value::raw2str;
    pub use aopt::value::Placeholder;
    pub use aopt::value::Stop;
    pub use aopt::GetoptRes;
    pub use aopt::Uid;
    pub use cote_derive::Cote;
    pub use cote_derive::CoteOpt;
    pub use cote_derive::CoteVal;

    pub use crate::help::display_set_help;
    pub use crate::help::HelpContext;
    pub use crate::help::HelpDisplay;
    pub use crate::help::DEFAULT_OPTION_WIDTH;
    pub use crate::help::DEFAULT_USAGE_WIDTH;
    pub use crate::infer::InferOverride;
    pub use crate::meta::OptionMeta;
    pub use crate::parser::Parser;
    pub use crate::rctx::Failure;
    pub use crate::rctx::Frame;
    pub use crate::rctx::HideValue;
    pub use crate::rctx::RunningCtx;
    pub use crate::valid;
    pub use crate::value::fetch_uid_impl;
    pub use crate::value::fetch_vec_uid_impl;
    pub use crate::value::Fetch;
    pub use crate::CoteRes;
    pub use crate::ExtractFromSetDerive;
    pub use crate::IntoParserDerive;
    pub use crate::NullPolicy;
    pub use crate::Status;
    pub use aopt::prelude::ASet as CoteSet;

    pub type FwdPolicy<'inv, S> =
        aopt::prelude::FwdPolicy<Parser<'inv, S>, DefaultSetChecker<Parser<'inv, S>>>;

    pub type DelayPolicy<'inv, S> =
        aopt::prelude::DelayPolicy<Parser<'inv, S>, DefaultSetChecker<Parser<'inv, S>>>;

    pub type SeqPolicy<'inv, S> =
        aopt::prelude::SeqPolicy<Parser<'inv, S>, DefaultSetChecker<Parser<'inv, S>>>;
}

use std::marker::PhantomData;

use aopt::args::Args;
use aopt::ctx::Invoker;
use aopt::parser::Policy;
use aopt::parser::PolicySettings;
use aopt::parser::Return;
use aopt::parser::UserStyle;
use aopt::prelude::ConfigValue;
use aopt::prelude::OptParser;
use aopt::prelude::OptStyleManager;
use aopt::prelude::OptValidator;
use aopt::prelude::SetCfg;
use aopt::prelude::SetValueFindExt;
use aopt::set::Set;

use crate::prelude::Parser;

pub trait IntoParserDerive<'inv, S>
where
    SetCfg<S>: ConfigValue + Default,
    S: Set + OptParser + OptValidator + Default,
{
    fn into_parser() -> Result<Parser<'inv, S>> {
        let mut parser = Parser::default();
        Self::update(&mut parser)?;
        Ok(parser)
    }
    fn update(parser: &mut Parser<'inv, S>) -> Result<()>;
}

pub trait ExtractFromSetDerive<'set, S: SetValueFindExt>
where
    SetCfg<S>: ConfigValue + Default,
{
    fn try_extract(set: &'set mut S) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoteRes<P, Policy>
where
    Policy: crate::prelude::Policy,
{
    pub policy: P,

    pub ret: Policy::Ret,

    pub parser: Policy::Set,
}

pub trait Status {
    fn status(&self) -> bool;
}

impl Status for Return {
    fn status(&self) -> bool {
        Return::status(self)
    }
}

#[derive(Debug, Clone)]
pub struct NullPolicy<'inv, S> {
    style_manager: OptStyleManager,

    marker: PhantomData<(S, &'inv ())>,
}

impl<S> Default for NullPolicy<'_, S> {
    fn default() -> Self {
        Self {
            style_manager: OptStyleManager::default(),
            marker: Default::default(),
        }
    }
}

impl<'inv, S> Policy for NullPolicy<'inv, S> {
    type Ret = Return;

    type Set = Parser<'inv, S>;

    type Inv<'a> = Invoker<'a, Parser<'inv, S>>;

    type Error = crate::Error;

    fn parse(&mut self, _: &mut Self::Set, _: &mut Self::Inv<'_>, _: Args) -> Result<Self::Ret> {
        Ok(Return::default())
    }
}

impl<S> PolicySettings for NullPolicy<'_, S> {
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

    fn no_delay(&self) -> Option<&[String]> {
        None
    }

    fn overload(&self) -> bool {
        false
    }

    fn prepolicy(&self) -> bool {
        false
    }

    fn set_strict(&mut self, _: bool) -> &mut Self {
        self
    }

    fn set_styles(&mut self, _: Vec<UserStyle>) -> &mut Self {
        self
    }

    fn set_no_delay(&mut self, _: impl Into<String>) -> &mut Self {
        self
    }

    fn set_overload(&mut self, _: bool) -> &mut Self {
        self
    }

    fn set_prepolicy(&mut self, _: bool) -> &mut Self {
        self
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_example_simple() {
        use crate as cote;
        use crate::prelude::*;
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

        assert_eq!(parser["--foo"].help(), "a flag argument");
        assert_eq!(parser["bar"].help(), "a position argument");
    }

    #[test]
    fn test_multiple_pos_arguments() {
        use crate::prelude::*;
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
        use crate::prelude::*;
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

        #[allow(dead_code)]
        fn search<Set>(_: &mut Set, _: &mut Ctx) -> Result<Option<Vec<String>>, aopt::Error> {
            Ok(Some(
                ["file1", "file2", "dir1", "dir2"]
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect(),
            ))
        }

        fn find_main<Set>(set: &mut Set, _: &mut Ctx) -> Result<Option<()>, aopt::Error>
        where
            Set: SetValueFindExt,
            SetCfg<Set>: ConfigValue + Default,
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
        use crate::prelude::*;
        // macro generate the code depend on crate name
        use crate as cote;
        use std::path::PathBuf;

        #[derive(Debug, Cote, PartialEq, Default)]
        #[cote(prepolicy, help, name = "app")]
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
