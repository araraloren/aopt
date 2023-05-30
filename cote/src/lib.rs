pub mod ctx;
pub mod help;
pub mod meta;
pub mod ser;
pub mod valid;
pub mod value;

use std::borrow::Cow;
use std::fmt::Debug;
use std::future::Future;
use std::ops::{Deref, DerefMut};

use aopt::prelude::*;
use aopt::value::Placeholder;
use aopt::Error;
use aopt::RawVal;
use aopt_help::prelude::Block;
use aopt_help::prelude::Store;

pub use ctx::RunningCtx;
pub use help::HelpDisplayCtx;
pub use ser::CoteServiceExt;

pub mod prelude {
    pub use crate::help::HelpDisplayCtx;
    pub use crate::meta::IntoConfig;
    pub use crate::meta::OptionMeta;
    pub use crate::ser::CoteServiceExt;
    pub use crate::valid;
    pub use crate::value;
    pub use crate::value::InferValueMut;
    pub use crate::CoteApp;
    pub use crate::CoteParser;
    pub use crate::ExtractFromSetDerive;
    pub use crate::IntoParserDerive;
    pub use aopt;
    pub use aopt::prelude::*;
    pub use cote_derive::Cote;
}

use crate::meta::IntoConfig;

pub trait IntoParserDerive<Set, Inv, Ser>
where
    Ser: aopt::prelude::ErasedTy,
    Inv: aopt::prelude::ErasedTy,
    SetCfg<Set>: Config + ConfigValue,
    Set: aopt::prelude::Set + aopt::prelude::ErasedTy,
{
    fn into_parser() -> Result<Parser<Set, Inv, Ser>, Error>
    where
        Set: ADefaultVal,
        Ser: ADefaultVal,
        Inv: ADefaultVal,
    {
        Self::into_parser_with(
            Set::a_default_val(),
            Inv::a_default_val(),
            Ser::a_default_val(),
        )
    }

    fn into_parser_with(set: Set, inv: Inv, ser: Ser) -> Result<Parser<Set, Inv, Ser>, Error> {
        let mut parser = Parser::new(set, inv, ser);
        Self::update(&mut parser)?;
        Ok(parser)
    }

    fn update(parser: &mut Parser<Set, Inv, Ser>) -> Result<(), Error>;
}

