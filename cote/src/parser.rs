use std::borrow::Cow;
use std::ops::Deref;
use std::ops::DerefMut;

use aopt::ctx::Ctx;
use aopt::ctx::HandlerEntry;
use aopt::error;
use aopt::parser::AppServices;
use aopt::parser::AppStorage;
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
use aopt::prelude::Set;
use aopt::prelude::SetCfg;
use aopt::prelude::SetOpt;
use aopt::set::PrefixedValidator;
use aopt::set::SetValueFindExt;
use aopt::Error;
use aopt::Uid;

use crate::help::HelpDisplay;
use crate::prelude::HelpContext;
use crate::rctx::RunningCtx;
use crate::ExtractFromSetDerive;
///
/// A [`Parser`] using for generate code for struct.
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
///    let mut parser = Cli::into_parser()?;
///    let mut policy = Cli::into_policy();
///
///    let ret = parser.parse_policy(Args::from(["app", "list"]), &mut policy);
///
///    assert!(ret.is_ok());
///    Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Parser<'a, S> {
    name: String,
    set: S,
    rctx: RunningCtx,
    app_ser: AppServices,
    inv: Option<Invoker<'a, Self>>,
    sub_parsers: Vec<Self>,
}

impl<S> Default for Parser<'_, S>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            name: String::from("CoteParser"),
            set: Default::default(),
            rctx: RunningCtx::default(),
            app_ser: AppServices::default(),
            inv: Some(Invoker::default()),
            sub_parsers: Default::default(),
        }
    }
}

impl<'a, S> Parser<'a, S> {
    pub fn new(name: impl Into<String>, set: S) -> Self {
        Self {
            name: name.into(),
            set,
            rctx: RunningCtx::default(),
            app_ser: AppServices::default(),
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

    pub fn optset(&self) -> &S {
        &self.set
    }

    pub fn optset_mut(&mut self) -> &mut S {
        &mut self.set
    }

    pub fn set_optset(&mut self, set: S) -> &mut Self {
        self.set = set;
        self
    }

    pub fn service(&self) -> &AppServices {
        &self.app_ser
    }

    pub fn service_mut(&mut self) -> &mut AppServices {
        &mut self.app_ser
    }

    pub fn set_service(&mut self, ser: AppServices) -> &mut Self {
        self.app_ser = ser;
        self
    }

    #[doc(hidden)]
    pub fn transfer_appser_to_subparser(&mut self, index: usize) {
        let appser = std::mem::take(&mut self.app_ser);

        self.sub_parsers[index].set_service(appser);
    }

    #[doc(hidden)]
    pub fn transfer_appser_from_subparser(&mut self, index: usize) {
        let appser = std::mem::take(self.sub_parsers[index].service_mut());

        self.set_service(appser);
    }

    #[doc(hidden)]
    pub fn set_running_ctx(&mut self, rctx: RunningCtx) {
        self.rctx = rctx;
    }

    #[doc(hidden)]
    pub fn running_ctx(&mut self) -> &mut RunningCtx {
        &mut self.rctx
    }

    pub fn invoker(&self) -> &Invoker<'a, Self> {
        assert!(self.inv.is_some());
        self.inv.as_ref().unwrap()
    }

    pub fn invoker_mut(&mut self) -> &mut Invoker<'a, Self> {
        assert!(self.inv.is_some());
        self.inv.as_mut().unwrap()
    }

    pub fn set_invoker(&mut self, inv: Invoker<'a, Self>) -> &mut Self {
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
            .ok_or_else(|| aopt::error!("can not find parser at index {}", id))
    }

    pub fn parser_mut(&mut self, id: usize) -> Result<&mut Self, Error> {
        self.sub_parsers
            .get_mut(id)
            .ok_or_else(|| aopt::error!("can not find parser at index {}", id))
    }

