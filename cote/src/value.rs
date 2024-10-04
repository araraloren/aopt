use aopt::opt::ConfigBuild;
use aopt::opt::ConfigValue;
use aopt::opt::OptValueExt;
use aopt::set::SetCfg;

use crate::prelude::raise_error;
use crate::prelude::Any;
use crate::prelude::ErasedTy;
use crate::prelude::Main;
use crate::prelude::MutOpt;
use crate::prelude::Opt;
use crate::prelude::Placeholder;
use crate::prelude::RefOpt;
use crate::prelude::Set;
use crate::prelude::SetValueFindExt;
use crate::prelude::Uid;
use crate::prelude::{Cmd, Pos};

pub fn fetch_uid_impl<T, S: Set>(uid: Uid, set: &mut S) -> Result<T, aopt::Error>
where
    T: ErasedTy + Sized,
    SetCfg<S>: ConfigValue + Default,
{
    let opt = crate::prelude::SetExt::opt_mut(set, uid)?;
    let (name, uid) = (opt.name(), opt.uid());
    let err = raise_error!(
        "not enough value({}) can take from option `{name}`",
        std::any::type_name::<T>(),
    );

    opt.vals_mut::<T>()?.pop().ok_or_else(|| err.with_uid(uid))
}

pub fn fetch_vec_uid_impl<T, S: Set>(uid: Uid, set: &mut S) -> Result<Vec<T>, aopt::Error>
where
    T: ErasedTy + Sized,
    SetCfg<S>: ConfigValue + Default,
{
    let opt = crate::prelude::SetExt::opt_mut(set, uid)?;
    let (name, uid) = (opt.name(), opt.uid());
    let err = raise_error!(
        "Can not take values({}) of option `{name}`",
        std::any::type_name::<T>(),
    );

    Ok(std::mem::take(
        opt.vals_mut::<T>()
            .map_err(|e| err.with_uid(uid).cause_by(e))?,
    ))
}

/// Using for generate code for procedural macro.
pub trait Fetch<'a, S>
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
    Self: ErasedTy + Sized,
{
    fn fetch(name: impl ConfigBuild<SetCfg<S>>, set: &'a mut S) -> Result<Self, aopt::Error> {
        Self::fetch_uid(set.find_uid(name)?, set)
    }

    fn fetch_vec(
        name: impl ConfigBuild<SetCfg<S>>,
        set: &'a mut S,
    ) -> Result<Vec<Self>, aopt::Error> {
        Self::fetch_vec_uid(set.find_uid(name)?, set)
    }

    fn fetch_uid(uid: Uid, set: &'a mut S) -> Result<Self, aopt::Error> {
        fetch_uid_impl(uid, set)
    }

    fn fetch_vec_uid(uid: Uid, set: &'a mut S) -> Result<Vec<Self>, aopt::Error> {
        fetch_vec_uid_impl(uid, set)
    }
}

