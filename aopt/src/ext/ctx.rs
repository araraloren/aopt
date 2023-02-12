//! The structs hold the data from [`Cxt`](crate::ctx::Ctx).
//! They are all implemented [`Extract`].
//!
//! # Examples
//! ```rust
//! # use aopt::prelude::*;
//! # use aopt::ARef;
//! # use aopt::RawVal;
//! # use std::ops::Deref;
//! #
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut policy = AFwdPolicy::default();
//! let mut set = policy.default_set();
//! let mut ser = policy.default_ser();
//! let mut inv = policy.default_inv();
//!
//! set.add_opt("--/bool=b")?.run()?;
//! set.add_opt("set=c")?.run()?;
//! set.add_opt("pos_2=p@2")?.run()?;
//! set.add_opt("pos_v=p@3..")?.run()?;
//! inv.entry(0)
//!     .on(|_: &mut ASet, _: &mut ASer, value: ctx::Value<bool>| {
//!         assert_eq!(
//!             &true,
//!             value.deref(),
//!             "Value is parsed from argument of Ctx which set in Policy"
//!         );
//!         Ok(Some(false))
//!     });
//! inv.entry(1)
//!     .on(|_: &mut ASet, _: &mut ASer, val: ctx::Value<String>| {
//!         assert_eq!(
//!             &String::from("set"),
//!             val.deref(),
//!             "Value is parsed from argument of Ctx which set in Policy"
//!         );
//!         Ok(Some(true))
//!     });
//! inv.entry(2)
//!     .on(|_: &mut ASet, _: &mut ASer, val: ctx::Value<i64>| {
//!         assert_eq!(
//!             &42,
//!             val.deref(),
//!             "Value is parsed from argument of Ctx which set in Policy"
//!         );
//!         Ok(Some(*val.deref()))
//!     });
//! inv.entry(3).on(
//!     |_: &mut ASet, _: &mut ASer, index: ctx::Index, raw_val: ctx::RawVal| {
//!         Ok(Some((*index.deref(), raw_val.deref().clone())))
//!     },
//! );
//!
//! let args = Args::from_array(["app", "--/bool", "set", "42", "foo", "bar"]);
//!
//! policy
//!     .parse(&mut set, &mut inv, &mut ser, ARef::new(args))?
//!     .unwrap();
//!
//! assert_eq!(set.find_val::<bool>("--/bool")?, &false);
//! assert_eq!(set.find_val::<bool>("set")?, &true);
//! assert_eq!(set.find_val::<i64>("pos_2")?, &42);
//!
//! let test = vec![(3, RawVal::from("foo")), (4, RawVal::from("bar"))];
//!
//! for (idx, val) in set[3].vals::<(usize, RawVal)>()?.iter().enumerate() {
//!     assert_eq!(val.0, test[idx].0);
//!     assert_eq!(val.1, test[idx].1);
//! }
//! # Ok(())
//! # }
//!```
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::Extract;
use crate::value::RawValParser;
use crate::ARef;
use crate::Error;
use crate::Str;

impl<Set, Ser> Extract<Set, Ser> for Ctx {
    type Error = Error;

    fn extract(_: &Set, _: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(ctx.clone())
    }
}

/// The uid copied from [`Ctx`] which set in [`Policy`](crate::parser::Policy).
///
/// It is same as the uid of matched option in generally.
///
/// # Example
///
/// ```rust
/// # use std::ops::Deref;
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// # use aopt::ARef;
/// #
/// # fn main() -> Result<(), Error> {
///   let mut policy = AFwdPolicy::default();
///   let mut set = policy.default_set();
///   let mut inv = policy.default_inv();
///   let mut ser = policy.default_ser();
///
///   set.add_opt("--/bool=b")?.run()?;
///   inv.entry(0)
///       .on(|_: &mut ASet, _: &mut ASer, ctx_uid: ctx::Uid| {
///           assert_eq!(&0, ctx_uid.deref(), "The uid in Ctx is same as the uid of matched option");
///           Ok(Some(false))
///       });
///
///   let args = Args::from_array(["app", "--/bool", ]);
///
///   policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args))?.unwrap();
///
///   assert_eq!(set[0].val::<bool>()?, &false);
///
/// #  Ok(())
/// #
/// # }
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uid(crate::Uid);

