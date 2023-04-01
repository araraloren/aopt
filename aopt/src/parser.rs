pub(crate) mod checker;
pub(crate) mod commit;
pub(crate) mod policy_delay;
pub(crate) mod policy_fwd;
pub(crate) mod policy_pre;
pub(crate) mod process;
pub(crate) mod returnval;
pub(crate) mod style;

pub use self::checker::DefaultSetChecker;
pub use self::commit::ParserCommit;
pub use self::commit::ParserCommitWithValue;
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

pub(crate) use self::process::invoke_callback_opt;
pub(crate) use self::process::process_non_opt;
pub(crate) use self::process::process_opt;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::args::Args;
use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::HandlerCollection;
use crate::ctx::HandlerEntry;
use crate::ctx::InnerCtx;
use crate::ext::APolicyExt;
use crate::map::ErasedTy;
use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::prelude::SetCommit;
use crate::ser::ServicesValExt;
use crate::set::OptValidator;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetOpt;
use crate::value::Infer;
use crate::value::Placeholder;
use crate::value::RawValParser;
use crate::ARef;
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
/// impl<S: Set, Ser> Policy for EmptyPolicy<S, Ser> {
///     type Ret = bool;
///
///     type Set = S;
///
///     type Inv<'a> = Invoker<'a, S, Ser>;
///
///     type Ser = Ser;
///
///     type Error = Error;
///
///     fn parse<'a>(
///         &mut self,
///         _: &mut Self::Set,
///         _: &mut Self::Inv<'a>,
///         _: &mut Self::Ser,
///         _: ARef<Args>,
///    ) -> Result<bool, Error> {
///         // ... parsing logical code
///        Ok(true)
///     }
/// }
/// ```
pub trait Policy {
    type Ret;
    type Set;
    type Inv<'a>;
    type Ser;
    type Error: Into<Error>;

    fn parse<'a>(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv<'a>,
        ser: &mut Self::Ser,
        args: ARef<Args>,
    ) -> Result<Self::Ret, Self::Error>;
}

pub trait PolicySettings {
    fn style_manager(&self) -> &OptStyleManager;

    fn style_manager_mut(&mut self) -> &mut OptStyleManager;

    fn strict(&self) -> bool;

    fn styles(&self) -> &[UserStyle];

    fn no_delay(&self) -> Option<&[Str]>;

    fn set_strict(&mut self, strict: bool) -> &mut Self;

    fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self;

    fn set_no_delay(&mut self, name: impl Into<Str>) -> &mut Self;
}

/// Parser manage the components are using in [`parse`](Policy::parse) of [`Policy`].
///
/// # Example
///
/// ```rust
/// # use aopt::getopt;
/// # use aopt::prelude::*;
/// # use aopt::ARef;
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
///         args.iter().skip(1)
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
/// let parser = ret.parser;
///
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
pub struct Parser<'a, P: Policy> {
    policy: P,
    optset: P::Set,
    invoker: P::Inv<'a>,
    appser: P::Ser,
}

impl<'a, P: Policy> Default for Parser<'a, P>
where
    P::Set: Default,
    P::Inv<'a>: Default,
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

impl<'a, P: Policy> Deref for Parser<'a, P> {
    type Target = P::Set;

    fn deref(&self) -> &Self::Target {
        &self.optset
    }
}

impl<'a, P: Policy> DerefMut for Parser<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.optset
    }
}

impl<'a, P> Parser<'a, P>
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

impl<'a, P: Policy> Parser<'a, P> {
    pub fn new_with(policy: P, optset: P::Set, invoker: P::Inv<'a>, valser: P::Ser) -> Self {
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

    pub fn invoker(&self) -> &P::Inv<'a> {
        &self.invoker
    }

    pub fn invoker_mut(&mut self) -> &mut P::Inv<'a> {
        &mut self.invoker
    }

    pub fn set_invoker(&mut self, invser: P::Inv<'a>) -> &mut Self {
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

impl<'a, P> Parser<'a, P>
where
    P::Set: Set,
    P::Ser: ServicesValExt,
    P: Policy,
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
    /// let mut parser = Parser::new(AFwdPolicy::default());
    ///
    /// // Register a value can access in handler parameter.
    /// parser.set_app_data(ser::Value::new(Int(42)))?;
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
    /// # use aopt::ARef;
    /// # use aopt::Error;
    /// # use std::ops::Deref;
    /// #
    /// # fn main() -> Result<(), Error> {
    /// #[derive(Debug)]
    /// struct Int(i64);
    ///
    /// let mut parser = Parser::new(AFwdPolicy::default());
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
    /// getopt!(Args::from_array(["--guess", "42"]), &mut parser)?;
    /// #
    /// # Ok(())
    /// # }
    ///```
    pub fn set_app_data<T: ErasedTy>(&mut self, val: T) -> Result<Option<T>, Error> {
        Ok(self.appser.sve_insert(val))
    }
}

impl<'a, P> Parser<'a, P>
where
    P::Set: Set,
    P: Policy,
{
    /// Call the [`init`](crate::opt::Opt::init) of [`Opt`] initialize the option value.
    pub fn init(&mut self) -> Result<(), Error> {
        let optset = &mut self.optset;

        for opt in optset.iter_mut() {
            opt.init()?;
        }
        Ok(())
    }
}