    pub fn find_parser(&self, name: &str) -> Result<&Self, Error> {
        self.sub_parsers
            .iter()
            .find(|v| v.name() == name)
            .ok_or_else(|| aopt::error!("can not find parser named {}", name))
    }

    pub fn find_parser_mut(&mut self, name: &str) -> Result<&mut Self, Error> {
        self.sub_parsers
            .iter_mut()
            .find(|v| v.name() == name)
            .ok_or_else(|| aopt::error!("can not find parser named {}", name))
    }

    pub fn add_parser(&mut self, parser: Self) -> &mut Self {
        self.sub_parsers.push(parser);
        self
    }
}

impl<S> Deref for Parser<'_, S>
where
    S: Set,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<S> DerefMut for Parser<'_, S>
where
    S: Set,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

impl<S> Parser<'_, S>
where
    S: Set,
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

impl<'a, S> Parser<'a, S>
where
    S: Set,
{
    #[cfg(feature = "sync")]
    #[allow(clippy::type_complexity)]
    pub fn entry<O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, Invoker<'a, Self>, Self, H, O>, Error>
    where
        O: ErasedTy,
        H: FnMut(&mut Self, &mut Ctx) -> Result<Option<O>, Error> + Send + Sync + 'a,
    {
        Ok(HandlerEntry::new(self.inv.as_mut().unwrap(), uid))
    }

    #[cfg(not(feature = "sync"))]
    #[allow(clippy::type_complexity)]
    pub fn entry<O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, Invoker<'a, Self>, Self, H, O>, Error>
    where
        O: ErasedTy,
        H: FnMut(&mut Self, &mut Ctx) -> Result<Option<O>, Error> + 'a,
    {
        Ok(HandlerEntry::new(self.inv.as_mut().unwrap(), uid))
    }
}

impl<'a, 'b, S> Parser<'a, S>
where
    'a: 'b,
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn extract_type<T>(&'b mut self) -> Result<T, Error>
    where
        T: ExtractFromSetDerive<'b, S>,
    {
        let set = self.optset_mut();

        T::try_extract(set)
    }
}

impl<S> Set for Parser<'_, S>
where
    S: Set,
{
    type Ctor = S::Ctor;

    fn register(&mut self, ctor: Self::Ctor) -> Option<Self::Ctor> {
        S::register(&mut self.set, ctor)
    }

    fn get_ctor(&self, name: &str) -> Option<&Self::Ctor> {
        S::get_ctor(&self.set, name)
    }

    fn get_ctor_mut(&mut self, name: &str) -> Option<&mut Self::Ctor> {
        S::get_ctor_mut(&mut self.set, name)
    }

    fn reset(&mut self) {
        S::reset(&mut self.set)
    }

    fn len(&self) -> usize {
        S::len(&self.set)
    }

    fn iter(&self) -> std::slice::Iter<'_, SetOpt<Self>> {
        S::iter(&self.set)
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<'_, SetOpt<Self>> {
        S::iter_mut(&mut self.set)
    }

    fn insert(&mut self, opt: SetOpt<Self>) -> Uid {
        S::insert(&mut self.set, opt)
    }
}

impl<S> OptParser for Parser<'_, S>
where
    S: OptParser,
{
    type Output = S::Output;

    type Error = S::Error;

    fn parse_opt(&self, pattern: &str) -> Result<Self::Output, Self::Error> {
        OptParser::parse_opt(&self.set, pattern)
    }
}

impl<S> OptValidator for Parser<'_, S>
where
    S: OptValidator,
{
    type Error = S::Error;

    fn check(&mut self, name: &str) -> Result<bool, Self::Error> {
        OptValidator::check(&mut self.set, name)
    }

    fn split<'b>(&self, name: &Cow<'b, str>) -> Result<(Cow<'b, str>, Cow<'b, str>), Self::Error> {
        OptValidator::split(&self.set, name)
    }
}