impl Uid {
    pub fn extract_ctx(ctx: &Ctx) -> Result<Self, Error> {
        Ok(Self(ctx.uid()?))
    }
}

impl<Set, Ser> Extract<Set, Ser> for Uid {
    type Error = Error;

    fn extract(_set: &Set, _ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        Self::extract_ctx(ctx)
    }
}

impl Deref for Uid {
    type Target = crate::Uid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Uid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Uid({})", self.0)
    }
}

/// The index of option/NOA copied from [`Ctx`] which set in [`Policy`](crate::parser::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AFwdPolicy::default();
/// let mut set = policy.default_set();
/// let mut inv = policy.default_inv();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--/bool=b")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// inv.entry(0)
///     .on(|_: &mut ASet, _: &mut ASer, index: ctx::Index| {
///         assert_eq!(
///             &1,
///             index.deref(),
///             "Index is the current index value of Args"
///         );
///         Ok(Some(false))
///     });
///
/// inv.entry(1)
///     .on(|_: &mut ASet, _: &mut ASer, index: ctx::Index| {
///         assert_eq!(
///             &1,
///             index.deref(),
///             "Index is the current index value of Args"
///         );
///         Ok(Some(true))
///     });
///
/// inv.entry(2)
///     .on(|_: &mut ASet, _: &mut ASer, index: ctx::Index| {
///         assert_eq!(
///             &2,
///             index.deref(),
///             "Index is the current index value of Args"
///         );
///         Ok(Some(2i64))
///     });
///
/// let args = Args::from_array(["app", "--/bool", "set", "value"]);
///
/// policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args))?.unwrap();
///
/// assert_eq!(set.find_val::<bool>("--/bool")?, &false);
/// assert_eq!(set[1].val::<bool>()?, &true);
/// assert_eq!(set[2].val::<i64>()?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(usize);

impl Index {
    pub fn extract_ctx(ctx: &Ctx) -> Result<Self, Error> {
        Ok(Self(ctx.idx()?))
    }
}

impl<Set, Ser> Extract<Set, Ser> for Index {
    type Error = Error;

    fn extract(_set: &Set, _ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        Self::extract_ctx(ctx)
    }
}

impl Deref for Index {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Index {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index({})", self.0)
    }
}

/// The total argument number copied from [`Ctx`] which set in [`Policy`](crate::parser::Policy).
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AFwdPolicy::default();
/// let mut set = policy.default_set();
/// let mut inv = policy.default_inv();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--/bool=b")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// inv.entry(0)
///     .on(|_: &mut ASet, _: &mut ASer, total: ctx::Total| {
///         assert_eq!( &5, total.deref(), "Total is the length of Args");
///         Ok(Some(false))
///     });
///
/// inv.entry(1)
///     .on(|_: &mut ASet, _: &mut ASer, total: ctx::Total| {
///         assert_eq!(&4, total.deref(), "Total is the length of Args");
///         Ok(Some(true))
///     });
///
/// inv.entry(2)
///     .on(|_: &mut ASet, _: &mut ASer, total: ctx::Total| {
///         assert_eq!(&4, total.deref(), "Total is the length of Args");
///         Ok(Some(2i64))
///     });
///
/// let args = Args::from_array(["app", "--/bool", "set", "value", "foo"]);
///
/// policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args))?.unwrap();
///
/// assert_eq!(set[0].val::<bool>()?, &false);
/// assert_eq!(set[1].val::<bool>()?, &true);
/// assert_eq!(set[2].val::<i64>()?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Total(usize);

impl Total {
    pub fn extract_ctx(ctx: &Ctx) -> Result<Self, Error> {
        Ok(Self(ctx.total()?))
    }
}

