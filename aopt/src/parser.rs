pub(crate) mod commit;
pub(crate) mod policy_delay;
pub(crate) mod policy_fwd;
pub(crate) mod policy_pre;
pub(crate) mod process;
pub(crate) mod style;

pub use self::commit::ParserCommit;
pub use self::policy_delay::DelayPolicy;
pub use self::policy_fwd::FwdPolicy;
pub use self::policy_pre::PrePolicy;
pub use self::style::Guess;
pub use self::style::GuessNOACfg;
pub use self::style::GuessOptCfg;
pub use self::style::NOAGuess;
pub use self::style::OptGuess;
pub use self::style::UserStyle;

pub(crate) use self::process::invoke_callback_opt;
pub(crate) use self::process::process_non_opt;
pub(crate) use self::process::process_opt;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::args::Args;
use crate::ctx::Ctx;
use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ext::APolicyExt;
use crate::ext::ServicesExt;
use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::ser::invoke::HandlerEntry;
use crate::ser::Services;
use crate::set::Commit;
use crate::set::Ctor;
use crate::set::Filter;
use crate::set::Pre;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetOpt;
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

#[derive(Debug, Clone)]
pub struct CtxSaver {
    /// option uid
    pub uid: Uid,

    /// Index of matcher
    pub idx: usize,

    /// invoke context
    pub ctx: Ctx,
}

/// [`Policy`] doing real parsing work.
///
/// # Example
/// ```ignore
///
/// #[derive(Debug)]
/// pub struct EmptyPolicy<S>(PhantomData<S>);
///
/// // An empty policy do nothing.
/// impl<S: Set> Policy for EmptyPolicy<S> {
///     type Ret = bool;
///
///     type Set = S;
///
///     type Error = Error;
///
///     fn parse(&mut self, _: &mut S, _: &mut ASer, _: Arc<Args>) -> Result<Option<bool>, Error> {
///         // ... parsing logical code
///         Ok(Some(true))
///     }
/// }
/// ```
pub trait Policy {
    type Ret;
    type Set: Set;
    type Error: Into<Error>;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        ser: &mut Services,
        args: Arc<Args>,
    ) -> Result<Option<Self::Ret>, Self::Error>;
}

impl<S, R, E> Policy for Box<dyn Policy<Ret = R, Set = S, Error = E>>
where
    S: Set,
    E: Into<Error>,
{
    type Ret = R;

    type Set = S;

    type Error = E;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        ser: &mut Services,
        args: Arc<Args>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        Policy::parse(self.as_mut(), set, ser, args)
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
///     Arc::new(Args::new(["Where", "are", "you", "from", "?"].into_iter())),
///     &mut parser1,
///     &mut parser2
/// )?;
///
/// assert!(ret.is_some());
/// assert_eq!(
///     ret.unwrap()[0].name(),
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
pub struct Parser<S, P> {
    optset: S,
    policy: P,
    services: Services,
}

impl<S, P> Default for Parser<S, P>
where
    S: Default,
    P: Default,
{
    fn default() -> Self {
        Self {
            optset: Default::default(),
            policy: Default::default(),
            services: Default::default(),
        }
    }
}

impl<S, P> Deref for Parser<S, P> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.optset
    }
}

impl<S, P> DerefMut for Parser<S, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.optset
    }
}

impl<P> Parser<P::Set, P>
where
    P: Policy + APolicyExt<P::Set>,
{
    pub fn new(policy: P) -> Self {
        let set = policy.default_set();
        let services = policy.default_ser();

        Self {
            optset: set,
            policy,
            services,
        }
    }
}