#[macro_export]
macro_rules! impl_fetch {
    ($name:path) => {
        impl<'a, S> $crate::prelude::Fetch<'a, S> for $name
        where
            S: $crate::prelude::SetValueFindExt,
            $crate::prelude::SetCfg<S>: $crate::prelude::ConfigValue + Default,
            Self: $crate::prelude::ErasedTy + Sized,
        {   }
    };
    ($name:path, $inner_type:path, $map:expr) => {
        impl<'a, S> $crate::prelude::Fetch<'a, S> for $name
        where
            S: $crate::prelude::SetValueFindExt,
            $crate::prelude::SetCfg<S>: $crate::prelude::ConfigValue + Default,
            Self: $crate::prelude::ErasedTy + Sized, {
            fn fetch_uid(
                uid: $crate::prelude::Uid,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error> {
                $crate::prelude::fetch_uid_impl::<$inner_type, S>(uid, set).map($map)
            }

            fn fetch_vec_uid(
                uid: $crate::prelude::Uid,
                set: &'a mut S,
            ) -> Result<Vec<Self>, aopt::Error> {
                $crate::prelude::fetch_vec_uid_impl::<$inner_type, S>(uid, set)
                    .map(|v| v.into_iter().map($map).collect())
            }
        }
    };
    (&$a:lifetime $name:path) => {
        impl<$a, S> $crate::prelude::Fetch<$a, S> for &$a $name
        where
            S: $crate::prelude::SetValueFindExt,
            $crate::prelude::SetCfg<S>: $crate::prelude::ConfigValue + Default,
            Self: $crate::prelude::ErasedTy + Sized,
         {
            fn fetch_uid(
                uid: $crate::prelude::Uid,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error> {
                $crate::prelude::SetExt::opt(set, uid)?.val::<$name>()
            }

            fn fetch_vec_uid(
                uid: $crate::prelude::Uid,
                set: &'a mut S,
            ) -> Result<Vec<Self>, aopt::Error> {
                $crate::prelude::SetExt::opt(set, uid)
                    .and_then(|v|v.vals::<$name>()).map(|v|v.iter().collect())
            }
        }
    };
    (&$a:lifetime $name:path, $inner:path, $map:expr) => {
        impl<$a, S> $crate::prelude::Fetch<$a, S> for &$a $name
        where
            S: $crate::prelude::SetValueFindExt,
            $crate::prelude::SetCfg<S>: $crate::prelude::ConfigValue + Default,
            Self: $crate::prelude::ErasedTy + Sized, {
            fn fetch_uid(
                uid: $crate::prelude::Uid,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error>{
                $crate::prelude::SetExt::opt(set, uid)?.val::<$inner>().map($map)
            }

            fn fetch_vec_uid(
                uid: $crate::prelude::Uid,
                set: &'a mut S,
            ) -> Result<Vec<Self>, aopt::Error> {
                $crate::prelude::SetExt::opt(set, uid).and_then(|opt| opt.vals::<$inner>())
                    .map(|vals| vals.iter().map($map).collect::<Vec<_>>())
            }
        }
    };
}

macro_rules! value_fetch_forward {
    ($name:path, $map:expr) => {
        impl<'a, S, T> $crate::prelude::Fetch<'a, S> for $name
        where
            T: $crate::prelude::ErasedTy + $crate::prelude::Fetch<'a, S>,
            S: $crate::prelude::SetValueFindExt,
            $crate::prelude::SetCfg<S>: $crate::prelude::ConfigValue + Default,
            Self: $crate::prelude::ErasedTy + Sized,
        {
            fn fetch_uid(uid: Uid, set: &'a mut S) -> Result<Self, aopt::Error> {
                <T as $crate::prelude::Fetch<'a, S>>::fetch_uid(uid, set).map($map)
            }

            fn fetch_vec_uid(uid: Uid, set: &'a mut S) -> Result<Vec<Self>, aopt::Error> {
                <T as $crate::prelude::Fetch<'a, S>>::fetch_vec_uid(uid, set)
                    .map(|v| v.into_iter().map(|v| $map(v)).collect())
            }
        }
    };
    ($name:path, $inner_type:path, $map:expr) => {
        impl<'a, S> $crate::prelude::Fetch<'a, S> for $name
        where
            S: $crate::prelude::SetValueFindExt,
            $crate::prelude::SetCfg<S>: $crate::prelude::ConfigValue + Default,
            Self: $crate::prelude::ErasedTy + Sized,
        {
            fn fetch_uid(uid: Uid, set: &'a mut S) -> Result<Self, aopt::Error> {
                <$inner_type as $crate::prelude::Fetch>::fetch_uid(uid, set).map($map)
            }

            fn fetch_vec_uid(uid: Uid, set: &'a mut S) -> Result<Vec<Self>, aopt::Error> {
                <$inner_type as $crate::prelude::Fetch>::fetch_vec_uid(uid, set)
                    .map(|v| v.into_iter().map(|v| $map(v)).collect())
            }
        }
    };
}

impl_fetch!(Placeholder);

impl_fetch!(bool);

impl_fetch!(f64);

impl_fetch!(f32);

impl_fetch!(i64);

impl_fetch!(u64);

impl_fetch!(i32);

impl_fetch!(u32);

impl_fetch!(i16);

impl_fetch!(u16);

impl_fetch!(i8);

impl_fetch!(u8);

impl_fetch!(i128);

impl_fetch!(u128);

impl_fetch!(isize);

impl_fetch!(usize);

impl_fetch!(String);

impl_fetch!(std::path::PathBuf);

impl_fetch!(std::ffi::OsString);

impl_fetch!(std::io::Stdin);

impl_fetch!(aopt::value::Stop);

impl_fetch!(Cmd, bool, Cmd::new);

impl_fetch!(&'a f64);
impl_fetch!(&'a f32);

impl_fetch!(&'a i8);
impl_fetch!(&'a i16);
impl_fetch!(&'a i32);
impl_fetch!(&'a i64);

impl_fetch!(&'a u8);
impl_fetch!(&'a u16);
impl_fetch!(&'a u32);
impl_fetch!(&'a u64);

impl_fetch!(&'a i128);
impl_fetch!(&'a u128);

impl_fetch!(&'a isize);
impl_fetch!(&'a usize);
impl_fetch!(&'a String);
impl_fetch!(&'a std::path::PathBuf);
impl_fetch!(&'a std::ffi::OsString);
impl_fetch!(&'a std::path::Path, std::path::PathBuf, AsRef::as_ref);
impl_fetch!(&'a str, String, AsRef::as_ref);
impl_fetch!(&'a std::ffi::OsStr, std::ffi::OsString, AsRef::as_ref);

value_fetch_forward!(Pos<T>, Pos::new);

value_fetch_forward!(Any<T>, Any::new);

value_fetch_forward!(Main<T>, Main::new);

impl<'a, S, T: ErasedTy> Fetch<'a, S> for MutOpt<T>
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
    Self: ErasedTy + Sized,
{
    fn fetch_uid(uid: Uid, set: &'a mut S) -> Result<Self, aopt::Error> {
        fetch_uid_impl(uid, set).map(MutOpt::new)
    }

    fn fetch_vec_uid(uid: Uid, set: &'a mut S) -> Result<Vec<Self>, aopt::Error> {
        fetch_vec_uid_impl(uid, set).map(|v| v.into_iter().map(MutOpt::new).collect::<Vec<_>>())
    }
}

impl<'a, 'b, T: ErasedTy, S> Fetch<'a, S> for RefOpt<'b, T>
where
    'a: 'b,
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
    Self: ErasedTy + Sized,
{
    fn fetch_uid(uid: Uid, set: &'a mut S) -> Result<Self, aopt::Error> {
        Ok(RefOpt::new(crate::prelude::SetExt::opt(set, uid)?.val()?))
    }

    fn fetch_vec_uid(uid: Uid, set: &'a mut S) -> Result<Vec<Self>, aopt::Error> {
        crate::prelude::SetExt::opt(set, uid)
            .and_then(|opt| opt.vals())
            .map(|vals| vals.iter().map(RefOpt::new).collect::<Vec<_>>())
    }
}

impl<'a, S> Fetch<'a, S> for ()
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
    Self: ErasedTy + Sized,
{
}
