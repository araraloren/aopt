#![doc = include_str!("../README.md")]
pub mod meta;

use std::borrow::Cow;
use std::fmt::Debug;
use std::future::Future;
use std::ops::Deref;
use std::ops::DerefMut;

use aopt::prelude::*;
use aopt::value::Placeholder;
use aopt::Error;
use aopt::RawVal;
use aopt_help::prelude::Block;
use aopt_help::prelude::Store;

use meta::IntoConfig;

pub mod prelude {
    pub use crate::display_help;
    pub use crate::display_set_help;
    pub use crate::meta::IntoConfig;
    pub use crate::meta::OptionMeta;
    pub use crate::simple_display_set_help;
    pub use crate::CoteApp;
    pub use crate::ExtractFromSetDerive;
    pub use crate::IntoParserDerive;
    pub use aopt;
    pub use aopt_help;
    pub use cote_derive;

    pub mod derive {
        pub use cote_derive::Cote;
        pub use crate::ExtractFromSetDerive;
        pub use crate::IntoParserDerive;
    }
}

pub trait IntoParserDerive<'zlifetime, P>
where
    P::Ser: 'zlifetime,
    P::Set: Set + 'zlifetime,
    P::Error: Into<aopt::Error>,
    P: Policy + APolicyExt<P> + Default,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    fn into_parser() -> Result<Parser<'zlifetime, P>, Error> {
        let mut parser = Parser::<'zlifetime, P>::new(P::default());
        Self::update(&mut parser)?;
        Ok(parser)
    }
    fn update(parser: &mut Parser<'zlifetime, P>) -> Result<(), Error>;
}

pub trait ExtractFromSetDerive<'zlifetime, S>
where
    S: SetValueFindExt,
{
    fn try_extract(set: &'zlifetime mut S) -> Result<Self, aopt::Error>
    where
        Self: Sized;
}

pub struct CoteApp<'a, P>
where
    P: Policy,
{
    name: String,

    parser: Parser<'a, P>,
}

impl<'a, P> Debug for CoteApp<'a, P>
where
    P::Ret: Debug,
    P::Set: Debug,
    P::Ser: Debug,
    P: Policy + Debug,
    P::Inv<'a>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cote")
            .field("name", &self.name)
            .field("parser", &self.parser)
            .finish()
    }
}

impl<'a, P> Default for CoteApp<'a, P>
where
    P::Set: Default,
    P::Ser: Default,
    P::Inv<'a>: Default,
    P: Policy + APolicyExt<P> + Default,
{
    fn default() -> Self {
        Self {
            name: "Cote".to_owned(),
            parser: Parser::default(),
        }
    }
}

