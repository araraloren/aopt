pub(crate) mod checker;
#[cfg_attr(feature = "sync", path = "sync/parser/commit.rs")]
#[cfg_attr(not(feature = "sync"), path = "parser/commit.rs")]
pub(crate) mod commit;
pub(crate) mod policy_delay;
pub(crate) mod policy_fwd;
pub(crate) mod policy_pre;
pub(crate) mod process;
pub(crate) mod returnval;
pub(crate) mod style;
pub(crate) mod ucommit;

pub use self::checker::SetChecker;
pub use self::commit::ParserCommit;
pub use self::policy_delay::DelayPolicy;
pub use self::policy_fwd::FwdPolicy;
pub use self::policy_pre::PrePolicy;
pub use self::returnval::ReturnVal;
pub use self::style::Guess;
pub use self::style::GuessNOACfg;
pub use self::style::GuessOptCfg;
pub use self::style::NOAGuess;
pub use self::style::OptGuess;
pub use self::style::OptStyleManager;
pub use self::style::UserStyle;
pub use self::style::UserStyleMange;
pub use self::ucommit::UParserCommit;

pub(crate) use self::process::invoke_callback_opt;
pub(crate) use self::process::process_non_opt;
pub(crate) use self::process::process_opt;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::args::Args;
use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::HandlerEntry;
use crate::ctx::InnerCtx;
use crate::ctx::Invoker;
use crate::ext::APolicyExt;
use crate::map::ErasedTy;
use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::opt::OptValueExt;
use crate::ser::ServicesExt;
use crate::ser::ServicesValExt;
use crate::set::Commit;
use crate::set::Ctor;
use crate::set::Filter;
use crate::set::OptValidator;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetExt;
use crate::set::SetOpt;
use crate::set::UCommit;
use crate::value::Infer;
use crate::value::RawValParser;
use crate::Arc;
use crate::Error;
use crate::Str;
use crate::Uid;

#[derive(Debug, Clone)]
pub struct CtxSaver {
    /// option uid
    pub uid: Uid,

    /// Index of matcher
    pub idx: usize,

    /// invoke context
    pub ctx: InnerCtx,
}

/// [`Policy`] doing real parsing work.
///
/// # Example
/// ```ignore
///
/// #[derive(Debug)]
/// pub struct EmptyPolicy<Set, Ser>(PhantomData<(Set, Ser)>);
///
/// // An empty policy do nothing.
/// impl<S: Set, T: Ser> Policy for EmptyPolicy<S, T> {
///     type Ret = bool;
///
///     type Set = S;
///
///     type Inv = Invoker<S>;
///
///     type Ser = T;
///
///     type Error = Error;
///
///     fn parse(&mut self, _: &mut S, _: &mut T, _: Arc<Args>) -> Result<bool, Error> {
///         // ... parsing logical code
///         Ok(Some(true))
///     }
/// }
/// ```
pub trait Policy {
    type Ret;
    type Set;
    type Inv;
    type Ser;
    type Error: Into<Error>;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv,
        ser: &mut Self::Ser,
        args: Arc<Args>,
    ) -> Result<Self::Ret, Self::Error>;
}

impl<S, I, O, R, E> Policy for Box<dyn Policy<Ret = R, Set = S, Inv = I, Ser = O, Error = E>>
where
    E: Into<Error>,
{
    type Ret = R;

    type Set = S;

    type Inv = I;

    type Ser = O;

    type Error = E;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv,
        ser: &mut Self::Ser,
        args: Arc<Args>,
    ) -> Result<Self::Ret, Self::Error> {
        Policy::parse(self.as_mut(), set, inv, ser, args)
    }
}

