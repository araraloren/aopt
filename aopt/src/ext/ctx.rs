//! The structs hold the data from [`Cxt`](crate::ctx::Ctx).
//! They are all implemented [`Extract`].
//!
//! # Examples
//! ```rust
//! # use aopt::prelude::*;
//! # use aopt::Arc;
//! # use aopt::Error;
//! # use aopt::RawVal;
//! # use std::ops::Deref;
//! #
//! # fn main() -> Result<(), Error> {
//! let mut policy = AForward::default();
//! let mut set = policy.default_set();
//! let mut ser = policy.default_ser();
//!
//! set.add_opt("--bool=b/")?.run()?;
//! set.add_opt("set=c")?.run()?;
//! set.add_opt("pos_2=p@2")?.run()?;
//! set.add_opt("pos_v=p@>2")?.run()?;
//! ser.ser_invoke_mut::<ASet>()?
//!     .register(0, |_: Uid, _: &mut ASet, disable: ctx::Disable| {
//!         assert_eq!(
//!             &true,
//!             disable.deref(),
//!             "Value is parsed from argument of Ctx which set in Policy"
//!         );
//!         Ok(Some(false))
//!     })
//!     .with_default();
//! ser.ser_invoke_mut::<ASet>()?
//!     .register(1, |_: Uid, _: &mut ASet, val: ctx::Value<String>| {
//!         assert_eq!(
//!             &String::from("set"),
//!             val.deref(),
//!             "Value is parsed from argument of Ctx which set in Policy"
//!         );
//!         Ok(Some(true))
//!     })
//!     .with_default();
//! ser.ser_invoke_mut::<ASet>()?
//!     .register(2, |_: Uid, _: &mut ASet, val: ctx::Value<i64>| {
//!         assert_eq!(
//!             &42,
//!             val.deref(),
//!             "Value is parsed from argument of Ctx which set in Policy"
//!         );
//!         Ok(Some(*val.deref()))
//!     })
//!     .with_default();
//! ser.ser_invoke_mut::<ASet>()?
//!     .register(
//!         3,
//!         |_: Uid, _: &mut ASet, index: ctx::Index, raw_val: ctx::RawVal| {
//!             Ok(Some((*index.deref(), raw_val.deref().clone())))
//!         },
//!     )
//!     .with_default();
//!
//! let args = Args::new(["--/bool", "set", "42", "foo", "bar"].into_iter());
//!
//! policy.parse(Arc::new(args), &mut ser, &mut set)?;
//!
//! assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
//! assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
//! assert_eq!(ser.ser_val()?.val::<i64>(2)?, &42);
//!
//! let test = vec![(3, RawVal::from("foo")), (4, RawVal::from("bar"))];
//!
//! for (idx, val) in ser
//!     .ser_val()?
//!     .vals::<(usize, RawVal)>(3)?
//!     .iter()
//!     .enumerate()
//! {
//!     assert_eq!(val.0, test[idx].0);
//!     assert_eq!(val.1, test[idx].1);
//! }
//! # Ok(())
//! # }
//! ```
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::Extract;
use crate::opt::RawValParser;
use crate::ser::Services;
use crate::set::Set;
use crate::set::SetExt;
use crate::Arc;
use crate::Error;
use crate::Str;

impl<S: Set> Extract<S> for Ctx {
    type Error = Error;

    fn extract(_: crate::Uid, _: &S, _: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(ctx.clone())
    }
}

/// The uid copied from [`Ctx`] which set in [`Policy`](crate::policy::Policy).
///
/// It is same as the uid of matched option in generally.
///
/// # Example
///
/// ```rust
/// # use std::ops::Deref;
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// # use aopt::Arc;
/// #
/// # fn main() -> Result<(), Error> {
///   let mut policy = AForward::default();
///   let mut set = policy.default_set();
///   let mut ser = policy.default_ser();
///
///   set.add_opt("--bool=b/")?.run()?;
///   ser.ser_invoke_mut::<ASet>()?
///       .register(0, |uid: Uid, _: &mut ASet, ctx_uid: ctx::Uid| {
///           assert_eq!(&uid, ctx_uid.deref(), "The uid in Ctx is same as the uid of matched option");
///           Ok(Some(false))
///       }).with_default();
///
///   let args = Args::new(["--/bool", ].into_iter());
///
///   policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
///   assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
///
/// #  Ok(())
/// #
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Uid(crate::Uid);