impl<P> Parser<P::Set, P>
where
    P: Policy<Error = Error>,
{
    pub fn new_with(policy: P, optset: P::Set, services: Services) -> Self {
        Self {
            optset,
            policy,
            services,
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

    pub fn service(&self) -> &Services {
        &self.services
    }

    pub fn service_mut(&mut self) -> &mut Services {
        &mut self.services
    }

    pub fn set_service(&mut self, services: Services) -> &mut Self {
        self.services = services;
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

    pub fn usr_val<T: 'static>(&self) -> Result<&T, Error> {
        self.services.ser_usrval()?.val::<T>()
    }

    pub fn usr_val_mut<T: 'static>(&mut self) -> Result<&mut T, Error> {
        self.services.ser_usrval_mut()?.val_mut::<T>()
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
    /// parser.set_usr_val(ser::Value::new(Int(42)))?;
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
    /// getopt!(Arc::new(Args::new(["--guess", "42"].into_iter())), &mut parser)?;
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
    /// parser.set_usr_val(Int(42))?;
    /// parser.add_opt("--guess=i!")?.on(
    ///   |_: &mut ASet, ser: &mut ASer, mut val: ctx::Value<i64>| {
    ///       let answer = Int::usr_val(ser)?;
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
    /// getopt!(Arc::new(Args::new(["--guess", "42"].into_iter())), &mut parser)?;
    /// #
    /// # Ok(())
    /// # }
    ///```
    pub fn set_usr_val<T: 'static>(&mut self, val: T) -> Result<Option<T>, Error> {
        Ok(self.services.ser_usrval_mut()?.insert(val))
    }

    pub fn val<T: 'static>(&self, uid: Uid) -> Result<&T, Error> {
        self.services.ser_val()?.val::<T>(uid)
    }

    pub fn val_mut<T: 'static>(&mut self, uid: Uid) -> Result<&mut T, Error> {
        self.services.ser_val_mut()?.val_mut::<T>(uid)
    }

    pub fn vals<T: 'static>(&self, uid: Uid) -> Result<&Vec<T>, Error> {
        self.services.ser_val()?.vals::<T>(uid)
    }

    pub fn vals_mut<T: 'static>(&mut self, uid: Uid) -> Result<&mut Vec<T>, Error> {
        self.services.ser_val_mut()?.vals_mut::<T>(uid)
    }

    pub fn rawval(&self, uid: Uid) -> Result<&RawVal, Error> {
        self.services.ser_rawval()?.val(uid)
    }

    pub fn rawval_mut<T: 'static>(&mut self, uid: Uid) -> Result<&mut RawVal, Error> {
        self.services.ser_rawval_mut()?.val_mut(uid)
    }

    pub fn rawvals<T: 'static>(&self, uid: Uid) -> Result<&Vec<RawVal>, Error> {
        self.services.ser_rawval()?.vals(uid)
    }

    pub fn rawvals_mut<T: 'static>(&mut self, uid: Uid) -> Result<&mut Vec<RawVal>, Error> {
        self.services.ser_rawval_mut()?.vals_mut(uid)
    }
}

impl<P> Parser<P::Set, P>
where
    P: Policy<Error = Error>,
{
    pub fn parse(&mut self, args: Arc<Args>) -> Result<Option<P::Ret>, P::Error> {
        let optset = &mut self.optset;
        let services = &mut self.services;

        self.policy.parse(optset, services, args)
    }
}

impl<P> Parser<P::Set, P>
where
    P::Set: 'static,
    P: Policy<Error = Error>,
    SetOpt<P::Set>: Opt,
    P::Set: Pre + Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
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
    ///     let values = ser.ser_val_mut()?.entry::<u64>(uid).or_insert(vec![0]);
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
    /// // The store will called by `InvokeService` when storing option value.
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
    /// getopt!(Arc::new(Args::new(["foo", "bar"].into_iter())), &mut parser1)?;
    ///
    /// dbg!(parser1.find_val::<u64>("file=p")?);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_opt<T: Into<Str>>(&mut self, opt: T) -> Result<ParserCommit<'_, P::Set>, Error> {
        let info =
            <<<P::Set as Set>::Ctor as Ctor>::Config as Config>::new(&self.optset, opt.into())?;

        Ok(ParserCommit::new(
            Commit::new(&mut self.optset, info),
            self.services.ser_invoke_mut()?,
        ))
    }

    pub fn entry<A, O, H>(&mut self, uid: Uid) -> Result<HandlerEntry<'_, P::Set, H, A, O>, Error>
    where
        O: 'static,
        H: Handler<P::Set, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<P::Set, Error = Error> + 'static,
    {
        Ok(HandlerEntry::new(self.services.ser_invoke_mut()?, uid))
    }
}

impl<P> Parser<P::Set, P>
where
    P: Policy<Error = Error>,
    P::Set: Pre + Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    pub(crate) fn filter_optstr(&self, opt: Str) -> Result<Uid, Error> {
        let filter = Filter::new(
            &self.optset,
            SetCfg::<P::Set>::new(&self.optset, opt.clone())?,
        );
        filter.find().map(|v| v.uid()).ok_or_else(|| {
            Error::raise_error(format!(
                "Can not find option: invalid option string {}",
                opt
            ))
        })
    }

    pub fn find_val<T: 'static>(&self, opt: &str) -> Result<&T, Error> {
        self.val(self.filter_optstr(opt.into())?)
    }

    pub fn find_val_mut<T: 'static>(&mut self, opt: &str) -> Result<&mut T, Error> {
        self.val_mut(self.filter_optstr(opt.into())?)
    }

    pub fn find_vals<T: 'static>(&self, opt: &str) -> Result<&Vec<T>, Error> {
        self.vals(self.filter_optstr(opt.into())?)
    }

    pub fn find_vals_mut<T: 'static>(&mut self, opt: &str) -> Result<&mut Vec<T>, Error> {
        self.vals_mut(self.filter_optstr(opt.into())?)
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        unsafe impl<S, P> Send for Parser<S, P> { }

        unsafe impl<S, P> Sync for Parser<S, P> { }
    }
}