/// Parser manage the [`Set`], [`Services`] and [`Policy`].
///
/// # Example
///
/// ```rust
/// # use aopt::getopt;
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut parser1 = Parser::new(AFwdPolicy::default());
///
/// parser1.add_opt("Where=c")?;
/// parser1.add_opt("question=m")?.on(question)?;
///
/// let mut parser2 = Parser::new(AFwdPolicy::default());
///
/// parser2.add_opt("Who=c")?;
/// parser2.add_opt("question=m")?.on(question)?;
///
/// fn question(_: &mut ASet, _: &mut ASer, args: ctx::Args) -> Result<Option<()>, Error> {
///     // Output: The question is: Where are you from ?
///     println!(
///         "The question is: {}",
///         args.iter()
///             .map(|v| v.get_str().unwrap().to_owned())
///             .collect::<Vec<String>>()
///             .join(" ")
///     );
///     Ok(Some(()))
/// }
///
/// let ret = getopt!(
///     Args::from_array(["app", "Where", "are", "you", "from", "?"]),
///     &mut parser1,
///     &mut parser2
/// )?;
///
/// assert!(ret.is_some());
/// let ret = ret.unwrap();
/// let parser = ret.parser;
/// assert_eq!(
///     parser[0].name(),
///     "Where",
///     "Parser with `Where` cmd matched"
/// );
/// #
/// # Ok(())
/// # }
/// ```
///
/// Using it with macro [`getopt`](crate::getopt),
/// which can process multiple [`Parser`] with same type [`Policy`].
#[derive(Debug)]
pub struct Parser<P: Policy> {
    policy: P,
    optset: P::Set,
    invoker: P::Inv,
    appser: P::Ser,
}

impl<P: Policy> Default for Parser<P>
where
    P::Set: Default,
    P::Inv: Default,
    P::Ser: Default,
    P: Default + Policy + APolicyExt<P>,
{
    fn default() -> Self {
        let policy = P::default();
        Self {
            optset: policy.default_set(),
            invoker: policy.default_inv(),
            appser: policy.default_ser(),
            policy,
        }
    }
}

impl<P: Policy> Deref for Parser<P> {
    type Target = P::Set;

    fn deref(&self) -> &Self::Target {
        &self.optset
    }
}

impl<P: Policy> DerefMut for Parser<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.optset
    }
}

impl<P> Parser<P>
where
    P: Policy + APolicyExt<P>,
{
    pub fn new(policy: P) -> Self {
        let optset = policy.default_set();
        let valser = policy.default_ser();
        let invoker = policy.default_inv();

        Self {
            optset,
            policy,
            invoker,
            appser: valser,
        }
    }
}

pub type BoxedPolicy<P> = Box<
    dyn Policy<
        Ret = <P as Policy>::Ret,
        Set = <P as Policy>::Set,
        Inv = <P as Policy>::Inv,
        Ser = <P as Policy>::Ser,
        Error = <P as Policy>::Error,
    >,
>;

impl<P> Parser<P>
where
    P: Policy + 'static,
{
    pub fn into_boxed(self) -> Parser<BoxedPolicy<P>> {
        let policy: BoxedPolicy<P> = Box::new(self.policy);

        Parser {
            policy,
            optset: self.optset,
            invoker: self.invoker,
            appser: self.appser,
        }
    }
}

impl<P: Policy> Parser<P> {
    pub fn new_with(policy: P, optset: P::Set, invoker: P::Inv, valser: P::Ser) -> Self {
        Self {
            optset,
            policy,
            invoker,
            appser: valser,
        }
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

    pub fn invoker(&self) -> &P::Inv {
        &self.invoker
    }

    pub fn invoker_mut(&mut self) -> &mut P::Inv {
        &mut self.invoker
    }

    pub fn set_invoker(&mut self, invser: P::Inv) -> &mut Self {
        self.invoker = invser;
        self
    }

    pub fn service(&self) -> &P::Ser {
        &self.appser
    }

    pub fn service_mut(&mut self) -> &mut P::Ser {
        &mut self.appser
    }

    pub fn set_service(&mut self, valser: P::Ser) -> &mut Self {
        self.appser = valser;
        self
    }

    pub fn optset(&self) -> &P::Set {
        &self.optset
    }

    pub fn optset_mut(&mut self) -> &mut P::Set {
        &mut self.optset
    }

    pub fn set_optset(&mut self, optset: P::Set) -> &mut Self {
        self.optset = optset;
        self
    }
}

impl<P> Parser<P>
where
    P::Set: Set,
    P::Ser: ServicesValExt,
    P: Policy<Error = Error>,
{
    /// Reset the option set.
    pub fn reset(&mut self) -> Result<&mut Self, Error> {
        self.optset.reset();
        // ignore invoker, it is stateless
        Ok(self)
    }

    pub fn app_data<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.appser.sve_val()
    }

