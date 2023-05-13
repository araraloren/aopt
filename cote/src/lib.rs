pub mod ctx;
pub mod meta;
pub mod ser;
pub mod valid;
pub mod value;

use std::fmt::Debug;
use std::future::Future;
use std::ops::{Deref, DerefMut};

use aopt::prelude::*;
use aopt::value::Placeholder;
use aopt::Error;
use aopt::RawVal;

pub mod prelude {
    pub use crate::meta::IntoConfig;
    pub use crate::meta::OptionMeta;
    pub use crate::valid;
    pub use crate::value;
    pub use crate::value::InferValueMut;
    pub use crate::CoteApp;
    pub use crate::CoteParser;
    pub use crate::ExtractFromSetDerive;
    pub use crate::IntoParserDerive;
    pub use crate::ser::CoteServiceExt;
    pub use aopt;
    pub use aopt::prelude::*;
}

use crate::meta::IntoConfig;

pub trait IntoParserDerive<Set, Inv, Ser>
where
    Set: aopt::prelude::Set,
    SetCfg<Set>: Config + ConfigValue,
{
    fn into_parser(set: Set, inv: Inv, ser: Ser) -> Result<Parser<Set, Inv, Ser>, Error> {
        let mut parser = Parser::new(set, inv, ser);
        Self::update(&mut parser)?;
        Ok(parser)
    }

    fn into_parser_with<'a, P>(policy: &P) -> Result<Parser<Set, Inv, Ser>, Error>
    where
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser> + APolicyExt<P>,
    {
        let mut parser = Parser::new_with(policy);
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

#[derive(Debug, Default)]
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
    pub fn new(name: String, set: Set, inv: Inv, ser: Ser) -> Self {
        Self {
            name,
            parser: Parser::new(set, inv, ser),
        }
    }

    pub fn new_with<'a, P>(name: String, policy: &P) -> Self
    where
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser> + APolicyExt<P>,
    {
        Self {
            name,
            parser: Parser::new_with(policy),
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

pub struct CoteApp<'a, P>
where
    P: Policy,
{
    name: String,
    parser: PolicyParser<'a, P>,
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
            .field("name", &self.name)
            .field("parser", &self.parser)
            .finish()
    }
}

impl<'a, P: Policy> Default for CoteApp<'a, P>
where
    P::Set: Default,
    P::Inv<'a>: Default,
    P::Ser: Default,
    P: Default + Policy + APolicyExt<P>,
{
    fn default() -> Self {
        Self {
            name: "CoteApp".to_owned(),
            parser: PolicyParser::default(),
        }
    }
}

impl<'a, P: Policy> Deref for CoteApp<'a, P> {
    type Target = PolicyParser<'a, P>;

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
    pub fn new(name: String, policy: P) -> Self {
        Self {
            name,
            parser: PolicyParser::new(policy),
        }
    }
}

impl<'a, P: Policy> CoteApp<'a, P> {
    pub fn new_with(name: String, policy: P, set: P::Set, inv: P::Inv<'a>, ser: P::Ser) -> Self {
        Self {
            name,
            parser: PolicyParser::new_with(policy, set, inv, ser),
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

    pub fn parser(&self) -> &PolicyParser<'a, P> {
        &self.parser
    }

    pub fn parser_mut(&mut self) -> &mut PolicyParser<'a, P> {
        &mut self.parser
    }

    pub fn set_parser(&mut self, parser: PolicyParser<'a, P>) -> &mut Self {
        self.parser = parser;
        self
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P: Policy,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser + 'a,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    P::Inv<'a>: HandlerCollection<'a, P::Set, P::Ser>,
{
    pub fn add_opt_meta(
        &mut self,
        meta: impl IntoConfig<Ret = SetCfg<P::Set>>,
    ) -> Result<ParserCommit<'a, '_, P::Inv<'a>, P::Set, P::Ser, Placeholder>, aopt::Error> {
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

        let ret = parser.parse(aopt::ARef::new(Args::from(args)))?;

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

        let ret = parser.parse(aopt::ARef::new(Args::from(args)))?;

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
        F: FnMut(P::Ret, &'b Self) -> FUT,
    {
        let args = Args::from_env().into_inner();
        self.run_async_with(args.into_iter(), r).await
    }
}
