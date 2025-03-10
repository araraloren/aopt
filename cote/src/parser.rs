use std::borrow::Cow;
use std::ops::Deref;
use std::ops::DerefMut;

use aopt::ctx::Ctx;
use aopt::ctx::HandlerEntry;
use aopt::prelude::Args;
use aopt::prelude::ConfigBuild;
use aopt::prelude::ConfigValue;
use aopt::prelude::ErasedTy;
use aopt::prelude::Information;
use aopt::prelude::Invoker;
use aopt::prelude::Opt;
use aopt::prelude::OptParser;
use aopt::prelude::OptValidator;
use aopt::prelude::Policy;
use aopt::prelude::PolicyParser;
use aopt::prelude::SetCfg;
use aopt::prelude::SetOpt;
use aopt::raise_error;
use aopt::set::PrefixedValidator;
use aopt::set::SetValueFindExt;
use aopt::Error;
use aopt::Uid;

use crate::prelude::HelpContext;
use crate::ExtractFromSetDerive;
///
/// # Note
///
/// When the [`Parser`] has sub [`Parser`],
/// if you wish to directly use the current [`Parser`] type through the interface of [`PolicyParser`],
/// you must set up the required [`RunningCtx`](crate::prelude::RunningCtx) before invoking the interface of [`PolicyParser`].
///
/// ```
/// # use cote::prelude::*;
///
/// #[derive(Debug, Clone, Cote)]
/// struct Cli {
///     #[sub()]
///     list: Option<List>,
/// }
///
/// #[derive(Debug, Clone, Cote)]
/// struct List {}
///
/// #[tokio::main]
/// async fn main() -> color_eyre::Result<()> {
///     color_eyre::install()?;
///
///     {
///         let mut parser = Cli::into_parser()?;
///         let mut policy = Cli::into_policy();
///
///         // in sub command, the code generate by cote will access RunningCtx
///         let ret = parser.parse_policy(Args::from(["app", "list"]), &mut policy);
///
///         assert!(ret.is_err());
///     }
///     {
///         let mut parser = Cli::into_parser()?;
///         let mut policy = Cli::into_policy();
///         let rctx = RunningCtx::default().with_name(parser.name().clone());
///
///         // insert a RunningCtx before parse
///         parser.service_mut().sve_insert(rctx);
///         let ret = parser.parse_policy(Args::from(["app", "list"]), &mut policy);
///
///         assert!(ret.is_ok());
///     }
///
///    Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Parser<'a, Set, Ser> {
    name: String,
    set: Set,
    ser: Option<Ser>,
    inv: Option<Invoker<'a, Self, Ser>>,
    sub_parsers: Vec<Self>,
}

