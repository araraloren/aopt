use std::ops::Deref;
use std::ops::DerefMut;

use crate::args::Args;
use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::HandlerCollection;
use crate::ctx::HandlerEntry;
use crate::map::ErasedTy;
use crate::opt::ConfigBuild;
use crate::opt::ConfigValue;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::parser::ParserCommit;
use crate::parser::Policy;
use crate::ser::ServicesValExt;
use crate::set::OptValidator;
use crate::set::SetCfg;
use crate::set::SetCommit;
use crate::set::SetOpt;
use crate::set::SetValueFindExt;
use crate::value::Infer;
use crate::value::Placeholder;
use crate::value::RawValParser;
use crate::ARef;
use crate::Error;
use crate::Uid;

use super::Parser;
use super::PolicyParser;

#[derive(Debug, Default, Clone)]
pub struct HCOptSet<Set, Inv, Ser> {
    set: Set,
    inv: Inv,
    ser: Ser,
}

impl<Set, Inv, Ser> HCOptSet<Set, Inv, Ser> {
    pub fn new(set: Set, inv: Inv, ser: Ser) -> Self {
        Self { set, inv, ser }
    }

    pub fn invoker(&self) -> &Inv {
        &self.inv
    }

    pub fn invoker_mut(&mut self) -> &mut Inv {
        &mut self.inv
    }

    pub fn set_invoker(&mut self, inv: Inv) -> &mut Self {
        self.inv = inv;
        self
    }

    pub fn service(&self) -> &Ser {
        &self.ser
    }

    pub fn service_mut(&mut self) -> &mut Ser {
        &mut self.ser
    }

    pub fn set_service(&mut self, ser: Ser) -> &mut Self {
        self.ser = ser;
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

    pub fn set_policy<'a, P>(self, policy: P) -> Parser<'a, P>
    where
        P: Policy<Set = Set, Inv<'a> = Inv, Ser = Ser>,
    {
        Parser::new_with(policy, self.set, self.inv, self.ser)
    }
}

impl<Set, Inv, Ser> Deref for HCOptSet<Set, Inv, Ser> {
    type Target = Set;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<Set, Inv, Ser> DerefMut for HCOptSet<Set, Inv, Ser> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

impl<Set, Inv, Ser> HCOptSet<Set, Inv, Ser>
where
    Set: crate::set::Set,
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

impl<Set, Inv, Ser> HCOptSet<Set, Inv, Ser>
where
    Ser: ServicesValExt,
{
    pub fn app_data<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.ser.sve_val()
    }

    pub fn app_data_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.ser.sve_val_mut()
    }

    /// Set the value that can access in option handler.
    ///
    /// # Example 1
    /// ```rust
    /// # use aopt::getopt;
    /// # use aopt::prelude::*;
    /// # use aopt::ARef;
    /// # use aopt::Error;
    /// # use std::ops::Deref;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///
    /// #[derive(Debug)]
    /// struct Int(i64);
    ///
    /// let mut parser = Parser::new_policy(AFwdPolicy::default());
    ///
    /// // Register a value can access in handler parameter.
    /// parser.set_app_data(ser::Value::new(Int(42)))?;
    /// parser.add_opt("--guess=i!")?.on(
    ///   |_: &mut ASet, _: &mut ASer, ctx::Value(val), answer: ser::Value<Int>| {
    ///       match answer.deref().0.cmp(&val) {
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
    /// # use aopt::ARef;
    /// # use aopt::Error;
    /// # use std::ops::Deref;
    /// #
    /// # fn main() -> Result<(), Error> {
    /// #[derive(Debug)]
    /// struct Int(i64);
    ///
    /// let mut parser = Parser::new_policy(AFwdPolicy::default());
    ///
    /// // Register a value can access in handler parameter.
    /// parser.set_app_data(Int(42))?;
    /// parser.add_opt("--guess=i!")?.on(
    ///   |_: &mut ASet, ser: &mut ASer, mut val: ctx::Value<i64>| {
    ///       let answer = ser.sve_val::<Int>()?;
    ///
    ///       if &answer.0 == val.deref() {
    ///           println!("Congratulation, you win!");
    ///       } else if &answer.0 > val.deref() {
    ///           println!("Oops, too bigger!")
    ///       } else {
    ///           println!("Oops, too little!")
    ///       }
    ///       Ok(Some(val.take()))
    ///   },
    /// )?;
    ///
    /// getopt!(Args::from(["--guess", "42"]), &mut parser)?;
    /// #
    /// # Ok(())
    /// # }
    ///```
    pub fn set_app_data<T: ErasedTy>(&mut self, val: T) -> Result<Option<T>, Error> {
        Ok(self.ser.sve_insert(val))
    }
}

