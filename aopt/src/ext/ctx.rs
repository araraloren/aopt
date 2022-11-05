use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::ctx::Ctx;
use crate::ctx::ExtractCtx;
use crate::opt::RawValParser;
use crate::ser::Services;
use crate::set::Set;
use crate::set::SetExt;
use crate::Arc;
use crate::Error;
use crate::Str;

/// The uid of [`Ctx`]'s which set in [`Policy`].
///
/// It is same as the uid of matched option in generally.
///
/// # Example
///
/// ```rust
/// # use std::ops::Deref;
/// # use aopt::prelude::*;
/// # use aopt::Error;
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
///       }).or_default();
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
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uid(crate::Uid);

impl<S: Set> ExtractCtx<S> for Uid {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Uid(ctx.uid()))
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

/// The argument index from [`Ctx`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(usize);

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

impl<S: Set> ExtractCtx<S> for Index {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self(ctx.idx()))
    }
}

/// The total argument number from [`Ctx`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Total(usize);

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

impl<S: Set> ExtractCtx<S> for Total {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self(ctx.total()))
    }
}

/// The arguments from [`Ctx`].
#[derive(Debug, Clone, Default)]
pub struct Args(Arc<crate::args::Args>);

impl Deref for Args {
    type Target = crate::args::Args;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Set> ExtractCtx<S> for Args {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self(ctx.args().clone()))
    }
}

/// The name from [`Ctx`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(Str);

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

impl<S: Set> ExtractCtx<S> for Name {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
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

/// The prefix from [`Ctx`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Prefix(Str);

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

impl<S: Set> ExtractCtx<S> for Prefix {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
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

/// The style from [`Ctx`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Style(crate::opt::OptStyle);

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

impl<S: Set> ExtractCtx<S> for Style {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self(ctx.style()))
    }
}

/// The disable from [`Ctx`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Disable(bool);

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

impl<S: Set> ExtractCtx<S> for Disable {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
        Ok(Self(ctx.disable()))
    }
}

/// The argument from [`Ctx`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawVal(crate::RawVal);

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

impl<S: Set> ExtractCtx<S> for RawVal {
    type Error = Error;

    fn extract(
        _uid: crate::Uid,
        _set: &S,
        _ser: &Services,
        ctx: &Ctx,
    ) -> Result<Self, Self::Error> {
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

/// The [`Value`] will call [`parse`](RawValParser::parse) parsing the argument from [`Ctx`].
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

impl<S: Set, T: RawValParser<<S as Set>::Opt>> ExtractCtx<S> for Value<T> {
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