impl<Set, Ser> Extract<Set, Ser> for Total {
    type Error = Error;

    fn extract(_set: &Set, _ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        Self::extract_ctx(ctx)
    }
}

impl Deref for Total {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Total {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Total {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Total({})", self.0)
    }
}

/// The arguments cloned from [`Ctx`] which set in [`Policy`](crate::parser::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AFwdPolicy::default();
/// let mut set = policy.default_set();
/// let mut inv = policy.default_inv();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--/bool=b")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// inv.entry(0)
///     .on(|_: &mut ASet, _: &mut ASer, args: ctx::Args| {
///         let test = Args::from_array(["app", "--/bool", "set", "value", "foo"]);
///         for (idx, arg) in args.deref().deref().iter().enumerate() {
///             assert_eq!(arg, &test[idx], "Args is arguments used in Policy");
///         }
///         Ok(Some(false))
///     });
///
/// inv.entry(1)
///     .on(|_: &mut ASet, _: &mut ASer, args: ctx::Args| {
///         let test = Args::from_array(["app", "set", "value", "foo"]);
///         for (idx, arg) in args.deref().deref().iter().enumerate() {
///             assert_eq!(arg, &test[idx], "Args is arguments used in Policy");
///         }
///         Ok(Some(true))
///     });
///
/// inv.entry(2)
///     .on(|_: &mut ASet, _: &mut ASer, args: ctx::Args| {
///         let test = Args::from_array(["app", "set", "value", "foo"]);
///         for (idx, arg) in args.deref().deref().iter().enumerate() {
///             assert_eq!(arg, &test[idx], "Args is arguments used in Policy");
///         }
///         Ok(Some(2i64))
///     });
///
/// let args = Args::from_array(["app", "--/bool", "set", "value", "foo"]);
///
/// policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args))?.unwrap();
///
/// assert_eq!(set[0].val::<bool>()?, &false);
/// assert_eq!(set[1].val::<bool>()?, &true);
/// assert_eq!(set[2].val::<i64>()?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Args(ARef<crate::args::Args>);

impl Args {
    pub fn extract_ctx(ctx: &Ctx) -> Self {
        Self(ctx.args().clone())
    }
}

impl Deref for Args {
    type Target = crate::args::Args;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Set, Ser> Extract<Set, Ser> for Args {
    type Error = Error;

    fn extract(_set: &Set, _ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(Self::extract_ctx(ctx))
    }
}

/// The name cloned from [`Ctx`] which set in [`Policy`](crate::parser::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AFwdPolicy::default();
/// let mut set = policy.default_set();
/// let mut inv = policy.default_inv();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--/bool=b")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// inv.entry(0)
///     .on(|_: &mut ASet, _: &mut ASer, name: Option<ctx::Name>| {
///         assert_eq!(
///             "--/bool",
///             name.unwrap().deref().as_ref(),
///             "Name is the name from Ctx set in Policy"
///         );
///         Ok(Some(true))
///     });
///
/// inv.entry(1)
///     .on(|_: &mut ASet, _: &mut ASer, name: Option<ctx::Name>| {
///         assert_eq!(
///             "set",
///             name.unwrap().deref().as_ref(),
///             "Name is the name from Ctx set in Policy"
///         );
///         Ok(Some(true))
///     });
///
/// inv.entry(2)
///     .on(|_: &mut ASet, _: &mut ASer, name: Option<ctx::Name>| {
///         assert_eq!(
///             "value",
///             name.unwrap().deref().as_ref(),
///             "Name is the name from Ctx set in Policy"
///         );
///         Ok(Some(2i64))
///     });
///
/// let args = Args::from_array(["app", "--/bool", "set", "value", "foo"]);
///
/// policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args))?.unwrap();
///
/// assert_eq!(set[0].val::<bool>()?, &true);
/// assert_eq!(set[1].val::<bool>()?, &true);
/// assert_eq!(set[2].val::<i64>()?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(Str);

impl Name {
    pub fn extract_ctx(ctx: &Ctx) -> Result<Self, Error> {
        Ok(Self(
            ctx.name()?
                .ok_or_else(|| {
                    Error::sp_extract_error(
                        "consider using Option<Name> instead, Name maybe not exist",
                    )
                })?
                .clone(),
        ))
    }
}

impl<Set, Ser> Extract<Set, Ser> for Name {
    type Error = Error;

