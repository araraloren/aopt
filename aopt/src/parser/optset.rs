use std::borrow::Cow;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::args::Args;
use crate::ctx::Ctx;
use crate::ctx::HandlerEntry;
use crate::ctx::Invoker;
use crate::map::ErasedTy;
use crate::opt::ConfigBuild;
use crate::opt::ConfigValue;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::parser::ParserCommit;
use crate::parser::Policy;
use crate::error;
use crate::set::OptValidator;
use crate::set::PrefixedValidator;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetCommit;
use crate::set::SetOpt;
use crate::set::SetValueFindExt;
use crate::value::Infer;
use crate::value::Placeholder;
use crate::value::RawValParser;
use crate::Error;
use crate::Uid;

use super::AppServices;
use super::AppStorage;
use super::Parser;
use super::PolicyParser;

#[derive(Debug)]
pub struct HCOptSet<'a, S> {
    set: S,
    inv: Option<Invoker<'a, Self>>,
    ser: AppServices,
}

impl<S: Default> Default for HCOptSet<'_, S> {
    fn default() -> Self {
        Self {
            set: Default::default(),
            inv: Some(Invoker::default()),
            ser: Default::default(),
        }
    }
}

impl<'a, S> HCOptSet<'a, S> {
    pub fn new(set: S, inv: Invoker<'a, Self>) -> Self {
        Self {
            set,
            inv: Some(inv),
            ser: AppServices::default(),
        }
    }

    pub fn invoker(&self) -> Result<&Invoker<'a, Self>, Error> {
        self.inv
            .as_ref()
            .ok_or_else(|| error!("Can not access Invoker in callback"))
    }

    pub fn invoker_mut(&mut self) -> Result<&mut Invoker<'a, Self>, Error> {
        self.inv
            .as_mut()
            .ok_or_else(|| error!("Can not access Invoker in callback"))
    }

    pub fn set_invoker(&mut self, inv: Invoker<'a, Self>) -> &mut Self {
        self.inv = Some(inv);
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
        &self.ser
    }

    pub fn service_mut(&mut self) -> &mut AppServices {
        &mut self.ser
    }

    pub fn set_service(&mut self, ser: AppServices) -> &mut Self {
        self.ser = ser;
        self
    }

    pub fn set_policy<P>(self, policy: P) -> Parser<Self, P>
    where
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
    {
        Parser::new(policy, self)
    }
}

impl<S> Deref for HCOptSet<'_, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<S> DerefMut for HCOptSet<'_, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

impl<S> HCOptSet<'_, S>
where
    S: Set,
{
    /// Reset the option set.
    pub fn reset(&mut self) -> Result<&mut Self, Error> {
        self.set.reset();
        // ignore invoker, it is stateless
        Ok(self)
    }

    /// Call the [`init`](crate::opt::Opt::init) of [`Opt`] initialize the option value.
    pub fn init(&mut self) -> Result<(), Error> {
        let set = &mut self.set;

        for opt in set.iter_mut() {
            opt.init()?;
        }
        Ok(())
    }
}

impl<S> AppStorage for HCOptSet<'_, S> {
    /// Set the value that can access in option handler.
    ///
    /// # Example 1
    /// ```rust
    /// # use aopt::getopt;
    /// # use aopt::prelude::*;
    /// # use aopt::Error;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///
    /// #[derive(Debug)]
    /// struct Int(i64);
    ///
    /// let mut parser = Parser::new_policy(AFwdPolicy::default());
    ///
    /// // Register a value can access in handler parameter.
    /// parser.set_app_data(Int(42));
    /// parser.add_opt("--guess=i!")?.on(
    ///   |set, ctx: &mut Ctx| {
    ///       let val = ctx.value::<i64>()?;
    ///       let answer = set.app_data::<Int>()?;
    ///       match answer.0.cmp(&val) {
    ///         std::cmp::Ordering::Equal => println!("Congratulation, you win!"),
    ///         std::cmp::Ordering::Greater => println!("Oops, too bigger!"),
    ///         std::cmp::Ordering::Less => println!("Oops, too little!"),
    ///       }
    ///       Ok(Some(val))
    ///   },
    /// )?;
    ///
    /// getopt!(Args::from(["--guess", "42"]), &mut parser)?;
    /// #
    /// # Ok(())
    /// # }
    ///```
    ///
    /// # Example 2
    /// ```rust
    /// # use aopt::getopt;
    /// # use aopt::prelude::*;
    /// # use aopt::Error;
    /// #
    /// # fn main() -> Result<(), Error> {
    /// #[derive(Debug)]
    /// struct Int(i64);
    ///
    /// let mut parser = Parser::new_policy(AFwdPolicy::default());
    ///
    /// // Register a value can access in handler parameter.
    /// parser.set_app_data(Int(42));
    /// parser.add_opt("--guess=i!")?.on(|set, ctx: &mut Ctx| {
    ///       let val = ctx.value::<i64>()?;
    ///       let answer = set.app_data::<Int>()?;
    ///
    ///       if answer.0 == val {
    ///           println!("Congratulation, you win!");
    ///       } else if answer.0 > val {
    ///           println!("Oops, too bigger!")
    ///       } else {
    ///           println!("Oops, too little!")
    ///       }
    ///       Ok(Some(val))
    ///   },
    /// )?;
    ///
    /// getopt!(Args::from(["--guess", "42"]), &mut parser)?;
    /// #
    /// # Ok(())
    /// # }
    ///```
    fn set_app_data<T: ErasedTy>(&mut self, val: T) -> Option<T> {
        AppStorage::set_app_data(&mut self.ser, val)
    }

    fn app_data<T: ErasedTy>(&self) -> Result<&T, Error> {
        AppStorage::app_data(&self.ser)
    }

    fn app_data_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        AppStorage::app_data_mut(&mut self.ser)
    }

    fn take_app_data<T: ErasedTy>(&mut self) -> Result<T, Error> {
        AppStorage::take_app_data(&mut self.ser)
    }
}