impl Uid {
    pub fn extract_ctx(ctx: &Ctx) -> Self {
        Self(ctx.uid())
    }
}

impl<S: Set> Extract<S> for Uid {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self::extract_ctx(ctx))
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

/// The index of option/NOA copied from [`Ctx`] which set in [`Policy`](crate::policy::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register(0, |_: Uid, _: &mut ASet, index: ctx::Index| {
///         assert_eq!(
///             &0,
///             index.deref(),
///             "Index is the current index value of Args"
///         );
///         Ok(Some(false))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(1, |_: Uid, _: &mut ASet, index: ctx::Index| {
///         assert_eq!(
///             &1,
///             index.deref(),
///             "Index is the current index value of Args"
///         );
///         Ok(Some(true))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(2, |_: Uid, _: &mut ASet, index: ctx::Index| {
///         assert_eq!(
///             &2,
///             index.deref(),
///             "Index is the current index value of Args"
///         );
///         Ok(Some(2i64))
///     })
///     .with_default();
///
/// let args = Args::new(["--/bool", "set", "value"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
/// assert_eq!(ser.ser_val()?.val::<i64>(2)?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Index(usize);

impl Index {
    pub fn extract_ctx(ctx: &Ctx) -> Self {
        Self(ctx.idx())
    }
}

impl<S: Set> Extract<S> for Index {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self::extract_ctx(ctx))
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

/// The total argument number copied from [`Ctx`] which set in [`Policy`](crate::policy::Policy).
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register(0, |_: Uid, _: &mut ASet, total: ctx::Total| {
///         assert_eq!( &4, total.deref(), "Total is the length of Args");
///         Ok(Some(false))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(1, |_: Uid, _: &mut ASet, total: ctx::Total| {
///         assert_eq!(&3, total.deref(), "Total is the length of Args");
///         Ok(Some(true))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(2, |_: Uid, _: &mut ASet, total: ctx::Total| {
///         assert_eq!(&3, total.deref(), "Total is the length of Args");
///         Ok(Some(2i64))
///     })
///     .with_default();
///
/// let args = Args::new(["--/bool", "set", "value", "foo"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
/// assert_eq!(ser.ser_val()?.val::<i64>(2)?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Total(usize);

impl Total {
    pub fn extract_ctx(ctx: &Ctx) -> Self {
        Self(ctx.total())
    }
}

impl<S: Set> Extract<S> for Total {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self::extract_ctx(ctx))
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

/// The arguments cloned from [`Ctx`] which set in [`Policy`](crate::policy::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register(0, |_: Uid, _: &mut ASet, args: ctx::Args| {
///         let test = Args::new(["--/bool", "set", "value", "foo"].into_iter());
///         for (idx, arg) in args.deref().deref().iter().enumerate() {
///             assert_eq!(arg, &test[idx], "Args is arguments used in Policy");
///         }
///         Ok(Some(false))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(1, |_: Uid, _: &mut ASet, args: ctx::Args| {
///         let test = Args::new(["set", "value", "foo"].into_iter());
///         for (idx, arg) in args.deref().deref().iter().enumerate() {
///             assert_eq!(arg, &test[idx], "Args is arguments used in Policy");
///         }
///         Ok(Some(true))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(2, |_: Uid, _: &mut ASet, args: ctx::Args| {
///         let test = Args::new(["set", "value", "foo"].into_iter());
///         for (idx, arg) in args.deref().deref().iter().enumerate() {
///             assert_eq!(arg, &test[idx], "Args is arguments used in Policy");
///         }
///         Ok(Some(2i64))
///     })
///     .with_default();
///
/// let args = Args::new(["--/bool", "set", "value", "foo"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
/// assert_eq!(ser.ser_val()?.val::<i64>(2)?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Args(Arc<crate::args::Args>);

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

impl<S: Set> Extract<S> for Args {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self::extract_ctx(ctx))
    }
}

/// The name cloned from [`Ctx`] which set in [`Policy`](crate::policy::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register(0, |_: Uid, _: &mut ASet, name: Option<ctx::Name>| {
///         assert_eq!(
///             "bool",
///             name.unwrap().deref().as_ref(),
///             "Name is the name from Ctx set in Policy"
///         );
///         Ok(Some(false))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(1, |_: Uid, _: &mut ASet, name: Option<ctx::Name>| {
///         assert_eq!(
///             "set",
///             name.unwrap().deref().as_ref(),
///             "Name is the name from Ctx set in Policy"
///         );
///         Ok(Some(true))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(2, |_: Uid, _: &mut ASet, name: Option<ctx::Name>| {
///         assert_eq!(
///             "value",
///             name.unwrap().deref().as_ref(),
///             "Name is the name from Ctx set in Policy"
///         );
///         Ok(Some(2i64))
///     })
///     .with_default();
///
/// let args = Args::new(["--/bool", "set", "value", "foo"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
/// assert_eq!(ser.ser_val()?.val::<i64>(2)?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Name(Str);

