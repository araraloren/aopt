use aopt::opt::Any;
use aopt::prelude::ErasedTy;
use aopt::prelude::Main;
use aopt::prelude::MutOpt;
use aopt::prelude::RefOpt;
use aopt::prelude::SetValueFindExt;
use aopt::prelude::{Cmd, Pos};
use aopt::value::Placeholder;

/// Using for generate code for procedural macro.
pub trait InferValueMut<'a> {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, aopt::Error>
    where
        Self: ErasedTy + Sized,
    {
        set.take_val(name)
    }

    fn infer_fetch_vec<S: SetValueFindExt>(
        name: &str,
        set: &'a mut S,
    ) -> Result<Vec<Self>, aopt::Error>
    where
        Self: ErasedTy + Sized,
    {
        set.take_vals(name)
    }
}

#[macro_export]
macro_rules! cote_value_mut_impl {
    ($name:path) => {
        impl<'a> $crate::value::InferValueMut<'a> for $name {}
    };
    ($name:path, $map:expr) => {
        impl<'a, T> $crate::value::InferValueMut<'a> for $name
        where
            T: aopt::prelude::ErasedTy,
        {
            fn infer_fetch<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error>
            where
                Self: ErasedTy + Sized,
            {
                set.take_val::<T>(name).map(|v| $map(v))
            }

            fn infer_fetch_vec<S: aopt::prelude::SetValueFindExt>(
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
        impl<'a> $crate::value::InferValueMut<'a> for $name {
            fn infer_fetch<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                set.take_val::<$inner_type>(name).map(|v| $map(v))
            }

            fn infer_fetch_vec<S: aopt::prelude::SetValueFindExt>(
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
        impl<$a> $crate::value::InferValueMut<$a> for &$a $name {
            fn infer_fetch<S: aopt::prelude::SetValueFindExt>(name: &str, set: &$a mut S) -> Result<Self, aopt::Error>
            where Self: ErasedTy + Sized {
                set.find_val::<$name>(name)
            }

            fn infer_fetch_vec<S: aopt::prelude::SetValueFindExt>(name: &str, set: &$a mut S) -> Result<Vec<Self>, aopt::Error>
            where Self: aopt::prelude::ErasedTy + Sized {
                Ok(set.find_vals::<$name>(name)?.iter().collect())
            }
        }
    };
    (&$a:lifetime $name:path, $inner:path, $map:expr) => {
        impl<$a> $crate::value::InferValueMut<$a> for &$a $name {
            fn infer_fetch<S: aopt::prelude::SetValueFindExt>(name: &str, set: &$a mut S) -> Result<Self, aopt::Error>
            where Self: aopt::prelude::ErasedTy + Sized {
                set.find_val::<$inner>(name).map(|v|$map(v))
            }

            fn infer_fetch_vec<S: aopt::prelude::SetValueFindExt>(name: &str, set: &$a mut S) -> Result<Vec<Self>, aopt::Error>
            where Self: aopt::prelude::ErasedTy + Sized {
                Ok(set.find_vals::<$inner>(name)?.iter().map(|v|$map(v)).collect())
            }
        }
    };
}

macro_rules! cote_inner_fetch {
    ($name:path, $map:expr) => {
        impl<'a, T> $crate::value::InferValueMut<'a> for $name
        where
            T: aopt::prelude::ErasedTy + $crate::value::InferValueMut<'a>,
        {
            fn infer_fetch<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                <T as $crate::value::InferValueMut>::infer_fetch(name, set).map(|v| $map(v))
            }

            fn infer_fetch_vec<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Vec<Self>, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                <T as $crate::value::InferValueMut>::infer_fetch_vec(name, set)
                    .map(|v| v.into_iter().map(|v| $map(v)).collect())
            }
        }
    };
    ($name:path, $inner_type:path, $map:expr) => {
        impl<'a> $crate::value::InferValueMut<'a> for $name {
            fn infer_fetch<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Self, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                <$inner_type as $crate::value::InferValueMut>::infer_fetch(name, set)
                    .map(|v| $map(v))
            }

            fn infer_fetch_vec<S: aopt::prelude::SetValueFindExt>(
                name: &str,
                set: &'a mut S,
            ) -> Result<Vec<Self>, aopt::Error>
            where
                Self: aopt::prelude::ErasedTy + Sized,
            {
                <$inner_type as $crate::value::InferValueMut>::infer_fetch_vec(name, set)
                    .map(|v| v.into_iter().map(|v| $map(v)).collect())
            }
        }
    };
}

cote_value_mut_impl!(Placeholder);

cote_value_mut_impl!(bool);

cote_value_mut_impl!(f64);

cote_value_mut_impl!(f32);

cote_value_mut_impl!(i64);

cote_value_mut_impl!(u64);

cote_value_mut_impl!(i32);

cote_value_mut_impl!(u32);

cote_value_mut_impl!(i16);

cote_value_mut_impl!(u16);

cote_value_mut_impl!(i8);

cote_value_mut_impl!(u8);

cote_value_mut_impl!(i128);

cote_value_mut_impl!(u128);

cote_value_mut_impl!(isize);

cote_value_mut_impl!(usize);

cote_value_mut_impl!(String);

cote_value_mut_impl!(std::path::PathBuf);

cote_value_mut_impl!(std::ffi::OsString);

cote_value_mut_impl!(std::io::Stdin);

cote_value_mut_impl!(Cmd, bool, Cmd::new);

cote_value_mut_impl!(&'a f64);
cote_value_mut_impl!(&'a f32);

cote_value_mut_impl!(&'a i8);
cote_value_mut_impl!(&'a i16);
cote_value_mut_impl!(&'a i32);
cote_value_mut_impl!(&'a i64);

cote_value_mut_impl!(&'a u8);
cote_value_mut_impl!(&'a u16);
cote_value_mut_impl!(&'a u32);
cote_value_mut_impl!(&'a u64);

cote_value_mut_impl!(&'a i128);
cote_value_mut_impl!(&'a u128);

cote_value_mut_impl!(&'a isize);
cote_value_mut_impl!(&'a usize);
cote_value_mut_impl!(&'a String);
cote_value_mut_impl!(&'a std::path::PathBuf);
cote_value_mut_impl!(&'a std::ffi::OsString);
cote_value_mut_impl!(&'a std::path::Path, std::path::PathBuf, AsRef::as_ref);
cote_value_mut_impl!(&'a str, String, AsRef::as_ref);
cote_value_mut_impl!(&'a std::ffi::OsStr, std::ffi::OsString, AsRef::as_ref);

cote_inner_fetch!(Pos<T>, Pos::new);

cote_inner_fetch!(Any<T>, Any::new);

cote_inner_fetch!(Main<T>, Main::new);

cote_value_mut_impl!(MutOpt<T>, MutOpt::new);

impl<'a, 'b, T: ErasedTy> InferValueMut<'a> for RefOpt<'b, T>
where
    'a: 'b,
{
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, aopt::Error>
    where
        Self: ErasedTy + Sized,
    {
        Ok(RefOpt::new(set.find_val::<T>(name)?))
    }

    fn infer_fetch_vec<S: SetValueFindExt>(
        name: &str,
        set: &'a mut S,
    ) -> Result<Vec<Self>, aopt::Error>
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

impl<'a> InferValueMut<'a> for () {}