impl<S> PrefixedValidator for Parser<'_, S>
where
    S: PrefixedValidator,
{
    type Error = S::Error;

    fn reg_prefix(&mut self, val: &str) -> Result<(), Self::Error> {
        PrefixedValidator::reg_prefix(&mut self.set, val)
    }

    fn unreg_prefix(&mut self, val: &str) -> Result<(), Self::Error> {
        PrefixedValidator::unreg_prefix(&mut self.set, val)
    }
}

impl<S> SetValueFindExt for Parser<'_, S>
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
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

impl<'a, P, S> PolicyParser<P> for Parser<'a, S>
where
    S: Set + OptParser + OptValidator,
    P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
{
    type Error = Error;

    fn parse_policy(
        &mut self,
        args: Args,
        policy: &mut P,
    ) -> Result<<P as Policy>::Ret, Self::Error> {
        assert!(self.inv.is_some());

        self.init()?;

        let mut inv = self.inv.take().unwrap();

        let ret = policy.parse(self, &mut inv, args).map_err(Into::into);

        self.inv = Some(inv);

        ret
    }
}

impl<S> AppStorage for Parser<'_, S> {
    fn set_app_data<T: ErasedTy>(&mut self, val: T) -> Option<T> {
        AppStorage::set_app_data(&mut self.app_ser, val)
    }

    fn app_data<T: ErasedTy>(&self) -> Result<&T, Error> {
        AppStorage::app_data(&self.app_ser)
    }

    fn app_data_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        AppStorage::app_data_mut(&mut self.app_ser)
    }

    fn take_app_data<T: ErasedTy>(&mut self) -> Result<T, Error> {
        AppStorage::take_app_data(&mut self.app_ser)
    }
}

impl<'a, S> Parser<'a, S>
where
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
    S: Set + OptValidator + OptParser<Output: Information>,
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
    ///     let mut parser = Parser::<CoteSet>::default().with_name("example");
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
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
        F: FnMut(P::Ret, &mut Self) -> Result<R, Error>,
    {
        let ret = self.parse_policy(args.into(), policy)?;

        r(ret, self)
    }

    /// Call [`run_mut_with`](Parser::run_mut_with) with default arguments [`args()`](std::env::args).
    pub fn run_mut<R, F, P>(&mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
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
    ///     let mut parser = Parser::<CoteSet>::default().with_name("example");
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
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
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
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
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
    ///     let mut parser = Parser::<CoteSet>::default().with_name("example");
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
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
        F: FnMut(P::Ret, &Self) -> Result<R, Error>,
    {
        let ret = self.parse_policy(args.into(), policy)?;

        r(ret, self)
    }

    /// Call [`run_with`](Self::run_with) with default arguments [`args()`](std::env::args).
    pub fn run<R, F, P>(&mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
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
    ///     let mut parser = Parser::<CoteSet>::default().with_name("example");
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
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
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
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
    {
        self.run_async_with(Args::from_env(), policy, r).await
    }
}

impl<S: Set> HelpDisplay<S> for Parser<'_, S> {
    type Error = crate::Error;

    fn display(&self, ctx: HelpContext) -> Result<(), Self::Error> {
        let set = self.optset();
        let name = ctx.name();
        let head = ctx.head();
        let foot = ctx.foot();
        let max_width = ctx.width();
        let usage_width = ctx.usagew();

        crate::help::display_set_help(set, name, head, foot, max_width, usage_width)
            .map_err(|e| aopt::error!("Can not show help message: {:?}", e))
    }

    fn display_sub(&self, names: Vec<&str>, ctx: &HelpContext) -> Result<(), Self::Error> {
        self.display_sub_help(names, ctx)
    }
}

