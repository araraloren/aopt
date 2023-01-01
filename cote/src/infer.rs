use aopt::prelude::{Assoc, Action};

pub enum InferConvert {
    AsRef,

    AsMut,

    Deref,

    DerefMut,

    Clone,

    Copy,

    AsCopy,

    Null,
}

pub trait Infer {
    type Val;

    fn ty() -> &'static str;

    fn assoc() -> Assoc;

    fn action() -> Action;

    fn converter() -> InferConvert;
}

macro_rules! infer_type_def {
    ($ty:ident, $val:ident, $name:literal, $assoc:expr, $action:expr, $converter:expr) => {
            impl Infer for $ty {
                type Val = $val;

                fn ty() -> &'static str {
                    $name
                }
            
                fn assoc() -> Assoc {
                    $assoc
                }
            
                fn action() -> Action {
                    $action
                }

                fn converter() -> InferConvert {
                    $converter
                }
            }
    };
}

macro_rules! infer_reftype_def {
    (&$a:lifetime $ty:ident, $val:ident, $name:literal, $assoc:expr, $action:expr, $converter:expr) => {
            impl<$a> Infer for &$a $ty {
                type Val = $val;

                fn ty() -> &'static str {
                    $name
                }
            
                fn assoc() -> Assoc {
                    $assoc
                }
            
                fn action() -> Action {
                    $action
                }

                fn converter() -> InferConvert {
                    $converter
                }
            }
    };
}

infer_type_def!(bool, bool, "b", Assoc::Bool, Action::Set, InferConvert::Copy);
infer_reftype_def!(&'a bool, bool, "b", Assoc::Bool, Action::Set, InferConvert::Null);

infer_type_def!(String, String, "s", Assoc::Str, Action::App, InferConvert::Clone);
infer_reftype_def!(&'a String, String, "s", Assoc::Str, Action::App, InferConvert::Null);
infer_reftype_def!(&'a str, String, "s", Assoc::Str, Action::App, InferConvert::AsRef);

infer_type_def!(i128, i128, "i", Assoc::Int128, Action::App, InferConvert::Copy);
infer_type_def!(i64, i64, "i", Assoc::Int64, Action::App, InferConvert::Copy);
infer_type_def!(i32, i64, "i", Assoc::Int32, Action::App, InferConvert::Copy);
infer_type_def!(i16, i64, "i", Assoc::Int16, Action::App, InferConvert::Copy);
infer_type_def!( i8, i64, "i",  Assoc::Int8, Action::App, InferConvert::Copy);

infer_type_def!(u128, u128, "u", Assoc::Uint128, Action::App, InferConvert::Copy);
infer_type_def!(u64, u64, "u", Assoc::Uint64, Action::App, InferConvert::Copy);
infer_type_def!(u32, u64, "u", Assoc::Uint32, Action::App, InferConvert::Copy);
infer_type_def!(u16, u64, "u", Assoc::Uint16, Action::App, InferConvert::Copy);
infer_type_def!( u8, u64, "u",  Assoc::Uint8, Action::App, InferConvert::Copy);

infer_type_def!(f64, i64, "f", Assoc::Flt64, Action::App, InferConvert::Copy);
infer_type_def!(f32, i64, "f", Assoc::Flt32, Action::App, InferConvert::Copy);

infer_type_def!(isize, isize, "i", Assoc::ISize, Action::App, InferConvert::Copy);
infer_type_def!(usize, usize, "u", Assoc::USize, Action::App, InferConvert::Copy);

infer_reftype_def!(&'a i128, i128, "i", Assoc::Int128, Action::App, InferConvert::Copy);
infer_reftype_def!(&'a i64, i64, "i", Assoc::Int64, Action::App, InferConvert::Null);
infer_reftype_def!(&'a i32, i64, "i", Assoc::Int32, Action::App, InferConvert::Null);
infer_reftype_def!(&'a i16, i64, "i", Assoc::Int16, Action::App, InferConvert::Null);
infer_reftype_def!(&'a  i8, i64, "i",  Assoc::Int8, Action::App, InferConvert::Null);

infer_reftype_def!(&'a u128, u128, "u", Assoc::Uint128, Action::App, InferConvert::Copy);
infer_reftype_def!(&'a u64, u64, "u", Assoc::Uint64, Action::App, InferConvert::Null);
infer_reftype_def!(&'a u32, u64, "u", Assoc::Uint32, Action::App, InferConvert::Null);
infer_reftype_def!(&'a u16, u64, "u", Assoc::Uint16, Action::App, InferConvert::Null);
infer_reftype_def!(&'a  u8, u64, "u",  Assoc::Uint8, Action::App, InferConvert::Null);

infer_reftype_def!(&'a f64, i64, "f", Assoc::Flt64, Action::App, InferConvert::Null);
infer_reftype_def!(&'a f32, i64, "f", Assoc::Flt32, Action::App, InferConvert::Null);

infer_reftype_def!(&'a isize, isize, "i", Assoc::ISize, Action::App, InferConvert::Null);
infer_reftype_def!(&'a usize, usize, "u", Assoc::USize, Action::App, InferConvert::Null);