impl<Set, Ser> Default for Parser<'_, Set, Ser>
where
    Set: Default,
    Ser: Default,
{
    fn default() -> Self {
        Self {
            name: String::from("CoteParser"),
            set: Default::default(),
            ser: Some(Ser::default()),
            inv: Some(Invoker::default()),
            sub_parsers: Default::default(),
        }
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser> {
    pub fn new(name: impl Into<String>, set: Set) -> Self {
        Self {
            name: name.into(),
            set,
            ser: None,
            inv: None,
            sub_parsers: vec![],
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn optset(&self) -> &Set {
        &self.set
    }

    pub fn optset_mut(&mut self) -> &mut Set {
        &mut self.set
    }

    pub fn set_optset(&mut self, set: Set) -> &mut Self {
        self.set = set;
        self
    }

    pub fn service(&self) -> &Ser {
        assert!(self.ser.is_some());
        self.ser.as_ref().unwrap()
    }

    pub fn service_mut(&mut self) -> &mut Ser {
        assert!(self.ser.is_some());
        self.ser.as_mut().unwrap()
    }

    pub fn set_service(&mut self, ser: Ser) -> &mut Self {
        self.ser = Some(ser);
        self
    }

    pub fn invoker(&self) -> &Invoker<'a, Self, Ser> {
        assert!(self.inv.is_some());
        self.inv.as_ref().unwrap()
    }

    pub fn invoker_mut(&mut self) -> &mut Invoker<'a, Self, Ser> {
        assert!(self.inv.is_some());
        self.inv.as_mut().unwrap()
    }

    pub fn set_invoker(&mut self, inv: Invoker<'a, Self, Ser>) -> &mut Self {
        self.inv = Some(inv);
        self
    }

    pub fn parsers(&self) -> &[Self] {
        &self.sub_parsers
    }

    pub fn parsers_mut(&mut self) -> &mut [Self] {
        &mut self.sub_parsers
    }

    pub fn set_parsers(&mut self, parsers: Vec<Self>) -> &mut Self {
        self.sub_parsers = parsers;
        self
    }

    pub fn parser(&self, id: usize) -> Result<&Self, Error> {
        self.sub_parsers
            .get(id)
            .ok_or_else(|| aopt::raise_error!("can not find parser at index {}", id))
    }

    pub fn parser_mut(&mut self, id: usize) -> Result<&mut Self, Error> {
        self.sub_parsers
            .get_mut(id)
            .ok_or_else(|| aopt::raise_error!("can not find parser at index {}", id))
    }

    pub fn find_parser(&self, name: &str) -> Result<&Self, Error> {
        self.sub_parsers
            .iter()
            .find(|v| v.name() == name)
            .ok_or_else(|| aopt::raise_error!("can not find parser named {}", name))
    }

    pub fn find_parser_mut(&mut self, name: &str) -> Result<&mut Self, Error> {
        self.sub_parsers
            .iter_mut()
            .find(|v| v.name() == name)
            .ok_or_else(|| aopt::raise_error!("can not find parser named {}", name))
    }

    pub fn add_parser(&mut self, parser: Self) -> &mut Self {
        self.sub_parsers.push(parser);
        self
    }
}

impl<Set, Ser> Deref for Parser<'_, Set, Ser>
where
    Set: aopt::set::Set,
{
    type Target = Set;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<Set, Ser> DerefMut for Parser<'_, Set, Ser>
where
    Set: aopt::set::Set,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

impl<Set, Ser> Parser<'_, Set, Ser>
where
    Set: aopt::set::Set,
{
    /// Reset the option set.
    pub fn reset(&mut self) -> Result<&mut Self, Error> {
        self.optset_mut().reset();
        Ok(self)
    }

    /// Call the [`init`](crate::prelude::Opt::init) of [`Opt`](crate::prelude::Opt) initialize the option value.
    pub fn init(&mut self) -> Result<(), Error> {
        let optset = self.optset_mut();

        for opt in optset.iter_mut() {
            opt.init()?;
        }
        Ok(())
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser>
where
    Set: aopt::set::Set,
{
    #[cfg(feature = "sync")]
    #[allow(clippy::type_complexity)]
    pub fn entry<O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, Invoker<'a, Self, Ser>, Self, Ser, H, O>, Error>
    where
        O: ErasedTy,
        H: FnMut(&mut Self, &mut Ser, &Ctx) -> Result<Option<O>, Error> + Send + Sync + 'a,
    {
        Ok(HandlerEntry::new(self.inv.as_mut().unwrap(), uid))
    }

    #[cfg(not(feature = "sync"))]
    #[allow(clippy::type_complexity)]
    pub fn entry<O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, Invoker<'a, Self, Ser>, Self, Ser, H, O>, Error>
    where
        O: ErasedTy,
        H: FnMut(&mut Self, &mut Ser, &Ctx) -> Result<Option<O>, Error> + 'a,
    {
        Ok(HandlerEntry::new(self.inv.as_mut().unwrap(), uid))
    }
}

impl<'a, 'b, Set, Ser> Parser<'a, Set, Ser>
where
    'a: 'b,
    Set: SetValueFindExt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn extract_type<T>(&'b mut self) -> Result<T, Error>
    where
        T: ExtractFromSetDerive<'b, Set>,
    {
        let set = self.optset_mut();

        T::try_extract(set)
    }
}

impl<Set, Ser> aopt::set::Set for Parser<'_, Set, Ser>
where
    Set: aopt::set::Set,
{
    type Ctor = Set::Ctor;

    fn register(&mut self, ctor: Self::Ctor) -> Option<Self::Ctor> {
        Set::register(&mut self.set, ctor)
    }

    fn get_ctor(&self, name: &str) -> Option<&Self::Ctor> {
        Set::get_ctor(&self.set, name)
    }

    fn get_ctor_mut(&mut self, name: &str) -> Option<&mut Self::Ctor> {
        Set::get_ctor_mut(&mut self.set, name)
    }

    fn reset(&mut self) {
        Set::reset(&mut self.set)
    }

    fn len(&self) -> usize {
        Set::len(&self.set)
    }

    fn iter(&self) -> std::slice::Iter<'_, SetOpt<Self>> {
        Set::iter(&self.set)
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<'_, SetOpt<Self>> {
        Set::iter_mut(&mut self.set)
    }

    fn insert(&mut self, opt: SetOpt<Self>) -> Uid {
        Set::insert(&mut self.set, opt)
    }
}

impl<Set, Ser> OptParser for Parser<'_, Set, Ser>
where
    Set: OptParser,
{
    type Output = Set::Output;

    type Error = Set::Error;

    fn parse_opt(&self, pattern: &str) -> Result<Self::Output, Self::Error> {
        OptParser::parse_opt(&self.set, pattern)
    }
}

impl<Set, Ser> OptValidator for Parser<'_, Set, Ser>
where
    Set: OptValidator,
{
    type Error = Set::Error;

    fn check(&mut self, name: &str) -> Result<bool, Self::Error> {
        OptValidator::check(&mut self.set, name)
    }

    fn split<'b>(&self, name: &Cow<'b, str>) -> Result<(Cow<'b, str>, Cow<'b, str>), Self::Error> {
        OptValidator::split(&self.set, name)
    }
}

impl<Set, Ser> PrefixedValidator for Parser<'_, Set, Ser>
where
    Set: PrefixedValidator,
{
    type Error = Set::Error;

    fn reg_prefix(&mut self, val: &str) -> Result<(), Self::Error> {
        PrefixedValidator::reg_prefix(&mut self.set, val)
    }

    fn unreg_prefix(&mut self, val: &str) -> Result<(), Self::Error> {
        PrefixedValidator::unreg_prefix(&mut self.set, val)
    }
}

impl<Set, Ser> SetValueFindExt for Parser<'_, Set, Ser>
where
    Set: SetValueFindExt,
    SetCfg<Set>: ConfigValue + Default,
{
    fn find_uid(&self, cb: impl ConfigBuild<SetCfg<Self>>) -> Result<Uid, Error> {
        SetValueFindExt::find_uid(&self.set, cb)
    }

    fn find_opt(&self, cb: impl ConfigBuild<SetCfg<Self>>) -> Result<&SetOpt<Self>, Error> {
        SetValueFindExt::find_opt(&self.set, cb)
    }

    fn find_opt_mut(
        &mut self,
        cb: impl ConfigBuild<SetCfg<Self>>,
    ) -> Result<&mut SetOpt<Self>, Error> {
        SetValueFindExt::find_opt_mut(&mut self.set, cb)
    }
}

impl<'a, P, Set, Ser> PolicyParser<P> for Parser<'a, Set, Ser>
where
    Set: aopt::set::Set + OptParser + OptValidator,
    P: Policy<Set = Self, Ser = Ser, Inv<'a> = Invoker<'a, Self, Ser>>,
{
    type Error = Error;

    fn parse_policy(
        &mut self,
        args: Args,
        policy: &mut P,
    ) -> Result<<P as Policy>::Ret, Self::Error> {
        assert!(self.inv.is_some());
        assert!(self.ser.is_some());

        self.init()?;

        let mut inv = self.inv.take().unwrap();
        let mut ser = self.ser.take().unwrap();

        let ret = policy
            .parse(self, &mut inv, &mut ser, args)
            .map_err(Into::into);

        self.inv = Some(inv);
        self.ser = Some(ser);

        ret
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser>
where
    SetOpt<Set>: Opt,
    Set: aopt::set::Set + OptValidator + OptParser,
    <Set as OptParser>::Output: Information,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Running function after parsing.
    ///
    /// # Example
    ///
    ///```rust
    /// # use aopt::Error;
    /// # use cote::prelude::*;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     let mut policy = FwdPolicy::default();
    ///     let mut parser = Parser::<CoteSet, CoteSer>::default().with_name("example");
    ///
    ///     parser.add_opt("-a!".infer::<bool>())?;
    ///     parser.add_opt("-b".infer::<i64>())?;
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
    pub fn run_mut_with<R, F, P>(
        &mut self,
        args: impl Into<Args>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
        F: FnMut(P::Ret, &mut Self) -> Result<R, Error>,
    {
        let ret = self.parse_policy(args.into(), policy)?;

        r(ret, self)
    }

    /// Call [`run_mut_with`](Parser::run_mut_with) with default arguments [`args()`](std::env::args).
    pub fn run_mut<R, F, P>(&mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
        F: FnMut(P::Ret, &mut Self) -> Result<R, Error>,
    {
        self.run_mut_with(Args::from_env(), policy, r)
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
    ///     let mut policy = FwdPolicy::default();
    ///     let mut parser = Parser::<CoteSet, CoteSer>::default().with_name("example");
    ///
    ///     parser.add_opt("-a!".infer::<bool>())?;
    ///     parser.add_opt("-b".infer::<i64>())?;
    ///
    ///     parser
    ///         .run_async_mut_with(
    ///             ["-a", "-b", "42"],
    ///             &mut policy,
    ///             async |ret, parser| {
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
    pub async fn run_async_mut_with<R, F, P>(
        &mut self,
        args: impl Into<Args>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        F: AsyncFnMut(P::Ret, &mut Self) -> Result<R, Error>,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
    {
        match self.parse_policy(args.into(), policy) {
            Ok(ret) => r(ret, self).await,
            Err(e) => Err(e),
        }
    }

    /// Call [`run_async_mut_with`](Self::run_async_mut_with) with default arguments [`args()`](std::env::args).
    pub async fn run_async_mut<R, F, P>(&mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        F: AsyncFnMut(P::Ret, &mut Self) -> Result<R, Error>,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
    {
        self.run_async_mut_with(Args::from_env(), policy, r).await
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
    ///     let mut policy = FwdPolicy::default();
    ///     let mut parser = Parser::<CoteSet, CoteSer>::default().with_name("example");
    ///
    ///     parser.add_opt("-a!".infer::<bool>())?;
    ///     parser.add_opt("-b".infer::<i64>())?;
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
    pub fn run_with<R, F, P>(
        &mut self,
        args: impl Into<Args>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
        F: FnMut(P::Ret, &Self) -> Result<R, Error>,
    {
        let ret = self.parse_policy(args.into(), policy)?;

        r(ret, self)
    }

    /// Call [`run_with`](Self::run_with) with default arguments [`args()`](std::env::args).
    pub fn run<R, F, P>(&mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
        F: FnMut(P::Ret, &Self) -> Result<R, Error>,
    {
        self.run_with(Args::from_env(), policy, r)
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
    ///     let mut policy = FwdPolicy::default();
    ///     let mut parser = Parser::<CoteSet, CoteSer>::default().with_name("example");
    ///
    ///     parser.add_opt("-a!".infer::<bool>())?;
    ///     parser.add_opt("-b".infer::<i64>())?;
    ///
    ///     parser
    ///         .run_async_with(
    ///             ["-a", "-b", "42"].into_iter(),
    ///             &mut policy,
    ///             async |ret, parser| {
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
    pub async fn run_async_with<R, F, P>(
        &mut self,
        args: impl Into<Args>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        F: AsyncFnMut(P::Ret, &Self) -> Result<R, Error>,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
    {
        match self.parse_policy(args.into(), policy) {
            Ok(ret) => r(ret, self).await,
            Err(e) => Err(e),
        }
    }

    /// Call [`run_async_with`](Self::run_async_with) with default arguments [`args()`](std::env::args).
    pub async fn run_async<R, F, P>(&mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        F: AsyncFnMut(P::Ret, &Self) -> Result<R, Error>,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
    {
        self.run_async_with(Args::from_env(), policy, r).await
    }
}

impl<Set, Ser> Parser<'_, Set, Ser>
where
    Set: aopt::set::Set,
{
    pub const DEFAULT_OPTION_WIDTH: usize = 40;
    pub const DEFAULT_USAGE_WIDTH: usize = 10;

    pub fn display_help(
        &self,
        author: &str,
        version: &str,
        description: &str,
    ) -> Result<(), Error> {
        let set = self.optset();
        let name = self.name.as_str();

        crate::display_help!(
            set,
            name,
            author,
            version,
            description,
            Self::DEFAULT_OPTION_WIDTH,
            Self::DEFAULT_USAGE_WIDTH
        )
    }

    pub fn display_help_ctx(&self, ctx: HelpContext) -> Result<(), Error> {
        let set = self.optset();

        crate::display_help!(
            set,
            ctx.name(),
            ctx.head(),
            ctx.foot(),
            ctx.width(),
            ctx.usagew()
        )
    }

    pub fn display_sub_help(&self, names: Vec<&str>, ctx: &HelpContext) -> Result<(), Error> {
        self.display_sub_help_impl(names, ctx, 0)
    }

    fn display_sub_help_impl(
        &self,
        names: Vec<&str>,
        ctx: &HelpContext,
        i: usize,
    ) -> Result<(), Error> {
        if !names.is_empty() {
            let max = names.len() - 1;

            if let Some(name) = names.get(i) {
                if i == max && (i > 0 || name == self.name()) {
                    let name = names.join(" ");
                    let optset = self.optset();

                    return crate::display_help!(
                        optset,
                        &name,
                        ctx.head(),
                        ctx.foot(),
                        ctx.width(),
                        ctx.usagew()
                    );
                } else if i < max && name == self.name() {
                    if let Some(name) = names.get(i + 1) {
                        let sub_parsers = self.parsers();

                        for sub_parser in sub_parsers {
                            if sub_parser.name() == name {
                                return sub_parser.display_sub_help_impl(names, ctx, i + 1);
                            }
                        }
                    }
                }
            }
        }
        Err(raise_error!(
            "can not display help message for names `{names:?}` with context: {ctx:?}"
        ))
    }
}

impl<Set, Ser> Parser<'_, Set, Ser>
where
    Set: SetValueFindExt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn display_help_if(
        &self,
        option: impl ConfigBuild<SetCfg<Set>>,
        author: &str,
        version: &str,
        description: &str,
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

    pub fn display_help_if_ctx(
        &self,
        option: impl ConfigBuild<SetCfg<Set>>,
        ctx: &HelpContext,
    ) -> Result<bool, Error> {
        let set = self.optset();

        if let Ok(help_option) = set.find_val::<bool>(option) {
            if *help_option {
                let set = self.optset();

                crate::help::display_set_help(
                    set,
                    ctx.name(),
                    ctx.head(),
                    ctx.foot(),
                    ctx.width(),
                    ctx.usagew(),
                )
                .map_err(|e| aopt::raise_error!("can not show help message: {:?}", e))?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn display_help_if_width(
        &self,
        option: impl ConfigBuild<SetCfg<Set>>,
        author: &str,
        version: &str,
        description: &str,
        option_width: usize,
        usage_width: usize,
    ) -> Result<bool, Error> {
        let set = self.optset();

        if let Ok(help_option) = set.find_val::<bool>(option) {
            if *help_option {
                let name = self.name.as_str();

                crate::display_help!(
                    set,
                    name,
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