impl Name {
    pub fn extract_ctx(ctx: &Ctx) -> Result<Self, Error> {
        Ok(Self(
            ctx.name()
                .ok_or_else(|| {
                    Error::raise_error(
                        "Consider using Option<Name> instead, cause the name is an Option in Ctx",
                    )
                })?
                .clone(),
        ))
    }
}

impl<S: Set> Extract<S> for Name {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
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

/// The prefix cloned from [`Ctx`] which set in [`Policy`](crate::policy::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register(0, |_: Uid, _: &mut ASet, prefix: Option<ctx::Prefix>| {
///         assert_eq!(
///             "--",
///             prefix.unwrap().deref().as_ref(),
///             "Prefix is the prefix from Ctx set in Policy"
///         );
///         Ok(Some(false))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(1, |_: Uid, _: &mut ASet, prefix: Option<ctx::Prefix>| {
///         assert_eq!(None, prefix, "Prefix is the prefix from Ctx set in Policy");
///         Ok(Some(true))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(2, |_: Uid, _: &mut ASet, prefix: Option<ctx::Prefix>| {
///         assert_eq!(None, prefix, "Prefix is the prefix from Ctx set in Policy");
///         Ok(Some(2i64))
///     })
///     .with_default();
///
/// let args = Args::new(["--/bool", "set", "value", "foo"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
/// assert_eq!(ser.ser_val()?.val::<i64>(2)?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Prefix(Str);

impl Prefix {
    pub fn extract_ctx(ctx: &Ctx) -> Result<Self, Error> {
        Ok(Self(
            ctx.prefix()
                .ok_or_else(|| {
                    Error::raise_error(
                        "Consider using Option<Prefix> instead, cause the Prefix is an Option in Ctx",
                    )
                })?
                .clone(),
        ))
    }
}

impl<S: Set> Extract<S> for Prefix {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Self::extract_ctx(ctx)
    }
}

impl Deref for Prefix {
    type Target = Str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Prefix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prefix({})", self.0)
    }
}

/// The style copied from [`Ctx`] which set in [`Policy`](crate::policy::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register(0, |_: Uid, _: &mut ASet, style: ctx::Style| {
///         assert_eq!(
///             &OptStyle::Boolean,
///             style.deref(),
///             "Style is the option style copied from Ctx set in Policy"
///         );
///         Ok(Some(false))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(1, |_: Uid, _: &mut ASet, style: ctx::Style| {
///         assert_eq!(
///             &OptStyle::Cmd,
///             style.deref(),
///             "Style is the option style copied from Ctx set in Policy"
///         );
///         Ok(Some(true))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(2, |_: Uid, _: &mut ASet, style: ctx::Style| {
///         assert_eq!(
///             &OptStyle::Pos,
///             style.deref(),
///             "Style is the option style copied from Ctx set in Policy"
///         );
///         Ok(Some(2i64))
///     })
///     .with_default();
///
/// let args = Args::new(["--/bool", "set", "value", "foo"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
/// assert_eq!(ser.ser_val()?.val::<i64>(2)?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style(crate::opt::OptStyle);

impl Style {
    pub fn extract_ctx(ctx: &Ctx) -> Self {
        Self(ctx.style())
    }
}

impl Deref for Style {
    type Target = crate::opt::OptStyle;

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

impl<S: Set> Extract<S> for Style {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self::extract_ctx(ctx))
    }
}

/// The disable value copied from [`Ctx`] which set in [`Policy`](crate::policy::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register(0, |_: Uid, _: &mut ASet, disable: ctx::Disable| {
///         assert_eq!(
///             &true,
///             disable.deref(),
///             "Disable is the disable value copied from Ctx set in Policy"
///         );
///         Ok(Some(false))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(1, |_: Uid, _: &mut ASet, disable: ctx::Disable| {
///         assert_eq!(
///             &false,
///             disable.deref(),
///             "Disable is the disable value copied from Ctx set in Policy"
///         );
///         Ok(Some(true))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(2, |_: Uid, _: &mut ASet, disable: ctx::Disable| {
///         assert_eq!(
///             &false,
///             disable.deref(),
///             "Disable is the disable value copied from Ctx set in Policy"
///         );
///         Ok(Some(2i64))
///     })
///     .with_default();
///
/// let args = Args::new(["--/bool", "set", "value", "foo"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
/// assert_eq!(ser.ser_val()?.val::<i64>(2)?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Disable(bool);

