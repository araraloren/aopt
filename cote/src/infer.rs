use std::ffi::OsString;
use std::io::Stdin;
use std::path::PathBuf;

use crate::prelude::ConfigValue;

pub trait InferOverride {
    fn infer_force() -> bool {
        true
    }

    fn infer_fill_info<C>(cfg: &mut C) -> Result<(), crate::Error>
    where
        C: ConfigValue + Default,
    {
        let force = <Self as InferOverride>::infer_force();

        (!cfg.has_force()).then(|| cfg.set_force(force));
        Ok(())
    }
}

macro_rules! infer_override {
    ($type:ty) => {
        impl $crate::prelude::InferOverride for $type {}
    };
}

impl InferOverride for bool {
    fn infer_force() -> bool {
        false
    }
}

infer_override!(crate::prelude::Cmd);
infer_override!(Stdin);
infer_override!(crate::aopt::value::Stop);
infer_override!(crate::aopt::value::Placeholder);

impl<T: InferOverride> InferOverride for crate::prelude::Pos<T> {
    fn infer_force() -> bool {
        <T as InferOverride>::infer_force()
    }

    fn infer_fill_info<C>(cfg: &mut C) -> Result<(), crate::Error>
    where
        C: ConfigValue + Default,
    {
        <T as InferOverride>::infer_fill_info(cfg)
    }
}
impl<T: InferOverride> InferOverride for crate::prelude::Main<T> {
    fn infer_force() -> bool {
        <T as InferOverride>::infer_force()
    }

    fn infer_fill_info<C>(cfg: &mut C) -> Result<(), crate::Error>
    where
        C: ConfigValue + Default,
    {
        <T as InferOverride>::infer_fill_info(cfg)
    }
}
impl<T> InferOverride for crate::prelude::MutOpt<T> {}
impl<T: InferOverride> InferOverride for crate::prelude::AnyOpt<T> {
    fn infer_force() -> bool {
        <T as InferOverride>::infer_force()
    }

    fn infer_fill_info<C>(cfg: &mut C) -> Result<(), crate::Error>
    where
        C: ConfigValue + Default,
    {
        <T as InferOverride>::infer_fill_info(cfg)
    }
}

infer_override!(f64);
infer_override!(f32);

infer_override!(i8);
infer_override!(i16);
infer_override!(i32);
infer_override!(i64);

infer_override!(u8);
infer_override!(u16);
infer_override!(u32);
infer_override!(u64);

infer_override!(i128);
infer_override!(u128);

infer_override!(isize);
infer_override!(usize);
infer_override!(String);
infer_override!(PathBuf);
infer_override!(OsString);

infer_override!(());

impl<T> InferOverride for Option<T> {
    fn infer_force() -> bool {
        false
    }
}

impl<T, E> InferOverride for Result<T, E> {
    fn infer_force() -> bool {
        false
    }
}

impl<T> InferOverride for Vec<T> {}