    fn extract(_set: &Set, _ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        Self::extract_ctx(ctx)
    }
}

impl Deref for Name {
    type Target = Str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Name {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name({})", self.0)
    }
}

/// The style copied from [`Ctx`] which set in [`Policy`](crate::parser::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AFwdPolicy::default();
/// let mut set = policy.default_set();
/// let mut inv = policy.default_inv();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--/bool=b")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// inv.entry(0)
///     .on(|_: &mut ASet, _: &mut ASer, style: ctx::Style| {
///         assert_eq!(
///             &Style::Boolean,
///             style.deref(),
///             "Style is the option style copied from Ctx set in Policy"
///         );
///         Ok(Some(false))
///     });
///
/// inv.entry(1)
///     .on(|_: &mut ASet, _: &mut ASer, style: ctx::Style| {
///         assert_eq!(
///             &Style::Cmd,
///             style.deref(),
///             "Style is the option style copied from Ctx set in Policy"
///         );
///         Ok(Some(true))
///     });
///
/// inv.entry(2)
///     .on(|_: &mut ASet, _: &mut ASer, style: ctx::Style| {
///         assert_eq!(
///             &Style::Pos,
///             style.deref(),
///             "Style is the option style copied from Ctx set in Policy"
///         );
///         Ok(Some(2i64))
///     });
///
/// let args = Args::from_array(["app", "--/bool", "set", "value", "foo"]);
///
/// policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args))?.unwrap();
///
/// assert_eq!(set.find_val::<bool>("--/bool")?, &false);
/// assert_eq!(set.find_val::<bool>("set")?, &true);
/// assert_eq!(set.find_val::<i64>("pos_2")?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style(crate::opt::Style);

impl Style {
    pub fn extract_ctx(ctx: &Ctx) -> Result<Self, Error> {
        Ok(Self(ctx.style()?))
    }
}

impl Deref for Style {
    type Target = crate::opt::Style;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Style {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Style({})", self.0)
    }
}

impl<Set, Ser> Extract<Set, Ser> for Style {
    type Error = Error;

    fn extract(_set: &Set, _ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        Self::extract_ctx(ctx)
    }
}

/// The raw value cloned from [`Ctx`] which set in [`Policy`](crate::parser::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// # use aopt::RawVal;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AFwdPolicy::default();
/// let mut set = policy.default_set();
/// let mut inv = policy.default_inv();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--/bool=b")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// inv.entry(0)
///     .on(|_: &mut ASet, _: &mut ASer, raw_val: ctx::RawVal| {
///         assert_eq!(
///             &RawVal::from("true"),
///             raw_val.deref(),
///             "RawVal is the raw value copied from Ctx set in Policy"
///         );
///         Ok(Some(false))
///     });
///
/// inv.entry(1)
///     .on(|_: &mut ASet, _: &mut ASer, raw_val: ctx::RawVal| {
///         assert_eq!(
///             &RawVal::from("set"),
///             raw_val.deref(),
///             "RawVal is the raw value copied from Ctx set in Policy"
///         );
///         Ok(Some(true))
///     });
///
/// inv.entry(2)
///     .on(|_: &mut ASet, _: &mut ASer, raw_val: ctx::RawVal| {
///         assert_eq!(
///             &RawVal::from("value"),
///             raw_val.deref(),
///             "RawVal is the raw value copied from Ctx set in Policy"
///         );
///         Ok(Some(2i64))
///     });
///
/// let args = Args::from_array(["app", "--/bool", "set", "value", "foo"]);
///
/// policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args))?.unwrap();
///
/// assert_eq!(set[0].val::<bool>()?, &false);
/// assert_eq!(set[1].val::<bool>()?, &true);
/// assert_eq!(set[2].val::<i64>()?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawVal(ARef<crate::RawVal>);

impl RawVal {
    pub fn extract_ctx(ctx: &Ctx) -> Result<Self, Error> {
        Ok(Self(ctx.arg()?.ok_or_else(|| {
            Error::sp_extract_error("consider using Option<RawVal> instead, RawVal maybe not exist")
        })?))
    }