    pub fn app_data_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.appser.sve_val_mut()
    }

    /// Set the user value that can access in option handler.
    ///
    /// # Example 1
    /// ```rust
    /// # use aopt::getopt;
    /// # use aopt::prelude::*;
    /// # use aopt::Arc;
    /// # use aopt::Error;
    /// # use std::ops::Deref;
    /// #
    /// # fn main() -> Result<(), Error> {
    /// struct Int(i64);
    ///
    /// let mut parser = Parser::new(AFwdPolicy::default());
    ///
    /// // Register a value can access in handler parameter.
    /// parser.set_usrval(ser::Value::new(Int(42)))?;
    /// parser.add_opt("--guess=i!")?.on(
    ///   |_: &mut ASet, _: &mut ASer, mut val: ctx::Value<i64>, answer: ser::Value<Int>| {
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
    /// getopt!(Args::from_array(["--guess", "42"]), &mut parser)?;
    /// #
    /// # Ok(())
    /// # }
    ///```
    ///
    /// # Example 2
    /// ```rust
    /// # use aopt::getopt;
    /// # use aopt::prelude::*;
    /// # use aopt::Arc;
    /// # use aopt::Error;
    /// # use std::ops::Deref;
    /// #
    /// # fn main() -> Result<(), Error> {
    /// struct Int(i64);
    ///
    /// let mut parser = Parser::new(AFwdPolicy::default());
    ///
    /// // Register a value can access in handler parameter.
    /// parser.set_usrval(Int(42))?;
    /// parser.add_opt("--guess=i!")?.on(
    ///   |_: &mut ASet, ser: &mut ASer, mut val: ctx::Value<i64>| {
    ///       let answer = ser.sve_usrval::<Int>()?;
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
    /// getopt!(Args::from_array(["--guess", "42"]), &mut parser)?;
    /// #
    /// # Ok(())
    /// # }
    ///```
    pub fn set_app_data<T: ErasedTy>(&mut self, val: T) -> Result<Option<T>, Error> {
        Ok(self.appser.sve_insert(val))
    }
}

impl<P> Parser<P>
where
    P::Set: Set,
    P: Policy<Error = Error>,
{
    /// Call the [`init`](crate::opt::Opt::init) on [`Services`] initialize the option value.
    pub fn init(&mut self) -> Result<(), P::Error> {
        let optset = &mut self.optset;

        for opt in optset.iter_mut() {
            opt.init()?;
        }
        Ok(())
    }
}

impl<P> Parser<P>
where
    P::Set: Set,
    P: Policy<Error = Error>,
{
    /// Call [`parse`](Policy::parse) parsing the given arguments.
    ///
    /// The [`status`](ReturnVal::status) is true if parsing successes
    /// otherwise it will be false if any [`failure`](Error::is_failure) raised.
    pub fn parse(&mut self, args: Arc<Args>) -> Result<P::Ret, P::Error> {
        let optset = &mut self.optset;
        let valser = &mut self.appser;
        let invser = &mut self.invoker;

        self.policy.parse(optset, invser, valser, args)
    }

    /// Call [`parse`](Parser::parse) parsing the [`Args`](Args::from_env).
    ///
    /// The [`status`](ReturnVal::status) is true if parsing successes
    /// otherwise it will be false if any [`failure`](Error::is_failure) raised.
    pub fn parse_env(&mut self) -> Result<P::Ret, P::Error> {
        let optset = &mut self.optset;
        let valser = &mut self.appser;
        let invser = &mut self.invoker;
        let args = crate::Arc::new(Args::from_env());

        self.policy.parse(optset, invser, valser, args)
    }
}

