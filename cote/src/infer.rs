use aopt::prelude::{Assoc, Action};

enum InferConvert {
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
infer_reftype_def!(&'b bool, bool, "b", Assoc::Bool, Action::Set, InferConvert::Null);

infer_type_def!(String, String, "s", Assoc::Str, Action::App, InferConvert::Clone);
infer_reftype_def!(&'b String, String, "s", Assoc::Str, Action::App, InferConvert::Null);
infer_reftype_def!(&'b str, String, "s", Assoc::Str, Action::App, InferConvert::AsRef);

infer_type_def!(i64, i64, "i", Assoc::Int, Action::App, InferConvert::Copy);
infer_type_def!(i32, i64, "i", Assoc::Int, Action::App, InferConvert::AsCopy);
infer_type_def!(i16, i64, "i", Assoc::Int, Action::App, InferConvert::AsCopy);
infer_type_def!( i8, i64, "i", Assoc::Int, Action::App, InferConvert::AsCopy);

infer_type_def!(u64, u64, "u", Assoc::Uint, Action::App, InferConvert::Copy);
infer_type_def!(u32, u64, "u", Assoc::Uint, Action::App, InferConvert::AsCopy);
infer_type_def!(u16, u64, "u", Assoc::Uint, Action::App, InferConvert::AsCopy);
infer_type_def!( u8, u64, "u", Assoc::Uint, Action::App, InferConvert::AsCopy);

infer_type_def!(f64, i64, "f", Assoc::Flt, Action::App, InferConvert::Copy);
infer_type_def!(f32, i64, "f", Assoc::Flt, Action::App, InferConvert::AsCopy);

infer_type_def!(isize, isize, "i", Assoc::Int, Action::App, InferConvert::AsCopy);
infer_type_def!(usize, usize, "u", Assoc::Uint, Action::App, InferConvert::AsCopy);

infer_reftype_def!(&'b i64, i64, "i", Assoc::Int, Action::App, InferConvert::Null);
infer_reftype_def!(&'b i32, i64, "i", Assoc::Int, Action::App, InferConvert::Null);
infer_reftype_def!(&'b i16, i64, "i", Assoc::Int, Action::App, InferConvert::Null);
infer_reftype_def!(&'b  i8, i64, "i", Assoc::Int, Action::App, InferConvert::Null);

infer_reftype_def!(&'b u64, u64, "u", Assoc::Uint, Action::App, InferConvert::Null);
infer_reftype_def!(&'b u32, u64, "u", Assoc::Uint, Action::App, InferConvert::Null);
infer_reftype_def!(&'b u16, u64, "u", Assoc::Uint, Action::App, InferConvert::Null);
infer_reftype_def!(&'b  u8, u64, "u", Assoc::Uint, Action::App, InferConvert::Null);

infer_reftype_def!(&'b f64, i64, "f", Assoc::Flt, Action::App, InferConvert::Null);
infer_reftype_def!(&'b f32, i64, "f", Assoc::Flt, Action::App, InferConvert::Null);

infer_reftype_def!(&'b isize, isize, "i", Assoc::Int, Action::App, InferConvert::Null);
infer_reftype_def!(&'b usize, usize, "u", Assoc::Uint, Action::App, InferConvert::Null);