impl<S> Parser<'_, S>
where
    S: Set,
{
    fn display_sub_help(&self, names: Vec<&str>, ctx: &HelpContext) -> Result<(), Error> {
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

                    return {
                        let head = ctx.head();
                        let foot = ctx.foot();
                        let max_width = ctx.width();
                        let usage_width = ctx.usagew();

                        crate::help::display_set_help(
                            optset,
                            name,
                            head,
                            foot,
                            max_width,
                            usage_width,
                        )
                        .map_err(|e| aopt::error!("Can not show help message: {:?}", e))
                    };
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
        Err(error!(
            "Can not display help message for names `{names:?}` with context: {ctx:?}"
        ))
    }
}

#[cfg(feature = "shell")]
pub(crate) mod shell {
    use std::borrow::Cow;

    use aopt::prelude::ConfigValue;
    use aopt::prelude::Opt;
    use aopt::prelude::OptValidator;
    use aopt::prelude::Set;
    use aopt::prelude::SetCfg;
    use aopt::prelude::SetOpt;
    use aopt::prelude::SetValueFindExt;
    use aopt::prelude::Style;
    use aopt::shell::shell;
    use aopt::shell::shell::complete_eq;
    use aopt::shell::shell::complete_opt;
    use aopt::shell::shell::complete_val;
    use aopt::shell::shell::Complete;
    use aopt::shell::shell::Shell;
    use aopt::shell::value::Values;
    use aopt::shell::Context;
    use aopt::trace;
    use aopt::HashMap;
    use aopt::Uid;

    use crate::parser::Parser;
    use crate::Error;

    pub struct CompletionManager<'a, S>
    where
        S: Set,
    {
        parser: Parser<'a, S>,

        values: HashMap<Uid, Box<dyn Values<SetOpt<S>, Err = Error>>>,

        submanager: HashMap<String, CompletionManager<'a, S>>,
    }

    impl<'a, S> CompletionManager<'a, S>
    where
        S: Set,
        SetOpt<S>: Opt,
    {
        pub fn new(mut parser: Parser<'a, S>) -> Self {
            let mut submanager = HashMap::default();

            for sub_parser in std::mem::take(&mut parser.sub_parsers) {
                submanager.insert(sub_parser.name().to_string(), Self::new(sub_parser));
            }
            Self {
                parser,
                values: HashMap::default(),
                submanager,
            }
        }

        pub fn with_values<V>(mut self, uid: Uid, v: V) -> Self
        where
            V: Values<SetOpt<S>> + 'static,
        {
            self.set_values(uid, v);
            self
        }

        pub fn with_parser(mut self, parser: Parser<'a, S>) -> Self {
            self.parser = parser;
            self
        }

        pub fn with_manager(mut self, name: &str, parser: Parser<'a, S>) -> Result<Self, Error> {
            self.add_manager(name, parser)?;
            Ok(self)
        }

        pub fn set_parser(&mut self, parser: Parser<'a, S>) -> &mut Self {
            self.parser = parser;
            self
        }

        pub fn set_values<V>(&mut self, uid: Uid, v: V) -> &mut Self
        where
            V: Values<SetOpt<S>> + 'static,
        {
            self.values
                .insert(uid, Box::new(aopt::shell::value::wrap(v)));
            self
        }

        pub fn add_manager(
            &mut self,
            name: &str,
            parser: Parser<'a, S>,
        ) -> Result<&mut Self, Error> {
            if self
                .parser
                .iter()
                .filter(|v| v.mat_style(aopt::opt::Style::Cmd))
                .any(|v| v.name() == name)
            {
                self.submanager
                    .insert(name.to_string(), CompletionManager::new(parser));
                Ok(self)
            } else {
                Err(aopt::error!("not a sub command name: {name}"))
            }
        }

        pub fn parser(&self) -> &Parser<'a, S> {
            &self.parser
        }