impl<P> Parser<P>
where
    P::Ser: ServicesExt,
    SetOpt<P::Set>: Opt,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    P::Set: Set + OptParser + OptValidator + 'static,
    P: Policy<Inv = Invoker<<P as Policy>::Set, <P as Policy>::Ser>, Error = Error>,
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
    /// # use aopt::Arc;
    /// # use aopt::Error;
    /// # use aopt::RawVal;
    /// # use std::ops::Deref;
    /// #
    /// # fn main() -> Result<(), Error> {
    /// let mut parser1 = Parser::new(AFwdPolicy::default());
    ///
    /// // Add an option `--count` with type `i`.
    /// parser1.add_opt("--count=i")?;
    /// // Add an option `--len` with type `u`, and get its unique id.
    /// let _len_id = parser1.add_opt("--len=u")?.run()?;
    ///
    /// // Add an option `--size` with type `u`, it has an alias `-s`.
    /// parser1.add_opt("--size=u")?.add_alias("-s");
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
    ///     _: &mut ASet,
    ///     ser: &mut ASer,
    ///     _: Option<&RawVal>,
    ///     val: Option<bool>,
    /// ) -> Result<Option<()>, Error> {
    ///     let values = ser.ser_val_mut().entry::<u64>(uid).or_insert(vec![0]);
    ///
    ///     if let Some(is_file) = val {
    ///         if is_file {
    ///             values[0] += 1;
    ///             return Ok(Some(()));
    ///         }
    ///     }
    ///     Ok(None)
    /// }
    /// // Add an NOA `file` with type `p`.
    /// // The handler which add by `on` will called when option set.
    /// // The store will called by `Invoker` when storing option value.
    /// parser1
    ///     .add_opt("file=p@2..")?
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
    /// getopt!(Args::from_array(["app", "foo", "bar"]), &mut parser1)?;
    ///
    /// dbg!(parser1.find_val::<u64>("file=p")?);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_opt(
        &mut self,
        opt: impl Into<Str>,
    ) -> Result<ParserCommit<'_, P::Set, P::Ser>, Error> {
        let info =
            <<<P::Set as Set>::Ctor as Ctor>::Config as Config>::new(&self.optset, opt.into())?;

        Ok(ParserCommit::new(
            Commit::new(&mut self.optset, info),
            &mut self.invoker,
        ))
    }

    pub fn add_opt_i<U>(
        &mut self,
        opt: impl Into<Str>,
    ) -> Result<UParserCommit<'_, P::Set, P::Ser, U>, Error>
    where
        U: Infer,
        U::Val: RawValParser,
    {
        let info =
            <<<P::Set as Set>::Ctor as Ctor>::Config as Config>::new(&self.optset, opt.into())?;

        Ok(UParserCommit::new(
            UCommit::new(&mut self.optset, info),
            &mut self.invoker,
        ))
    }

    /// Add an option to the [`Set`](Policy::Set), return a [`ParserCommit`].
    ///
    /// ```rust
    /// # use aopt::Error;
    /// # use aopt::prelude::*;
    /// # use std::convert::From;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     pub struct Bool;
    ///
    ///     impl From<Bool> for OptConfig {
    ///         fn from(_: Bool) -> Self {
    ///             OptConfig::default()
    ///                 .with_type("a")
    ///                 .with_action(Some(Action::Set))
    ///                 .with_assoc(Some(Assoc::Bool))
    ///                 .with_initiator(Some(ValInitiator::bool(false)))
    ///         }
    ///     }
    ///
    ///     pub struct Int64;
    ///
    ///     impl From<Int64> for OptConfig {
    ///         fn from(_: Int64) -> Self {
    ///             OptConfig::default()
    ///                 .with_type("i")
    ///                 .with_action(Some(Action::Set))
    ///                 .with_assoc(Some(Assoc::Int))
    ///                 .with_initiator(Some(ValInitiator::i64(0)))
    ///         }
    ///     }
    ///
    ///     let mut parser = AFwdParser::default();
    ///
    ///     parser.add_opt_cfg(Bool)?.set_name("--round");
    ///     parser.add_opt_cfg(Int64)?.set_name("--poll");
    ///
    ///     parser.init()?;
    ///     parser.parse(aopt::Arc::new(Args::from(["--poll", "42"].into_iter())))?;
    ///
    ///     assert_eq!(parser.find_val::<bool>("--round")?, &false);
    ///     assert_eq!(parser.find_val::<i64>("--poll")?, &42);
    ///
    /// #    Ok(())
    /// # }
    ///```
    pub fn add_opt_cfg<Cfg: Into<SetCfg<P::Set>>>(
        &mut self,
        config: Cfg,
    ) -> Result<ParserCommit<'_, P::Set, P::Ser>, Error> {
        Ok(ParserCommit::new(
            Commit::new(&mut self.optset, config.into()),
            &mut self.invoker,
        ))
    }

    pub fn add_opt_cfg_i<U: Infer, Cfg: Into<SetCfg<P::Set>>>(
        &mut self,
        config: Cfg,
    ) -> Result<UParserCommit<'_, P::Set, P::Ser, U>, Error>
    where
        U: Infer,
        U::Val: RawValParser,
    {
        Ok(UParserCommit::new(
            UCommit::new(&mut self.optset, config.into()),
            &mut self.invoker,
        ))
    }

    #[cfg(feature = "sync")]
    #[allow(clippy::type_complexity)]
    pub fn entry<A, O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'_, P::Set, P::Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<P::Set, P::Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
        A: Extract<P::Set, P::Ser, Error = Error> + Send + Sync + 'static,
    {
        Ok(HandlerEntry::new(&mut self.invoker, uid))
    }

    #[cfg(not(feature = "sync"))]
    #[allow(clippy::type_complexity)]
    pub fn entry<A, O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'_, P::Set, P::Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<P::Set, P::Ser, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<P::Set, P::Ser, Error = Error> + 'static,
    {
        Ok(HandlerEntry::new(&mut self.invoker, uid))
    }
}