pub trait ExtractFromSetDerive<'a, Set>
where
    Set: SetValueFindExt,
{
    fn try_extract(set: &'a mut Set) -> Result<Self, aopt::Error>
    where
        Self: Sized;
}

#[derive(Debug, Default, Clone)]
pub struct CoteParser<Set, Inv, Ser> {
    name: String,

    parser: Parser<Set, Inv, Ser>,
}

impl<Set, Inv, Ser> Deref for CoteParser<Set, Inv, Ser> {
    type Target = Parser<Set, Inv, Ser>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl<Set, Inv, Ser> DerefMut for CoteParser<Set, Inv, Ser> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

impl<Set, Inv, Ser> CoteParser<Set, Inv, Ser> {
    pub fn new_with_policy<'a, P>(name: impl Into<String>) -> Self
    where
        Set: ADefaultVal,
        Ser: ADefaultVal,
        Inv: ADefaultVal,
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
    {
        Self::new(
            name,
            Set::a_default_val(),
            Inv::a_default_val(),
            Ser::a_default_val(),
        )
    }
}

impl<Set, Inv, Ser> CoteParser<Set, Inv, Ser> {
    pub fn new(name: impl Into<String>, set: Set, inv: Inv, ser: Ser) -> Self {
        Self {
            name: name.into(),
            parser: Parser::new(set, inv, ser),
        }
    }

    pub fn new_with_parser(name: impl Into<String>, parser: Parser<Set, Inv, Ser>) -> Self {
        Self {
            name: name.into(),
            parser,
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

    pub fn inner_parser(&self) -> &Parser<Set, Inv, Ser> {
        &self.parser
    }

    pub fn inner_parser_mut(&mut self) -> &mut Parser<Set, Inv, Ser> {
        &mut self.parser
    }

    pub fn set_policy<'a, P>(self, policy: P) -> CoteApp<'a, P>
    where
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
    {
        CoteApp::new_with_parser(policy, self)
    }
}

impl<'a, Set, Inv, Ser> CoteParser<Set, Inv, Ser>
where
    SetOpt<Set>: Opt,
    Set: aopt::set::Set + OptValidator + OptParser,
    <Set as OptParser>::Output: Information,
    SetCfg<Set>: Config + ConfigValue + Default,
    Inv: HandlerCollection<'a, Set, Ser>,
{
    /// Add option by option configuration generated from [`OptionMeta`](crate::meta::OptionMeta).
    ///
    /// # Example load option from json configuration.
    /// ```rust
    /// # use aopt::Error;
    /// # use cote::prelude::*;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     let mut policy = AFwdPolicy::default();
    ///     let mut parser = CoteParser::new_with("example".to_owned(), &policy);
    ///
    ///     let config: OptionMeta<String> = serde_json::from_str(
    ///         r#"
    ///             {
    ///                 "id": "-c",
    ///                 "option": "-c=s",
    ///                 "hint": "-c <str>",
    ///                 "help": "This is a help for option c",
    ///                 "action": "App",
    ///                 "alias": null,
    ///                 "value": [
    ///                 "we",
    ///                 "it"
    ///                 ]
    ///             }
    ///         "#,
    ///     )
    ///     .unwrap();
    ///
    ///     parser.add_opt_meta(config)?;
    ///
    ///     let config: OptionMeta<i64> = serde_json::from_str(
    ///         r#"
    ///             {
    ///                 "id": "-p",
    ///                 "option": "--point=i",
    ///                 "hint": "--point <int>",
    ///                 "help": "This is a help for option",
    ///                 "action": "App",
    ///                 "alias": [
    ///                     "-p"
    ///                 ]
    ///             }
    ///         "#,
    ///     )
    ///     .unwrap();
    ///
    ///     parser.add_opt_meta(config)?;
    ///
    ///     parser.run_with(["-p", "256"].into_iter(), &mut policy, |ret, cote| {
    ///         if ret.status() {
    ///             assert_eq!(
    ///                 &vec!["we".to_owned(), "it".to_owned()],
    ///                 cote.find_vals::<String>("-c")?
    ///             );
    ///             assert_eq!(&256, cote.find_val::<i64>("--point")?);
    ///             println!("cote parser running okay!!!");
    ///         }
    ///         Ok(())
    ///     })?;
    /// #
    /// #    Ok(())
    /// # }
    /// ```
    pub fn add_opt_meta(
        &mut self,
        meta: impl IntoConfig<Ret = SetCfg<Set>>,
    ) -> Result<ParserCommit<'a, '_, Inv, Set, Ser, Placeholder>, aopt::Error> {
        let set = self.parser.optset();
        let config = meta.into_config(set)?;

        self.parser.add_opt_cfg(config)
    }

    /// This function will insert help option `--help;-h;-?: Display help message`.
    pub fn add_help_option(&mut self) -> Result<&mut Self, aopt::Error> {
        self.add_opt_i::<bool>("--help;-h;-?: Display help message")?;
        Ok(self)
    }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    ///```rust
    /// # use aopt::Error;
    /// # use cote::prelude::*;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     let mut policy = AFwdPolicy::default();
    ///     let mut parser = CoteParser::new_with("example".to_owned(), &policy);
    ///
    ///     parser.add_opt_i::<bool>("-a!")?;
    ///     parser.add_opt_i::<i64>("-b")?;
    ///
    ///     parser.run_mut_with(
    ///         ["-a", "-b", "42"].into_iter(),
    ///         &mut policy,
    ///         |ret, parser| {
    ///             if ret.status() {
    ///                 assert_eq!(parser.find_val::<bool>("-a")?, &true);
    ///                 assert_eq!(parser.find_val::<i64>("-b")?, &42);
    ///             }
    ///             Ok(())
    ///         },
    ///     )?;
    ///     println!("{} running over!", parser.name());
    /// #
    /// # Ok(())
    /// # }
    ///```
    pub fn run_mut_with<'c, 'b, I, R, F, P>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
        F: FnMut(P::Ret, &'b mut Self) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;

        // initialize the option value
        parser.init()?;

        let ret = parser.parse_with(aopt::ARef::new(Args::from(args)), policy)?;

        r(ret, self)
    }

    /// Call [`run_mut_with`](CoteParser::run_mut_with) with default arguments [`args()`](std::env::args).
    pub fn run_mut<'c, 'b, R, F, P>(&'c mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        'c: 'b,
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
        F: FnMut(P::Ret, &'b mut Self) -> Result<R, Error>,
    {
        let args = Args::from_env().into_inner();
        self.run_mut_with(args.into_iter(), policy, r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aopt::Error;
    /// # use cote::prelude::*;
    /// #
    /// #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    ///     let mut policy = AFwdPolicy::default();
    ///     let mut parser = CoteParser::new_with("example".to_owned(), &policy);
    ///
    ///     parser.add_opt_i::<bool>("-a!")?;
    ///     parser.add_opt_i::<i64>("-b")?;
    ///
    ///     parser
    ///         .run_async_mut_with(
    ///             ["-a", "-b", "42"].into_iter(),
    ///             &mut policy,
    ///             |ret, parser| async move {
    ///                 if ret.status() {
    ///                     assert_eq!(parser.find_val::<bool>("-a")?, &true);
    ///                     assert_eq!(parser.find_val::<i64>("-b")?, &42);
    ///                 }
    ///                 Ok(())
    ///             },
    ///         )
    ///         .await?;
    ///     println!("{} running over!", parser.name());
    /// # Ok(())
    /// # }
    ///```
    pub async fn run_async_mut_with<'c, 'b, I, R, FUT, F, P>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b mut Self) -> FUT,
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        // initialize the option value
        parser.init()?;
        match parser.parse_with(aopt::ARef::new(Args::from(args)), policy) {
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

    /// Call [`run_async_mut_with`](Self::run_async_mut_with) with default arguments [`args()`](std::env::args).
    pub async fn run_async_mut<'c, 'b, R, FUT, F, P>(
        &'c mut self,
        policy: &mut P,
        r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b mut Self) -> FUT,
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
    {
        let args = Args::from_env().into_inner();
        self.run_async_mut_with(args.into_iter(), policy, r).await
    }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    ///```rust
    /// # use aopt::Error;
    /// # use cote::prelude::*;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     let mut policy = AFwdPolicy::default();
    ///     let mut parser = CoteParser::new_with("example".to_owned(), &policy);
    ///
    ///     parser.add_opt_i::<bool>("-a!")?;
    ///     parser.add_opt_i::<i64>("-b")?;
    ///
    ///     parser.run_with(
    ///         ["-a", "-b", "42"].into_iter(),
    ///         &mut policy,
    ///         |ret, parser| {
    ///             if ret.status() {
    ///                 assert_eq!(parser.find_val::<bool>("-a")?, &true);
    ///                 assert_eq!(parser.find_val::<i64>("-b")?, &42);
    ///             }
    ///             Ok(())
    ///         },
    ///     )?;
    ///     println!("{} running over!", parser.name());
    /// #
    /// # Ok(())
    /// # }
    ///```
    pub fn run_with<'c, 'b, I, R, F, P>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
        F: FnMut(P::Ret, &'b Self) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;

        // initialize the option value
        parser.init()?;

        let ret = parser.parse_with(aopt::ARef::new(Args::from(args)), policy)?;

        r(ret, self)
    }

    /// Call [`run_with`](Self::run_with) with default arguments [`args()`](std::env::args).
    pub fn run<'c, 'b, R, F, P>(&'c mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        'c: 'b,
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
        F: FnMut(P::Ret, &'b Self) -> Result<R, Error>,
    {
        let args = Args::from_env().into_inner();
        self.run_with(args.into_iter(), policy, r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aopt::Error;
    /// # use cote::prelude::*;
    /// #
    /// #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    ///     let mut policy = AFwdPolicy::default();
    ///     let mut parser = CoteParser::new_with("example".to_owned(), &policy);
    ///
    ///     parser.add_opt_i::<bool>("-a!")?;
    ///     parser.add_opt_i::<i64>("-b")?;
    ///
    ///     parser
    ///         .run_async_with(
    ///             ["-a", "-b", "42"].into_iter(),
    ///             &mut policy,
    ///             |ret, parser| async move {
    ///                 if ret.status() {
    ///                     assert_eq!(parser.find_val::<bool>("-a")?, &true);
    ///                     assert_eq!(parser.find_val::<i64>("-b")?, &42);
    ///                 }
    ///                 Ok(())
    ///             },
    ///         )
    ///         .await?;
    ///     println!("{} running over!", parser.name());
    /// # Ok(())
    /// # }
    ///```
    pub async fn run_async_with<'c, 'b, I, R, FUT, F, P>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b Self) -> FUT,
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        // initialize the option value
        parser.init()?;
        match parser.parse_with(aopt::ARef::new(Args::from(args)), policy) {
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

    /// Call [`run_async_with`](Self::run_async_with) with default arguments [`args()`](std::env::args).
    pub async fn run_async<'c, 'b, R, FUT, F, P>(
        &'c mut self,
        policy: &mut P,
        r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b Self) -> FUT,
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
    {
        let args = Args::from_env().into_inner();
        self.run_async_with(args.into_iter(), policy, r).await
    }
}

impl<Set, Inv, Ser> CoteParser<Set, Inv, Ser>
where
    Set: aopt::prelude::Set,
{
    const DEFAULT_OPTION_WIDTH: usize = 40;
    const DEFAULT_USAGE_WIDTH: usize = 10;

    /// Generate and display the help message of current parser.
    pub fn display_help(
        &self,
        author: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<(), Error> {
        let set = self.parser.optset();
        let (author, version, description) = (author.into(), version.into(), description.into());
        let name = self.name.to_string();

        crate::display_set_help!(
            &name,
            set,
            author,
            version,
            description,
            Self::DEFAULT_OPTION_WIDTH,
            Self::DEFAULT_USAGE_WIDTH
        )
    }

    pub fn display_help_ctx(&self, ctx: &HelpDisplayCtx) -> Result<(), Error> {
        let name = ctx.generate_name();
        let set = self.parser.optset();

        crate::simple_display_set_help(
            set,
            &name,
            ctx.head(),
            ctx.foot(),
            ctx.width(),
            ctx.usagew(),
        )
        .map_err(|e| aopt::raise_error!("Can not show help message: {:?}", e))?;
        Ok(())
    }
}

impl<Set, Inv, Ser> CoteParser<Set, Inv, Ser>
where
    Set: SetValueFindExt,
{
    pub fn display_help_if(
        &self,
        option: &str,
        author: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<bool, Error> {
        self.display_help_if_width(
            option,
            author,
            version,
            description,
            Self::DEFAULT_OPTION_WIDTH,
            Self::DEFAULT_USAGE_WIDTH,
        )
    }

    pub fn display_help_if_ctx(&self, option: &str, ctx: &HelpDisplayCtx) -> Result<bool, Error> {
        let set = self.parser.optset();

        if let Ok(help_option) = set.find_val::<bool>(option) {
            if *help_option {
                let name = ctx.generate_name();
                let set = self.parser.optset();

                crate::simple_display_set_help(
                    set,
                    &name,
                    ctx.head(),
                    ctx.foot(),
                    ctx.width(),
                    ctx.usagew(),
                )
                .map_err(|e| aopt::raise_error!("Can not show help message: {:?}", e))?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn display_help_if_width(
        &self,
        option: &str,
        author: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        option_width: usize,
        usage_width: usize,
    ) -> Result<bool, Error> {
        let set = self.parser.optset();

        if let Ok(help_option) = set.find_val::<bool>(option) {
            if *help_option {
                let (author, version, description) =
                    (author.into(), version.into(), description.into());
                let name = self.name.to_string();

                crate::display_set_help!(
                    &name,
                    set,
                    author,
                    version,
                    description,
                    option_width,
                    usage_width
                )?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl<'a, Set, Inv, Ser> CoteParser<Set, Inv, Ser>
where
    Set: SetValueFindExt,
{
    pub fn extract_type<T>(&'a mut self) -> Result<T, Error>
    where
        T: ExtractFromSetDerive<'a, Set>,
    {
        let set = self.parser.optset_mut();

        T::try_extract(set)
    }
}

pub struct CoteApp<'a, P>
where
    P: Policy,
{
    policy: P,

    parser: CoteParser<P::Set, P::Inv<'a>, P::Ser>,
}

impl<'a, P> Debug for CoteApp<'a, P>
where
    P: Policy + Debug,
    P::Ser: Debug,
    P::Inv<'a>: Debug,
    P::Set: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoteApp")
            .field("policy", &self.policy)
            .field("parser", &self.parser)
            .finish()
    }
}

impl<'a, P: Policy> Default for CoteApp<'a, P>
where
    P::Set: ADefaultVal,
    P::Ser: ADefaultVal,
    P::Inv<'a>: ADefaultVal,
    P: Policy + ADefaultVal,
{
    fn default() -> Self {
        let policy = P::a_default_val();
        let parser = CoteParser::new_with_policy::<P>("CoteApp".to_owned());

        Self { policy, parser }
    }
}

impl<'a, P: Policy> Deref for CoteApp<'a, P> {
    type Target = CoteParser<P::Set, P::Inv<'a>, P::Ser>;

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
    P::Set: ADefaultVal,
    P::Ser: ADefaultVal,
    P::Inv<'a>: ADefaultVal,
    P: Policy,
{
    pub fn new_with_policy(name: impl Into<String>, policy: P) -> Self {
        let parser = CoteParser::new_with_policy::<P>(name.into());
        Self { policy, parser }
    }
}

impl<'a, P: Policy> CoteApp<'a, P> {
    pub fn new(
        name: impl Into<String>,
        policy: P,
        set: P::Set,
        inv: P::Inv<'a>,
        ser: P::Ser,
    ) -> Self {
        Self {
            policy,
            parser: CoteParser::new(name, set, inv, ser),
        }
    }

    pub fn new_with_parser(policy: P, parser: CoteParser<P::Set, P::Inv<'a>, P::Ser>) -> Self {
        Self { policy, parser }
    }

    pub fn policy(&self) -> &P {
        &self.policy
    }

    pub fn policy_mut(&mut self) -> &mut P {
        &mut self.policy
    }

    pub fn set_policy(&mut self, policy: P) -> &mut Self {
        self.policy = policy;
        self
    }

    pub fn parser(&self) -> &CoteParser<P::Set, P::Inv<'a>, P::Ser> {
        &self.parser
    }

    pub fn parser_mut(&mut self) -> &mut CoteParser<P::Set, P::Inv<'a>, P::Ser> {
        &mut self.parser
    }

    pub fn set_parser(&mut self, parser: CoteParser<P::Set, P::Inv<'a>, P::Ser>) -> &mut Self {
        self.parser = parser;
        self
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P::Set: Set,
    P: Policy,
{
    /// Call [`parse`](Policy::parse) parsing the given arguments.
    pub fn parse(&mut self, args: ARef<Args>) -> Result<P::Ret, Error> {
        self.parser.parse_with(args, &mut self.policy)
    }

    /// Call [`parse`](CoteApp::parse) parsing the [`Args`](Args::from_env).
    ///
    /// The [`status`](ReturnVal::status) is true if parsing successes
    /// otherwise it will be false if any [`failure`](Error::is_failure) raised.
    pub fn parse_env(&mut self) -> Result<P::Ret, Error> {
        self.parser.parse_with_env(&mut self.policy)
    }
}

impl<'a, P> PolicySettings for CoteApp<'a, P>
where
    P: Policy + PolicySettings,
{
    fn style_manager(&self) -> &OptStyleManager {
        self.policy().style_manager()
    }

    fn style_manager_mut(&mut self) -> &mut OptStyleManager {
        self.policy_mut().style_manager_mut()
    }

    fn strict(&self) -> bool {
        self.policy().strict()
    }

    fn styles(&self) -> &[UserStyle] {
        self.policy.styles()
    }

    fn no_delay(&self) -> Option<&[aopt::Str]> {
        self.policy().no_delay()
    }

    fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.policy_mut().set_strict(strict);
        self
    }

    fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.policy_mut().set_styles(styles);
        self
    }

    fn set_no_delay(&mut self, name: impl Into<aopt::Str>) -> &mut Self {
        self.policy_mut().set_no_delay(name);
        self
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P: Policy + PolicySettings,
{
    /// Enable [`CombinedOption`](UserStyle::CombinedOption) option set style.
    /// This can support option style like `-abc` which set `-a`, `-b` and `-c` both.
    pub fn enable_combined(&mut self) -> &mut Self {
        self.style_manager_mut().push(UserStyle::CombinedOption);
        self
    }

    /// Enable [`EmbeddedValuePlus`](UserStyle::EmbeddedValuePlus) option set style.
    /// This can support option style like `--opt42` which set `--opt` value to 42.
    /// In default the [`EmbeddedValue`](UserStyle::EmbeddedValue) style only support
    /// one letter option such as `-i`.
    pub fn enable_embedded_plus(&mut self) -> &mut Self {
        self.style_manager_mut().push(UserStyle::EmbeddedValuePlus);
        self
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P: Policy,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    P::Inv<'a>: HandlerCollection<'a, P::Set, P::Ser>,
{
    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::CoteApp;
    /// use aopt::Error;
    /// use aopt::prelude::*;
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut cote = CoteApp::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_mut_with(["-a", "-b", "42"].into_iter(), move |ret, cote| {
    ///         if ret.status() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // cote still avilable here, CoteApp::run_mut_with pass mutable reference to closure.
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
        F: FnMut(P::Ret, &'b mut Self) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;

        // initialize the option value
        parser.init()?;

        let ret = self.parse(aopt::ARef::new(Args::from(args)))?;

        r(ret, self)
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub fn run_mut<'c, 'b, R, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        F: FnMut(P::Ret, &'b mut Self) -> Result<R, Error>,
    {
        let args = Args::from_env().into_inner();
        self.run_mut_with(args.into_iter(), r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::CoteApp;
    /// use aopt::Error;
    /// use aopt::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    ///     let mut cote = CoteApp::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_async_mut_with(["-a", "-b", "42"].into_iter(), |ret, cote| async move {
    ///         if ret.status() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // cote still avilable here, CoteApp::run_async_mut_with pass mutable reference to closure.
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
        F: FnMut(P::Ret, &'b mut Self) -> FUT,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        // initialize the option value
        parser.init()?;
        match self.parse(aopt::ARef::new(Args::from(args))) {
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
        F: FnMut(P::Ret, &'b mut Self) -> FUT,
    {
        let args = Args::from_env().into_inner();
        self.run_async_mut_with(args.into_iter(), r).await
    }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::CoteApp;
    /// use aopt::Error;
    /// use aopt::prelude::*;
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut cote = CoteApp::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_with(["-a", "-b", "42"].into_iter(), move |ret, cote| {
    ///         if ret.status() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // cote still avilable here, CoteApp::run_with pass reference to closure.
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
        F: FnMut(P::Ret, &'b Self) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;

        // initialize the option value
        parser.init()?;

        let ret = self.parse(aopt::ARef::new(Args::from(args)))?;

        r(ret, self)
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub fn run<'c, 'b, R, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        F: FnMut(P::Ret, &'b Self) -> Result<R, Error>,
    {
        let args = Args::from_env().into_inner();
        self.run_with(args.into_iter(), r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::CoteApp;
    /// use aopt::Error;
    /// use aopt::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    ///     let mut cote = CoteApp::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_async_with(["-a", "-b", "42"].into_iter(), |ret, cote| async move {
    ///         if ret.status() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // cote still avilable here, CoteApp::run_async_with pass reference to closure.
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
        F: FnMut(P::Ret, &'b Self) -> FUT,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        // initialize the option value
        parser.init()?;
        match self.parse(aopt::ARef::new(Args::from(args))) {
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
        F: FnMut(P::Ret, &'b Self) -> FUT,
    {
        let args = Args::from_env().into_inner();
        self.run_async_with(args.into_iter(), r).await
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

    global.add_block(Block::new("command", "<COMMAND>", "", "Commands:", ""))?;
    global.add_block(Block::new("option", "", "", "Options:", ""))?;
    global.add_block(Block::new("args", "[ARGS]", "", "Args:", ""))?;
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

/// Display help message of [`CoteApp`] generate from `Cargo.toml`.
/// The `head` will be generate from package's description.
/// The `foot` will be generate from package's authors and version.
/// Default option text width is 20, and default usage width is 10.
#[macro_export]
macro_rules! display_help {
    ($cote:ident) => {{
        $cote.display_help(
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_DESCRIPTION"),
        )
    }};
}

/// Display help message of [`CoteApp`] generate from `Cargo.toml`.
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
        .map_err(|e| aopt::raise_error!("Can not show help message: {:?}", e))
    }};
}