impl<'a, P: Policy> Deref for CoteApp<'a, P> {
    type Target = Parser<'a, P>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl<'a, P: Policy> DerefMut for CoteApp<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P: Policy + APolicyExt<P>,
{
    pub fn new<S: Into<String>>(name: S, policy: P) -> Self {
        Self {
            name: name.into(),

            parser: Parser::new(policy),
        }
    }

    pub fn inner_parser_mut(&mut self) -> &mut Parser<'a, P> {
        &mut self.parser
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P::Ser: 'a,
    P: Policy,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser + 'a,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    P::Inv<'a>: HandlerCollection<'a, P::Set, P::Ser>,
{
    /// Call [`inject_opt`](crate::InjectConfig::inject_opt) add option.
    ///
    /// ```ignore
    /// # use cote::prelude::*;
    /// # use aopt::prelude::*;
    /// # use aopt::Error;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     let config: MetaConfig<String> = serde_json::from_str(
    ///         r#"
    ///                 {
    ///                     "id": "-c",
    ///                     "option": "-c=s",
    ///                     "hint": "-c <str>",
    ///                     "help": "This is a help for option c",
    ///                     "action": "App",
    ///                     "assoc": "Str",
    ///                     "alias": null,
    ///                     "value": [
    ///                     "we",
    ///                     "it"
    ///                     ]
    ///                 }
    ///             "#,
    ///     )
    ///     .unwrap();
    ///
    ///     cote.inject_opt(config)?;
    ///
    ///     let config: MetaConfig<i64> = serde_json::from_str(
    ///         r#"
    ///                     {
    ///                         "id": "-p",
    ///                         "option": "--point=i",
    ///                         "hint": "--point <int>",
    ///                         "help": "This is a help for option",
    ///                         "action": "App",
    ///                         "assoc": "Int",
    ///                         "alias": [
    ///                             "-p"
    ///                         ]
    ///                     }
    ///                 "#,
    ///     )
    ///     .unwrap();
    ///
    ///     cote.inject_opt(config)?;
    ///
    ///     cote.run_with(["-p", "256"].into_iter(), |ret, cote: &Cote<AFwdPolicy>| {
    ///         if ret.is_some() {
    ///             assert_eq!(
    ///                 &vec!["we".to_owned(), "it".to_owned()],
    ///                 cote.find_vals::<String>("-c")?
    ///             );
    ///             assert_eq!(&256, cote.find_val::<i64>("--point")?);
    ///             println!("cote running okay!!!");
    ///         }
    ///         Ok(())
    ///     })?;
    /// #
    /// #   Ok(())
    /// # }
    /// ```
    pub fn inject_opt<T: ErasedTy + Clone, I: IntoConfig<Ret = SetCfg<P::Set>>>(
        &mut self,
        mut option_meta: I,
    ) -> Result<ParserCommitWithValue<'a, '_, P::Inv<'a>, P::Set, P::Ser, Placeholder, T>, Error>
    {
        let set = self.parser.optset();
        let config = option_meta.into_config(set)?;

        Ok(self.parser.add_opt_cfg(config)?.set_value_type_only::<T>())
    }

    pub fn add_help<S: Into<String>>(
        &mut self,
        author: S,
        version: S,
        description: S,
    ) -> Result<&mut Self, Error> {
        self.add_help_width(author, version, description, 20, 20)
    }

    pub fn add_help_width<S: Into<String>>(
        &mut self,
        author: S,
        version: S,
        description: S,
        option_width: usize,
        usage_width: usize,
    ) -> Result<&mut Self, Error> {
        let name = self.name.clone();
        let (author, version, description) = (author.into(), version.into(), description.into());

        self.add_opt("--help=b")?
            .add_alias("-h")
            .add_alias("-?")
            .set_help("Display help message")
            .on(
                move |set: &mut P::Set, _: &mut P::Ser| -> Result<Option<()>, Error> {
                    display_set_help!(
                        &name,
                        set,
                        author,
                        version,
                        description,
                        option_width,
                        usage_width
                    )?;
                    std::process::exit(0)
                },
            )?;
        Ok(self)
    }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::Cote;
    /// use cote::Error;
    /// use aopt::prelude::*;
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_mut_with(["-a", "-b", "42"].into_iter(), move |ret, cote| {
    ///         if ret.is_some() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // cote still avilable here, Cote::run_mut_with pass mutable reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub fn run_mut_with<'c, 'b, I, R, F>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        F: FnMut(P::Ret, &'b mut CoteApp<P>) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;

        // initialize the option value
        parser.init()?;

        let ret = parser
            .parse(aopt::ARef::new(Args::from(args)))
            .map_err(Into::into)?;

        r(ret, self)
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub fn run_mut<'c, 'b, R, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        F: FnMut(P::Ret, &'b mut CoteApp<P>) -> Result<R, Error>,
    {
        let args = Args::from_env().into_inner();
        self.run_mut_with(args.into_iter(), r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use cote::Cote;
    /// use cote::Error;
    /// use aopt::prelude::*;
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_async_mut_with(["-a", "-b", "42"].into_iter(), |ret, cote| async move {
    ///         if ret.is_some() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // cote still avilable here, Cote::run_async_mut_with pass mutable reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run_async_mut_with<'c, 'b, I, R, FUT, F>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b mut CoteApp<P>) -> FUT,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        // initialize the option value
        parser.init()?;
        match parser.parse(aopt::ARef::new(Args::from(args))) {
            Ok(ret) => {
                let ret = r(ret, self).await;

                async_ret = ret;
            }
            Err(e) => {
                async_ret = Err(e.into());
            }
        }
        async_ret
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub async fn run_async_mut<'c, 'b, R, FUT, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b mut CoteApp<P>) -> FUT,
    {
        let args = Args::from_env().into_inner();
        self.run_async_mut_with(args.into_iter(), r).await
    }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::Cote;
    /// use cote::Error;
    /// use aopt::prelude::*;
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_with(["-a", "-b", "42"].into_iter(), move |ret, cote| {
    ///         if ret.is_some() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // cote still avilable here, Cote::run_with pass reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub fn run_with<'c, 'b, I, R, F>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        F: FnMut(P::Ret, &'b CoteApp<P>) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;

        // initialize the option value
        parser.init()?;

        let ret = parser
            .parse(aopt::ARef::new(Args::from(args)))
            .map_err(Into::into)?;

        r(ret, self)
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub fn run<'c, 'b, R, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        F: FnMut(P::Ret, &'b CoteApp<P>) -> Result<R, Error>,
    {
        let args = Args::from_env().into_inner();
        self.run_with(args.into_iter(), r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use cote::Cote;
    /// use cote::Error;
    /// use aopt::prelude::*;
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_async_with(["-a", "-b", "42"].into_iter(), |ret, cote| async move {
    ///         if ret.is_some() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // cote still avilable here, Cote::run_async_with pass reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run_async_with<'c, 'b, I, R, FUT, F>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b CoteApp<P>) -> FUT,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        // initialize the option value
        parser.init()?;
        match parser.parse(aopt::ARef::new(Args::from(args))) {
            Ok(ret) => {
                let ret = r(ret, self).await;

                async_ret = ret;
            }
            Err(e) => {
                async_ret = Err(e.into());
            }
        }
        async_ret
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub async fn run_async<'c, 'b, R, FUT, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b CoteApp<P>) -> FUT,
    {
        let args = Args::from_env().into_inner();
        self.run_async_with(args.into_iter(), r).await
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P: Policy,
    P::Set: Set,
{
    pub fn new_with<S: Into<String>>(
        name: S,
        policy: P,
        optset: P::Set,
        invoker: P::Inv<'a>,
        appser: P::Ser,
    ) -> Self {
        Self {
            name: name.into(),

            parser: Parser::new_with(policy, optset, invoker, appser),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn display_help<'c>(
        &self,
        head: impl Into<Cow<'c, str>>,
        foot: impl Into<Cow<'c, str>>,
        max_width: usize,
        usage_width: usize,
    ) -> Result<(), Error> {
        let head = head.into();
        let foot = foot.into();
        let name = self.name.as_str();

        simple_display_set_help(self.optset(), name, &head, &foot, max_width, usage_width)
            .map_err(|e| Error::raise_error(format!("Can not show help message: {:?}", e)))
    }
}

pub fn simple_display_set_help<'a, T: Set, S: Into<Cow<'a, str>>>(
    set: &T,
    name: S,
    head: S,
    foot: S,
    max_width: usize,
    usage_width: usize,
) -> Result<(), aopt_help::Error> {
    let mut app_help = aopt_help::AppHelp::new(
        name,
        head,
        foot,
        aopt_help::prelude::Style::default(),
        std::io::stdout(),
        max_width,
        usage_width,
    );
    let global = app_help.global_mut();

    global.add_block(Block::new("command", "<COMMAND>", "", "COMMAND:", ""))?;
    global.add_block(Block::new("option", "", "", "OPTION:", ""))?;
    global.add_block(Block::new("args", "[ARGS]", "", "ARGS:", ""))?;
    for opt in set.iter() {
        if opt.mat_style(Style::Pos) {
            global.add_store(
                "args",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::default(),
                    !opt.force(),
                    true,
                ),
            )?;
        } else if opt.mat_style(Style::Cmd) {
            global.add_store(
                "command",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::default(),
                    !opt.force(),
                    true,
                ),
            )?;
        } else if opt.mat_style(Style::Argument)
            || opt.mat_style(Style::Boolean)
            || opt.mat_style(Style::Combined)
        {
            global.add_store(
                "option",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::default(),
                    !opt.force(),
                    false,
                ),
            )?;
        }
    }

    app_help.display(true)?;

    Ok(())
}

/// Display help message of [`Cote`] generate from `Cargo.toml`.
/// The `head` will be generate from package's description.
/// The `foot` will be generate from package's authors and version.
/// Default option text width is 20, and default usage width is 10.
#[macro_export]
macro_rules! display_help {
    ($cote:ident) => {{
        let foot = format!(
            "Create by {} v{}",
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_VERSION")
        );
        let head = format!("{}", env!("CARGO_PKG_DESCRIPTION"));

        $cote.display_help(head, foot, 10, 20)
    }};
}

/// Display help message of [`Cote`] generate from `Cargo.toml`.
/// The `head` will be generate from package's description.
/// The `foot` will be generate from package's authors and version.
#[macro_export]
macro_rules! display_set_help {
    ($name:expr, $set:ident, $author:expr, $version:expr, $description:expr, $width:expr, $usage_width:expr) => {{
        let foot = format!("Create by {} v{}", $author, $version,);
        let head = format!("{}", $description);

        fn __check_set<S: aopt::prelude::Set>(a: &S) -> &S {
            a
        }

        fn __check_name<T: Into<String>>(a: T) -> String {
            a.into()
        }

        $crate::simple_display_set_help(
            __check_set($set),
            __check_name($name),
            head,
            foot,
            $width,
            $usage_width,
        )
        .map_err(|e| aopt::Error::raise_error(format!("Can not show help message: {:?}", e)))
    }};
}