impl<'a, S> HCOptSet<'a, S>
where
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
    S: Set + OptParser<Output: Information> + OptValidator,
{
    /// Add an option to the [`Set`](Policy::Set), return a [`ParserCommit`].
    ///
    /// Then you can modify the option configurations through the api of [`ParserCommit`].
    /// Also you can call the function [`on`](crate::parser::ParserCommit::on),
    /// register option handler which will called when option set by user.
    /// # Example
    ///
    ///```rust
    /// # use aopt::getopt;
    /// # use aopt::prelude::*;
    /// # use aopt::Error;
    /// # use std::ffi::OsStr;
    /// #
    /// # fn main() -> Result<(), Error> {
    /// let mut parser1 = Parser::new_policy(AFwdPolicy::default());
    ///
    /// // Add an option `--count` with type `i`.
    /// parser1.add_opt("--count=i")?;
    ///
    /// // Add an option `--len` with type `u`, and get its unique id.
    /// let _len_id = parser1.add_opt("--len=u")?.run()?;
    ///
    /// // Add an option `--size` with type `usize`, it has an alias `-s`.
    /// parser1.add_opt("--size;-s".infer::<usize>())?;
    ///
    /// // Add an option `--path` with type `s`.
    /// // Set its value action to `Action::Set`.
    /// // The handler which add by `on` will called when option set.
    /// parser1
    ///     .add_opt("--path=s")?
    ///     .set_action(Action::Set)
    ///     .on(|_, ctx: &mut Ctx| Ok(Some(ctx.value::<String>()?)));
    ///
    /// fn file_count_storer(
    ///     uid: Uid,
    ///     set: &mut AHCSet,
    ///     _: Option<&OsStr>,
    ///     val: Option<bool>,
    /// ) -> Result<bool, Error> {
    ///     let values = set[uid].entry::<u64>().or_insert(vec![0]);
    ///
    ///     if let Some(is_file) = val {
    ///         if is_file {
    ///             values[0] += 1;
    ///
    ///             return Ok(true);
    ///         }
    ///     }
    ///     Ok(false)
    /// }
    /// // Add an NOA `file` with type `p`.
    /// // The handler which add by `on` will called when option set.
    /// // The `store` will called by `Invoker` when storing option value.
    /// parser1
    ///     .add_opt("file=p@1..")?
    ///     .on(|_, ctx: &mut Ctx| {
    ///         let path = ctx.value::<String>()?;
    ///
    ///         if let Ok(meta) = std::fs::metadata(&path) {
    ///             if meta.is_file() {
    ///                 println!("Got a file {:?}", path);
    ///                 return Ok(Some(true));
    ///             }
    ///         }
    ///         Ok(Some(false))
    ///     })?
    ///     .then(file_count_storer);
    ///
    /// getopt!(Args::from(["app", "foo", "-s", "10", "bar"]), &mut parser1)?;
    ///
    /// assert_eq!(parser1.find_val::<u64>("file=p")?, &0);
    /// assert_eq!(parser1.find_val::<usize>("--size")?, &10);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_opt<B>(&mut self, cb: B) -> Result<ParserCommit<'a, '_, Self, S, B::Val>, Error>
    where
        B::Val: Infer + 'static,
        B: ConfigBuild<SetCfg<S>>,
        <B::Val as Infer>::Val: RawValParser,
    {
        let cfg = cb.build(&self.set)?;
        let set = &mut self.set;
        let inv = self.inv.as_mut();

        Ok(ParserCommit::new(SetCommit::new(set, cfg), inv))
    }

    /// Add an option to the [`Set`](Policy::Set), return a [`ParserCommit`].
    ///
    /// ```rust
    /// # use aopt::Error;
    /// # use aopt::prelude::*;
    /// # use std::convert::From;
    /// #
    /// # fn main() -> Result<(), Error> {
    /// pub struct Bool;
    ///
    /// impl From<Bool> for OptConfig {
    ///     fn from(_: Bool) -> Self {
    ///         OptConfig::default()
    ///             .with_ctor("b")
    ///             .with_type::<bool>()
    ///             .with_style(vec![Style::Boolean, Style::Combined])
    ///             .with_action(Action::Set)
    ///             .with_storer(ValStorer::fallback::<bool>())
    ///             .with_ignore_index(true)
    ///             .with_initializer(ValInitializer::new_value(false))
    ///     }
    /// }
    ///
    /// pub struct Int64;
    ///
    /// impl From<Int64> for OptConfig {
    ///     fn from(_: Int64) -> Self {
    ///         OptConfig::default()
    ///             .with_ctor(aopt::prelude::ctor_default_name())
    ///             .with_style(vec![Style::Argument])
    ///             .with_type::<i64>()
    ///             .with_action(Action::Set)
    ///             .with_storer(ValStorer::fallback::<i64>())
    ///             .with_ignore_index(true)
    ///             .with_initializer(ValInitializer::new_value(0i64))
    ///     }
    /// }
    ///
    ///     let mut parser = AFwdParser::default();
    ///
    ///     parser.add_opt_cfg(Bool)?.set_name("--round");
    ///     parser.add_opt_cfg(Int64)?.set_name("--poll");
    ///
    ///     parser.parse(Args::from(["--poll", "42"]))?;
    ///
    ///     assert_eq!(parser.find_val::<bool>("--round")?, &false);
    ///     assert_eq!(parser.find_val::<i64>("--poll")?, &42);
    ///
    /// #    Ok(())
    /// # }
    ///```
    pub fn add_opt_cfg(
        &mut self,
        config: impl Into<SetCfg<S>>,
    ) -> Result<ParserCommit<'a, '_, Self, S, Placeholder>, Error> {
        Ok(ParserCommit::new(
            SetCommit::new_placeholder(&mut self.set, config.into()),
            self.inv.as_mut(),
        ))
    }

    pub fn add_opt_cfg_i<U>(
        &mut self,
        config: impl Into<SetCfg<S>>,
    ) -> Result<ParserCommit<'a, '_, Self, S, U>, Error>
    where
        U: Infer + 'static,
        U::Val: RawValParser,
    {
        Ok(ParserCommit::new(
            SetCommit::new(&mut self.set, config.into()),
            self.inv.as_mut(),
        ))
    }
}