impl<'a, Set, Inv, Ser> HCOptSet<Set, Inv, Ser>
where
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
    <Set as OptParser>::Output: Information,
    Inv: HandlerCollection<'a, Set, Ser>,
    Set: crate::set::Set + OptParser + OptValidator,
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
    /// # use aopt::ARef;
    /// # use aopt::Error;
    /// # use aopt::RawVal;
    /// # use std::ops::Deref;
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
    ///     .on(|_: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>| Ok(Some(val.take())))?;
    ///
    /// fn file_count_storer(
    ///     uid: Uid,
    ///     set: &mut ASet,
    ///     _: &mut ASer,
    ///     _: Option<&RawVal>,
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
    ///     .on(|_: &mut ASet, _: &mut ASer, val: ctx::Value<String>| {
    ///         let path = val.deref();
    ///
    ///         if let Ok(meta) = std::fs::metadata(path) {
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
    pub fn add_opt<B>(
        &mut self,
        cb: B,
    ) -> Result<ParserCommit<'a, '_, Inv, Set, Ser, B::Val>, Error>
    where
        B::Val: Infer + 'static,
        B: ConfigBuild<SetCfg<Set>>,
        <B::Val as Infer>::Val: RawValParser,
    {
        let cfg = cb.build(&self.set)?;
        let set = &mut self.set;
        let inv = &mut self.inv;

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
    ///     parser.parse(aopt::ARef::new(Args::from(["--poll", "42"].into_iter())))?;
    ///
    ///     assert_eq!(parser.find_val::<bool>("--round")?, &false);
    ///     assert_eq!(parser.find_val::<i64>("--poll")?, &42);
    ///
    /// #    Ok(())
    /// # }
    ///```
    pub fn add_opt_cfg(
        &mut self,
        config: impl Into<SetCfg<Set>>,
    ) -> Result<ParserCommit<'a, '_, Inv, Set, Ser, Placeholder>, Error> {
        Ok(ParserCommit::new(
            SetCommit::new_placeholder(&mut self.set, config.into()),
            &mut self.inv,
        ))
    }

    pub fn add_opt_cfg_i<U>(
        &mut self,
        config: impl Into<SetCfg<Set>>,
    ) -> Result<ParserCommit<'a, '_, Inv, Set, Ser, U>, Error>
    where
        U: Infer + 'static,
        U::Val: RawValParser,
    {
        Ok(ParserCommit::new(
            SetCommit::new(&mut self.set, config.into()),
            &mut self.inv,
        ))
    }
}

impl<'a, Set, Inv, Ser> HCOptSet<Set, Inv, Ser>
where
    Set: crate::set::Set,
    Inv: HandlerCollection<'a, Set, Ser>,
{
    #[cfg(feature = "sync")]
    #[allow(clippy::type_complexity)]
    pub fn entry<A, O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, Inv, Set, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'a,
        A: Extract<Set, Ser, Error = Error> + Send + Sync + 'a,
    {
        Ok(HandlerEntry::new(&mut self.inv, uid))
    }

    #[cfg(not(feature = "sync"))]
    #[allow(clippy::type_complexity)]
    pub fn entry<A, O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, Inv, Set, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + 'a,
        A: Extract<Set, Ser, Error = Error> + 'a,
    {
        Ok(HandlerEntry::new(&mut self.inv, uid))
    }
}

impl<Set, Inv, Ser> crate::set::Set for HCOptSet<Set, Inv, Ser>
where
    Set: crate::set::Set,
{
    type Ctor = Set::Ctor;

    fn register(&mut self, ctor: Self::Ctor) -> Option<Self::Ctor> {
        Set::register(&mut self.set, ctor)
    }

    fn get_ctor(&self, name: &crate::AStr) -> Option<&Self::Ctor> {
        Set::get_ctor(&self.set, name)
    }

    fn get_ctor_mut(&mut self, name: &crate::AStr) -> Option<&mut Self::Ctor> {
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

impl<Set, Inv, Ser> OptValidator for HCOptSet<Set, Inv, Ser>
where
    Set: OptValidator,
{
    type Error = Set::Error;

    fn check(&mut self, name: &str) -> Result<bool, Self::Error> {
        OptValidator::check(&mut self.set, name)
    }

    fn split<'a>(&self, name: &'a str) -> Result<(&'a str, &'a str), Self::Error> {
        OptValidator::split(&self.set, name)
    }
}

impl<'a, P: Policy> PolicyParser<P> for HCOptSet<P::Set, P::Inv<'a>, P::Ser>
where
    P::Set: crate::set::Set,
{
    type Error = Error;

    fn parse_policy(
        &mut self,
        args: ARef<Args>,
        policy: &mut P,
    ) -> Result<<P as Policy>::Ret, Self::Error> {
        self.init()?;

        let set = &mut self.set;
        let ser = &mut self.ser;
        let inv = &mut self.inv;

        policy.parse(set, inv, ser, args).map_err(Into::into)
    }
}

impl<Set, Inv, Ser> OptParser for HCOptSet<Set, Inv, Ser>
where
    Set: OptParser,
{
    type Output = Set::Output;

    type Error = Set::Error;

    fn parse_opt(&self, pattern: &str) -> Result<Self::Output, Self::Error> {
        OptParser::parse_opt(&self.set, pattern)
    }
}

impl<Set, Inv, Ser> SetValueFindExt for HCOptSet<Set, Inv, Ser>
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

#[cfg(test)]
mod test {
    use crate::{opt::config::ConfigBuildInfer, prelude::*};
    use std::ops::Deref;

    #[test]
    fn test() {
        assert!(test_hc_optset().is_ok());
    }

    fn test_hc_optset() -> Result<(), crate::Error> {
        let mut set = HCOptSet::<ASet, AInvoker, ASer>::default();

        set.add_opt("--aopt=b")?;
        set.add_opt("--bopt=i")?;
        set.entry(0)?
            .on(|_: &mut ASet, _: &mut ASer, mut val: ctx::Value<bool>| {
                assert_eq!(val.deref(), &true);
                Ok(Some(val.take()))
            });
        set.add_opt("ls".infer::<Cmd>())?;

        PolicyParser::<AFwdPolicy>::parse(
            &mut set,
            ARef::new(Args::from(["app", "ls", "--aopt", "--bopt=42"])),
        )?;

        assert_eq!(set.find_val::<bool>("ls")?, &true);
        assert_eq!(set.find_val::<bool>("--aopt")?, &true);
        assert_eq!(set.find_val::<i64>("--bopt")?, &42);

        Ok(())
    }
}
