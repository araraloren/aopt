use aopt::opt::Any;
use aopt::prelude::ErasedTy;
use aopt::prelude::Main;
use aopt::prelude::MutOpt;
use aopt::prelude::RefOpt;
use aopt::prelude::SetValueFindExt;
use aopt::prelude::{Cmd, Pos};
use aopt::value::Placeholder;

/// Using for generate code for procedural macro.
pub trait Fetch<'a> {
    fn fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, aopt::Error>
    where
        Self: ErasedTy + Sized,
    {
        set.take_val(name)
    }

    fn fetch_vec<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Vec<Self>, aopt::Error>
    where
        Self: ErasedTy + Sized,
    {
        set.take_vals(name)
    }
}

#[macro_export]
macro_rules! impl_fetch {
    ($name:path) => {
        impl<'a> $crate::value::Fetch<'a> for $name {}
    };
    ($name:path, $map:expr) => {
        impl<'a, T> $crate::value::Fetch<'a> for $name
        where
            T: aopt::prelude::ErasedTy,
        {
            fn fetch<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error>
            where
                Self: ErasedTy + Sized,
            {
                set.take_val::<T>(name).map(|v| $map(v))
            }

            fn fetch_vec<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Vec<Self>, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                set.take_vals::<T>(name)
                    .map(|v| v.into_iter().map(|v| $map(v)).collect())
            }
        }
    };
    ($name:path, $inner_type:path, $map:expr) => {
        impl<'a> $crate::value::Fetch<'a> for $name {
            fn fetch<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                set.take_val::<$inner_type>(name).map(|v| $map(v))
            }

            fn fetch_vec<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Vec<Self>, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                set.take_vals::<$inner_type>(name)
                    .map(|v| v.into_iter().map(|v| $map(v)).collect())
            }
        }
    };
    (&$a:lifetime $name:path) => {
        impl<$a> $crate::value::Fetch<$a> for &$a $name {
            fn fetch<S: aopt::prelude::SetValueFindExt>(name: &str, set: &$a mut S) -> Result<Self, aopt::Error>
            where Self: ErasedTy + Sized {
                set.find_val::<$name>(name)
            }

            fn fetch_vec<S: aopt::prelude::SetValueFindExt>(name: &str, set: &$a mut S) -> Result<Vec<Self>, aopt::Error>
            where Self: aopt::prelude::ErasedTy + Sized {
                Ok(set.find_vals::<$name>(name)?.iter().collect())
            }
        }
    };
    (&$a:lifetime $name:path, $inner:path, $map:expr) => {
        impl<$a> $crate::value::Fetch<$a> for &$a $name {
            fn fetch<S: aopt::prelude::SetValueFindExt>(name: &str, set: &$a mut S) -> Result<Self, aopt::Error>
            where Self: aopt::prelude::ErasedTy + Sized {
                set.find_val::<$inner>(name).map(|v|$map(v))
            }

            fn fetch_vec<S: aopt::prelude::SetValueFindExt>(name: &str, set: &$a mut S) -> Result<Vec<Self>, aopt::Error>
            where Self: aopt::prelude::ErasedTy + Sized {
                Ok(set.find_vals::<$inner>(name)?.iter().map(|v|$map(v)).collect())
            }
        }
    };
}

macro_rules! value_fetch_forward {
    ($name:path, $map:expr) => {
        impl<'a, T> $crate::value::Fetch<'a> for $name
        where
            T: aopt::prelude::ErasedTy + $crate::value::Fetch<'a>,
        {
            fn fetch<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                <T as $crate::value::Fetch>::fetch(name, set).map(|v| $map(v))
            }

            fn fetch_vec<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Vec<Self>, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                <T as $crate::value::Fetch>::fetch_vec(name, set)
                    .map(|v| v.into_iter().map(|v| $map(v)).collect())
            }
        }
    };
    ($name:path, $inner_type:path, $map:expr) => {
        impl<'a> $crate::value::Fetch<'a> for $name {
            fn fetch<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                <$inner_type as $crate::value::Fetch>::fetch(name, set).map(|v| $map(v))
            }

            fn fetch_vec<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Vec<Self>, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                <$inner_type as $crate::value::Fetch>::fetch_vec(name, set)
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

impl_fetch!(MutOpt<T>, MutOpt::new);

impl<'a, 'b, T: ErasedTy> Fetch<'a> for RefOpt<'b, T>
where
    'a: 'b,
{
    fn fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, aopt::Error>
    where
        Self: ErasedTy + Sized,
    {
        Ok(RefOpt::new(set.find_val::<T>(name)?))
    }

    fn fetch_vec<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Vec<Self>, aopt::Error>
    where
        Self: ErasedTy + Sized,
    {
        Ok(set
            .find_vals(name)?
            .iter()
            .map(|v| RefOpt::new(v))
            .collect())
    }
}

impl<'a> Fetch<'a> for () {}