impl<'a, S: Set> HCOptSet<'a, S> {
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
        Ok(HandlerEntry::new(self.invoker_mut()?, uid))
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
        Ok(HandlerEntry::new(self.invoker_mut()?, uid))
    }
}

impl<S: Set> crate::set::Set for HCOptSet<'_, S> {
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

impl<S> OptValidator for HCOptSet<'_, S>
where
    S: OptValidator,
{
    type Error = S::Error;

    fn check(&mut self, name: &str) -> Result<bool, Self::Error> {
        OptValidator::check(&mut self.set, name)
    }

    fn split<'a>(&self, name: &Cow<'a, str>) -> Result<(Cow<'a, str>, Cow<'a, str>), Self::Error> {
        OptValidator::split(&self.set, name)
    }
}

impl<S> PrefixedValidator for HCOptSet<'_, S>
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

impl<'a, S, P> PolicyParser<P> for HCOptSet<'a, S>
where
    S: crate::set::Set,
    P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self>>,
{
    type Error = Error;

    fn parse_policy(
        &mut self,
        args: Args,
        policy: &mut P,
    ) -> Result<<P as Policy>::Ret, Self::Error> {
        self.init()?;

        let mut inv = self.inv.take().unwrap();

        let ret = policy.parse(self, &mut inv, args).map_err(Into::into);

        self.inv = Some(inv);
        ret
    }
}

impl<S> OptParser for HCOptSet<'_, S>
where
    S: OptParser,
{
    type Output = S::Output;

    type Error = S::Error;

    fn parse_opt(&self, pattern: &str) -> Result<Self::Output, Self::Error> {
        OptParser::parse_opt(&self.set, pattern)
    }
}

impl<S> SetValueFindExt for HCOptSet<'_, S>
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

#[cfg(test)]
mod test {
    use crate::{opt::config::ConfigBuildInfer, prelude::*};

    #[test]
    fn test() {
        assert!(test_hc_optset().is_ok());
    }

    fn test_hc_optset() -> Result<(), crate::Error> {
        let mut set = HCOptSet::<ASet>::default();

        set.add_opt("--aopt=b")?;
        set.add_opt("--bopt=i")?;
        set.entry(0)?.on(|_, ctx: &mut Ctx| {
            let val = ctx.value::<bool>()?;

            assert!(val);
            Ok(Some(val))
        });
        set.add_opt("ls".infer::<Cmd>())?;

        PolicyParser::<AFwdPolicy>::parse(
            &mut set,
            Args::from(["app", "ls", "--aopt", "--bopt=42"]),
        )?;

        assert_eq!(set.find_val::<bool>("ls")?, &true);
        assert_eq!(set.find_val::<bool>("--aopt")?, &true);
        assert_eq!(set.find_val::<i64>("--bopt")?, &42);

        Ok(())
    }
}