        pub fn parser_mut(&mut self) -> &mut Parser<'a, S> {
            &mut self.parser
        }

        pub fn values(&self) -> &HashMap<Uid, Box<dyn Values<SetOpt<S>, Err = Error>>> {
            &self.values
        }

        pub fn managers(&self) -> &HashMap<String, CompletionManager<'a, S>> {
            &self.submanager
        }

        pub fn managers_mut(&mut self) -> &mut HashMap<String, CompletionManager<'a, S>> {
            &mut self.submanager
        }

        pub fn find_manager(&self, name: &str) -> Result<&CompletionManager<'a, S>, Error> {
            self.submanager
                .get(name)
                .ok_or_else(|| aopt::error!("can not find manager: {name}"))
        }

        pub fn find_manager_mut(
            &mut self,
            name: &str,
        ) -> Result<&mut CompletionManager<'a, S>, Error> {
            self.submanager
                .get_mut(name)
                .ok_or_else(|| aopt::error!("can not find manager: {name}"))
        }
    }

    impl<'a, S> Complete<SetOpt<S>> for CompletionManager<'a, S>
    where
        SetOpt<S>: Opt,
        SetCfg<S>: ConfigValue + Default,
        S: Set + OptValidator + SetValueFindExt,
    {
        type Out = ();
        type Ctx<'b> = Context<'b>;
        type Err = Error;

        fn complete<T, W>(&self, s: &mut T, ctx: &mut Self::Ctx<'_>) -> Result<Self::Out, Self::Err>
        where
            T: Shell<SetOpt<S>, W>,
        {
            let Context {
                args,
                curr,
                prev,
                cword,
            } = ctx;

            let mut incomp_arg = Cow::Borrowed(curr.as_ref());
            let mut incomp_val = None;

            if let Some((opt, val)) = crate::aopt::str::split_once(curr, '=') {
                incomp_arg = opt;
                incomp_val = Some(val);
            }

            trace!("complete start ...",);
            trace!("curr={}", curr.display());
            trace!("prev={}", prev.display());
            trace!("arg={}", incomp_arg.display());
            trace!("val={:?}", incomp_val.as_ref().map(|v| v.display()));
            trace!("args = {:?}", args);

            let mut s = shell::wrapref(s);
            let mut manager = self;
            let mut flags = vec![false; args.len()];
            let mut cmds = vec![];
            let mut manager_list = vec![self];

            for (idx, arg) in args.iter().enumerate() {
                if let Some(arg) = arg.to_str() {
                    trace!("finding `{}`", arg);
                    for cmd in manager.parser().iter().filter(|v| v.mat_style(Style::Cmd)) {
                        trace!("checking `{}`", cmd.name());
                        if cmd.mat_name(Some(arg)) || cmd.mat_alias(arg) {
                            manager = manager.find_manager(cmd.name())?;

                            flags[idx] = true;
                            cmds.push(cmd);
                            manager_list.push(manager);
                            trace!("find cmd `{}` in args at `{}`", arg, idx);
                            break;
                        }
                    }
                }
            }

            let mut available_cmds = vec![];

            // find cmd if val is none
            if let (Some(manager), None) = (manager_list.last(), &incomp_val) {
                trace!("try complete cmd");
                let arg = incomp_arg.to_str().unwrap_or_default();
                let optset = manager.parser();

                for opt in optset.iter().filter(|v| v.mat_style(Style::Cmd)) {
                    for name in std::iter::once(opt.name())
                        .chain(
                            opt.alias()
                                .iter()
                                .flat_map(|v| v.iter().map(|v| v.as_str())),
                        )
                        .filter(|v| v.starts_with(arg))
                    {
                        trace!("available cmd -> {name}");
                        available_cmds.push((name, opt));
                    }
                }
            }

            // find option value like [arg=val]
            if let (Some(arg), Some(val)) = (incomp_arg.to_str(), incomp_val.as_ref()) {
                let bytes = val.as_encoded_bytes();

                trace!("search.1 vals with arg=`{}`, val=`{}`", arg, val.display());
                for manager in manager_list
                    .iter()
                    .filter(|v| v.parser().split(&Cow::Borrowed(arg)).is_ok())
                {
                    let optset = manager.parser();
                    let values = manager.values();

                    complete_eq(arg, bytes, optset.iter(), values, |name, val, opt| {
                        s.write_eq(name, val, opt)
                    })?;
                }
            }

            let mut found_val = false;

            // find option value like [arg val]
            if let (Some(arg), Some(val)) = (prev.to_str(), Some(&curr)) {
                let bytes = val.as_encoded_bytes();

                trace!("search.2 vals with arg=`{}`, val=`{}`", arg, val.display());
                for manager in manager_list
                    .iter()
                    .filter(|v| v.parser().split(&Cow::Borrowed(arg)).is_ok())
                {
                    let optset = manager.parser();
                    let values = manager.values();

                    found_val = found_val
                        || complete_val(arg, bytes, optset.iter(), values, |val, opt| {
                            s.write_val(val, opt)
                        })?;
                }
            }

            // if we not found any val, print cmd if available
            if !found_val && !available_cmds.is_empty() {
                for (cmd, opt) in available_cmds {
                    s.write_cmd(cmd, opt)?;
                }
                return s.finish();
            }

            let mut found_opt = false;

            // find option if val is none
            if let (Some(arg), None) = (incomp_arg.to_str(), incomp_val) {
                trace!("search option with arg=`{}`", arg);
                for p in manager_list
                    .iter()
                    .map(|v| v.parser())
                    .filter(|v| v.split(&Cow::Borrowed(arg)).is_ok())
                {
                    found_opt = found_opt
                        || complete_opt(arg, p.iter(), |name, opt| s.write_opt(name, opt))?;
                }
            }

            // if we not found any opt
            if !found_opt && !found_val {
                // complete pos value in last manager
                if let Some(manager) = manager_list.last() {
                    let optset = manager.parser();

                    if optset.iter().any(|v| v.mat_style(Style::Pos)) {
                        let values = manager.values();
                        let mut noa_index = if cmds.is_empty() { 1 } else { 2 };
                        let mut index = 0;

                        while index < args.len() && index < *cword {
                            if !flags[index] {
                                // check if current is option
                                let mut like_opt = false;
                                let mut found_opt = false;
                                let mut need_val = false;
                                let (mut arg, mut val) =
                                    (Cow::Borrowed(args[index].as_os_str()), None);

                                if let Some((a, b)) =
                                    crate::aopt::str::split_once(&args[index], '=')
                                {
                                    arg = a;
                                    val = Some(b);
                                }
                                // if arg is valid str
                                if let Some(arg) = arg.to_str() {
                                    // like an option
                                    for p in manager_list
                                        .iter()
                                        .map(|v| v.parser())
                                        .filter(|v| v.split(&Cow::Borrowed(arg)).is_ok())
                                    {
                                        like_opt = true;

                                        for opt in p.iter() {
                                            if opt.mat_name(Some(arg)) || opt.mat_alias(arg) {
                                                found_opt = true;
                                                if opt.mat_style(Style::Argument) && val.is_none() {
                                                    need_val = true;
                                                }
                                                break;
                                            }
                                        }
                                        if found_opt {
                                            break;
                                        }
                                    }
                                }
                                if !like_opt {
                                    noa_index += 1;
                                }
                                if need_val && val.is_none() {
                                    index += 1;
                                }
                            }
                            index += 1;
                        }

                        let bytes = curr.as_encoded_bytes();

                        for pos in optset.iter().filter(|v| v.mat_style(Style::Pos)) {
                            if pos.mat_index(Some((noa_index, noa_index + 1))) {
                                if let Some(getter) = values.get(&pos.uid()) {
                                    for val in getter.get_values(pos)? {
                                        if !val.is_empty() && bytes.is_empty()
                                            || bytes
                                                .iter()
                                                .zip(val.as_encoded_bytes())
                                                .all(|(a, b)| *a == *b)
                                        {
                                            trace!("available pos value -> {}", val.display());
                                            s.write_val(val.as_os_str(), pos)?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            s.finish()
        }
    }
}