    pub fn clone_rawval(&self) -> crate::RawVal {
        self.0.as_ref().clone()
    }
}

impl Deref for RawVal {
    type Target = crate::RawVal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Set, Ser> Extract<Set, Ser> for RawVal {
    type Error = Error;

    fn extract(_set: &Set, _ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        Self::extract_ctx(ctx)
    }
}

/// The [`Value`] will call [`parse`](RawValParser::parse) parsing the argument from [`Ctx`].
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AFwdPolicy::default();
/// let mut set = policy.default_set();
/// let mut inv = policy.default_inv();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--/bool=b")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// inv.entry(0)
///     .on(|_: &mut ASet,  _: &mut ASer,val: ctx::Value<bool>| {
///         assert_eq!(
///             &true,
///             val.deref(),
///             "Value is parsed from argument of Ctx which set in Policy"
///         );
///         Ok(Some(false))
///     });
///
/// inv.entry(1)
///     .on(|_: &mut ASet,  _: &mut ASer,val: ctx::Value<String>| {
///         assert_eq!(
///             &String::from("set"),
///             val.deref(),
///             "Value is parsed from argument of Ctx which set in Policy"
///         );
///         Ok(Some(true))
///     });
///
/// inv.entry(2)
///     .on(|_: &mut ASet,  _: &mut ASer,val: ctx::Value<i64>| {
///         assert_eq!(
///             &42,
///             val.deref(),
///             "Value is parsed from argument of Ctx which set in Policy"
///         );
///         Ok(Some(*val.deref()))
///     });
///
/// let args = Args::from_array(["app", "--/bool", "set", "42", "foo"]);
///
/// policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args))?.unwrap();
///
/// assert_eq!(set.find_val::<bool>("--/bool")?, &false);
/// assert_eq!(set.find_val::<bool>("set")?, &true);
/// assert_eq!(set.find_val::<i64>("pos_2")?, &42);
/// #
/// # Ok(())
/// # }
/// ```
pub struct Value<T>(T);

impl<T> Value<T> {
    pub fn replace(&mut self, val: T) -> T {
        std::mem::replace(&mut self.0, val)
    }
}

impl<T: Default> Value<T> {
    pub fn take(&mut self) -> T {
        std::mem::take(&mut self.0)
    }
}

impl<T: Debug> Debug for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Value").field(&self.0).finish()
    }
}

impl<T: Display> Display for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value({})", self.0)
    }
}

impl<T: Clone> Clone for Value<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Default> Default for Value<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: PartialEq<T>> PartialEq<Self> for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq> Eq for Value<T> {}

impl<T: PartialOrd<T>> PartialOrd<Self> for Value<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord> Ord for Value<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: Hash> Hash for Value<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Deref for Value<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Value<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Set: crate::set::Set, Ser, T: RawValParser> Extract<Set, Ser> for Value<T> {
    type Error = Error;

    fn extract(_: &Set, _ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        let arg = ctx.arg()?;
        let arg = arg.as_ref().map(|v| v.as_ref());
        let uid = ctx.uid()?;

        Ok(Value(T::parse(arg, ctx).map_err(|e| {
            Error::sp_extract_error(format!(
                "failed parsing raw value of {{{}}}: {}",
                uid,
                e.into()
            ))
        })?))
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for Value<T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Value<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(T::deserialize(deserializer)?))
    }
}