impl<'a, P> Parser<'a, P>
where
    P::Set: Set,
    P: Policy,
{
    /// Call [`parse`](Policy::parse) parsing the given arguments.
    pub fn parse(&mut self, args: ARef<Args>) -> Result<P::Ret, P::Error> {
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
        let args = crate::ARef::new(Args::from_env());

        self.policy.parse(optset, invser, valser, args)
    }
}

impl<'a, P> Parser<'a, P>
where
    P: Policy,
    SetOpt<P::Set>: Opt,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    P::Set: Set + OptParser + OptValidator + 'a,
    P::Inv<'a>: HandlerCollection<'a, P::Set, P::Ser>,
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
    /// let mut parser1 = Parser::new(AFwdPolicy::default());
    ///
    /// // Add an option `--count` with type `i`.
    /// parser1.add_opt("--count=i")?;
    ///
    /// // Add an option `--len` with type `u`, and get its unique id.
    /// let _len_id = parser1.add_opt("--len=u")?.run()?;
    ///
    /// // Add an option `--size` with type `usize`, it has an alias `-s`.
    /// parser1.add_opt_i::<usize>("--size;-s")?;
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
    /// getopt!(Args::from_array(["app", "foo", "-s", "10", "bar"]), &mut parser1)?;
    ///
    /// assert_eq!(parser1.find_val::<u64>("file=p")?, &0);
    /// assert_eq!(parser1.find_val::<usize>("--size")?, &10);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_opt(
        &mut self,
        opt: impl Into<Str>,
    ) -> Result<ParserCommit<'a, '_, P::Inv<'a>, P::Set, P::Ser, Placeholder>, Error> {
        let info = <SetCfg<P::Set>>::new(&self.optset, opt.into())?;

        Ok(ParserCommit::new(
            SetCommit::new_placeholder(&mut self.optset, info),
            &mut self.invoker,
        ))
    }

    pub fn add_opt_i<U>(
        &mut self,
        opt: impl Into<Str>,
    ) -> Result<ParserCommit<'a, '_, P::Inv<'a>, P::Set, P::Ser, U>, Error>
    where
        U: Infer + 'static,
        U::Val: RawValParser,
    {
        let mut info = <SetCfg<P::Set>>::new(&self.optset, opt.into())?;

        U::infer_fill_info(&mut info, true);
        Ok(ParserCommit::new(
            SetCommit::new(&mut self.optset, info),
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
    /// pub struct Bool;
    ///
    /// impl From<Bool> for OptConfig {
    ///     fn from(_: Bool) -> Self {
    ///         OptConfig::default()
    ///             .with_ctor("b")
    ///             .with_type::<bool>()
    ///             .with_styles(vec![Style::Boolean, Style::Combined])
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
    ///             .with_ctor(ctor_default_name())
    ///             .with_styles(vec![Style::Argument])
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
    ///     parser.init()?;
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
        config: impl Into<SetCfg<P::Set>>,
    ) -> Result<ParserCommit<'a, '_, P::Inv<'a>, P::Set, P::Ser, Placeholder>, Error> {
        Ok(ParserCommit::new(
            SetCommit::new_placeholder(&mut self.optset, config.into()),
            &mut self.invoker,
        ))
    }

    pub fn add_opt_cfg_i<U>(
        &mut self,
        config: impl Into<SetCfg<P::Set>>,
    ) -> Result<ParserCommit<'a, '_, P::Inv<'a>, P::Set, P::Ser, U>, Error>
    where
        U: Infer + 'static,
        U::Val: RawValParser,
    {
        let mut info = config.into();

        U::infer_fill_info(&mut info, true);
        Ok(ParserCommit::new(
            SetCommit::new(&mut self.optset, info),
            &mut self.invoker,
        ))
    }
}

impl<'a, P> Parser<'a, P>
where
    P::Set: Set,
    P: Policy,
    P::Inv<'a>: HandlerCollection<'a, P::Set, P::Ser>,
{
    #[cfg(feature = "sync")]
    #[allow(clippy::type_complexity)]
    pub fn entry<A, O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, P::Inv<'a>, P::Set, P::Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<P::Set, P::Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'a,
        A: Extract<P::Set, P::Ser, Error = Error> + Send + Sync + 'a,
    {
        Ok(HandlerEntry::new(&mut self.invoker, uid))
    }

    #[cfg(not(feature = "sync"))]
    #[allow(clippy::type_complexity)]
    pub fn entry<A, O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, P::Inv<'a>, P::Set, P::Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<P::Set, P::Ser, A, Output = Option<O>, Error = Error> + 'a,
        A: Extract<P::Set, P::Ser, Error = Error> + 'a,
    {
        Ok(HandlerEntry::new(&mut self.invoker, uid))
    }
}

impl<'a, P> PolicySettings for Parser<'a, P>
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

    fn no_delay(&self) -> Option<&[Str]> {
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

    fn set_no_delay(&mut self, name: impl Into<Str>) -> &mut Self {
        self.policy_mut().set_no_delay(name);
        self
    }
}

impl<'a, P> Parser<'a, P>
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