impl<P> Parser<P>
where
    P::Ser: ServicesExt,
    P: Policy<Error = Error>,
    P::Set: Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    pub fn find_uid(&self, opt: &str) -> Result<Uid, Error> {
        let filter = Filter::new(
            &self.optset,
            SetCfg::<P::Set>::new(&self.optset, opt.into())?,
        );
        filter.find().map(|v| v.uid()).ok_or_else(|| {
            Error::raise_error(format!(
                "Can not find option: invalid option string {}",
                opt
            ))
        })
    }

    pub fn find_val<U: ErasedTy>(&self, opt: &str) -> Result<&U, Error> {
        self.opt(self.find_uid(opt)?)?.val()
    }

    pub fn find_val_mut<U: ErasedTy>(&mut self, opt: &str) -> Result<&mut U, Error> {
        let uid = self.find_uid(opt)?;

        self.opt_mut(uid)?.val_mut()
    }

    pub fn find_vals<U: ErasedTy>(&self, opt: &str) -> Result<&Vec<U>, Error> {
        self.opt(self.find_uid(opt)?)?.vals()
    }

    pub fn find_vals_mut<U: ErasedTy>(&mut self, opt: &str) -> Result<&mut Vec<U>, Error> {
        let uid = self.find_uid(opt)?;

        self.opt_mut(uid)?.vals_mut()
    }

    pub fn take_val<U: ErasedTy>(&mut self, opt: &str) -> Result<U, Error> {
        let opt = self.find_uid(opt)?;
        let vals = self.opt_mut(opt)?.vals_mut::<U>()?;

        vals.pop().ok_or_else(|| {
            Error::raise_error(format!("Not enough value can take from option {}", opt))
        })
    }
}

impl<P> UserStyleMange for Parser<P>
where
    P: Policy + UserStyleMange,
{
    fn style_manager(&self) -> &OptStyleManager {
        self.policy().style_manager()
    }

    fn style_manager_mut(&mut self) -> &mut OptStyleManager {
        self.policy_mut().style_manager_mut()
    }
}

impl<P> Parser<P>
where
    P: Policy + UserStyleMange,
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