impl Disable {
    pub fn extract_ctx(ctx: &Ctx) -> Self {
        Self(ctx.disable())
    }
}

impl Deref for Disable {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Disable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Disable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Disable({})", self.0)
    }
}

impl<S: Set> Extract<S> for Disable {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self::extract_ctx(ctx))
    }
}

/// The raw value cloned from [`Ctx`] which set in [`Policy`](crate::policy::Policy).
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use aopt::RawVal;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register(0, |_: Uid, _: &mut ASet, raw_val: ctx::RawVal| {
///         assert_eq!(
///             &RawVal::from("false"),
///             raw_val.deref(),
///             "Disable is the disable value copied from Ctx set in Policy"
///         );
///         Ok(Some(false))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(1, |_: Uid, _: &mut ASet, raw_val: ctx::RawVal| {
///         assert_eq!(
///             &RawVal::from("set"),
///             raw_val.deref(),
///             "Disable is the disable value copied from Ctx set in Policy"
///         );
///         Ok(Some(true))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(2, |_: Uid, _: &mut ASet, raw_val: ctx::RawVal| {
///         assert_eq!(
///             &RawVal::from("value"),
///             raw_val.deref(),
///             "Disable is the disable value copied from Ctx set in Policy"
///         );
///         Ok(Some(2i64))
///     })
///     .with_default();
///
/// let args = Args::new(["--/bool", "set", "value", "foo"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
/// assert_eq!(ser.ser_val()?.val::<i64>(2)?, &2);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawVal(crate::RawVal);

impl RawVal {
    pub fn extract_ctx(ctx: &Ctx) -> Result<Self, Error> {
        Ok(Self(
            ctx.arg()
                .ok_or_else(|| {
                    Error::raise_error(
                        "Consider using Option<RawVal> instead, cause the RawVal is an Option in Ctx",
                    )
                })?
                .clone(),
        ))
    }
}

impl Deref for RawVal {
    type Target = crate::RawVal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawVal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<S: Set> Extract<S> for RawVal {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Self::extract_ctx(ctx)
    }
}

/// The [`Value`] will call [`parse`](RawValParser::parse) parsing the argument from [`Ctx`].
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// set.add_opt("--bool=b/")?.run()?;
/// set.add_opt("set=c")?.run()?;
/// set.add_opt("pos_2=p@2")?.run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register(0, |_: Uid, _: &mut ASet, val: ctx::Value<bool>| {
///         assert_eq!(
///             &false,
///             val.deref(),
///             "Value is parsed from argument of Ctx which set in Policy"
///         );
///         Ok(Some(false))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(1, |_: Uid, _: &mut ASet, val: ctx::Value<String>| {
///         assert_eq!(
///             &String::from("set"),
///             val.deref(),
///             "Value is parsed from argument of Ctx which set in Policy"
///         );
///         Ok(Some(true))
///     })
///     .with_default();
/// ser.ser_invoke_mut::<ASet>()?
///     .register(2, |_: Uid, _: &mut ASet, val: ctx::Value<i64>| {
///         assert_eq!(
///             &42,
///             val.deref(),
///             "Value is parsed from argument of Ctx which set in Policy"
///         );
///         Ok(Some(*val.deref()))
///     })
///     .with_default();
///
/// let args = Args::new(["--/bool", "set", "42", "foo"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// assert_eq!(ser.ser_val()?.val::<bool>(0)?, &false);
/// assert_eq!(ser.ser_val()?.val::<bool>(1)?, &true);
/// assert_eq!(ser.ser_val()?.val::<i64>(2)?, &42);
/// #
/// # Ok(())
/// # }
/// ```
pub struct Value<T>(T);

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

impl<S: Set, T: RawValParser<<S as Set>::Opt>> Extract<S> for Value<T> {
    type Error = Error;

    fn extract(uid: crate::Uid, set: &S, _ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(Value(
            T::parse(set.opt(uid)?, ctx.arg(), ctx).map_err(|e| e.into())?,
        ))
    }
}

impl<T> Serialize for Value<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Value<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(T::deserialize(deserializer)?))
    }
}
