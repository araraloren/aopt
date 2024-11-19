use aopt::opt::ConfigBuild;
use aopt::opt::ConfigValue;
use aopt::opt::OptValueExt;
use aopt::set::SetCfg;

use crate::prelude::raise_error;
use crate::prelude::ErasedTy;
use crate::prelude::Infer;
use crate::prelude::Opt;
use crate::prelude::Placeholder;
use crate::prelude::RawValParser;
use crate::prelude::Set;
use crate::prelude::SetValueFindExt;
use crate::prelude::Uid;

pub fn fetch_uid_impl<T, S: Set>(uid: Uid, set: &mut S) -> Result<T, aopt::Error>
where
    T: ErasedTy + Sized,
    SetCfg<S>: ConfigValue + Default,
{
    let opt = crate::prelude::SetExt::opt_mut(set, uid)?;
    let (name, uid) = (opt.name(), opt.uid());
    let err = raise_error!(
        "can not take value({}) of option `{name}`",
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
        "can not take values({}) of option `{name}`",
        std::any::type_name::<T>(),
    );

    Ok(std::mem::take(
        opt.vals_mut::<T>()
            .map_err(|e| err.with_uid(uid).cause_by(e))?,
    ))
}

/// Using for generate code for procedural macro.
pub trait Fetch<S>
where
    Self: Infer + Sized,
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
    fn fetch(name: impl ConfigBuild<SetCfg<S>>, set: &mut S) -> Result<Self, aopt::Error> {
        Self::fetch_uid(set.find_uid(name)?, set)
    }

    fn fetch_uid(uid: Uid, set: &mut S) -> Result<Self, aopt::Error> {
        fetch_uid_impl::<<Self as Infer>::Val, S>(uid, set).map(<Self as Infer>::infer_map)
    }
}

#[macro_export]
macro_rules! impl_fetch {
    ($name:path) => {
        impl<S> $crate::prelude::Fetch<S> for $name
        where
            S: $crate::prelude::SetValueFindExt,
            $crate::prelude::SetCfg<S>: $crate::prelude::ConfigValue + Default,
            Self: $crate::prelude::Infer + Sized,
        {
        }
    };
    ($name:path, $inner_type:path, $map:expr) => {
        impl<S> $crate::prelude::Fetch<S> for $name
        where
            S: $crate::prelude::SetValueFindExt,
            $crate::prelude::SetCfg<S>: $crate::prelude::ConfigValue + Default,
            Self: $crate::prelude::Infer + Sized,
        {
            fn fetch_uid(uid: $crate::prelude::Uid, set: &mut S) -> Result<Self, aopt::Error> {
                $crate::prelude::fetch_uid_impl::<$inner_type, S>(uid, set).map($map)
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

impl_fetch!(crate::prelude::Cmd);

impl<S, T: Infer + ErasedTy> Fetch<S> for crate::prelude::AnyOpt<T>
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
}

impl<S, T: Infer + ErasedTy> Fetch<S> for crate::prelude::Pos<T>
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
}

impl<S, T: Infer + ErasedTy> Fetch<S> for crate::prelude::Main<T>
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
}

impl<S, T: RawValParser + ErasedTy> Fetch<S> for crate::prelude::MutOpt<T>
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
}

impl<S> Fetch<S> for ()
where
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
    Self: ErasedTy + Sized,
{
}

impl<S, T> Fetch<S> for Option<T>
where
    T: Fetch<S>,
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
    fn fetch(name: impl ConfigBuild<SetCfg<S>>, set: &mut S) -> Result<Self, aopt::Error> {
        Ok(<T as Fetch<S>>::fetch(name, set).ok())
    }

    fn fetch_uid(uid: Uid, set: &mut S) -> Result<Self, aopt::Error> {
        Ok(<T as Fetch<S>>::fetch_uid(uid, set).ok())
    }
}

impl<S, T> Fetch<S> for Result<T, aopt::Error>
where
    T: Fetch<S>,
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
    fn fetch(name: impl ConfigBuild<SetCfg<S>>, set: &mut S) -> Result<Self, aopt::Error> {
        Ok(<T as Fetch<S>>::fetch(name, set))
    }

    fn fetch_uid(uid: Uid, set: &mut S) -> Result<Self, aopt::Error> {
        Ok(<T as Fetch<S>>::fetch_uid(uid, set))
    }
}

impl<S, T> Fetch<S> for Vec<T>
where
    T: Fetch<S>,
    S: SetValueFindExt,
    SetCfg<S>: ConfigValue + Default,
{
    fn fetch(name: impl ConfigBuild<SetCfg<S>>, set: &mut S) -> Result<Self, aopt::Error> {
        Self::fetch_uid(set.find_uid(name)?, set)
    }

    fn fetch_uid(uid: Uid, set: &mut S) -> Result<Self, aopt::Error> {
        fetch_vec_uid_impl::<<T as Infer>::Val, S>(uid, set)
            .map(|v| v.into_iter().map(<T as Infer>::infer_map).collect())
    }
}
