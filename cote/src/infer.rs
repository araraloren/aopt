use aopt::prelude::{Assoc, Action};

pub trait Infer {
    type Val;

    type Ref<'a>;

    fn ty() -> &'static str;

    fn assoc() -> Assoc;

    fn action() -> Action;
}

macro_rules! infer_type_def {
    ($ty:ident, $val:ident, &$a:lifetime $ref:ident, $name:literal, $assoc:expr, $action:expr) => {
            impl Infer for $ty {
                type Val = $val;

                type Ref<$a> = &$a $ref;

                fn ty() -> &'static str {
                    $name
                }
            
                fn assoc() -> Assoc {
                    $assoc
                }
            
                fn action() -> Action {
                    $action
                }
            }
    };
}

macro_rules! infer_reftype_def {
    (&$b:lifetime $ty:ident, $val:ident, &$a:lifetime $ref:ident, $name:literal, $assoc:expr, $action:expr) => {
            impl<$b> Infer for &$b $ty {
                type Val = $val;

                type Ref<$a> = &$a $ref;

                fn ty() -> &'static str {
                    $name
                }
            
                fn assoc() -> Assoc {
                    $assoc
                }
            
                fn action() -> Action {
                    $action
                }
            }
    };
}

infer_type_def!(bool, bool, &'a bool, "b", Assoc::Bool, Action::Set);
infer_reftype_def!(&'b bool, bool, &'a bool, "b", Assoc::Bool, Action::Set);

infer_type_def!(String, String, &'a String, "s", Assoc::Str, Action::App);
infer_reftype_def!(&'b String, String, &'a String, "s", Assoc::Str, Action::App);
infer_reftype_def!(&'b str, String, &'a String, "s", Assoc::Str, Action::App);

infer_type_def!(i64, i64, &'a i64, "i", Assoc::Int, Action::App);
infer_type_def!(i32, i64, &'a i64, "i", Assoc::Int, Action::App);
infer_type_def!(i16, i64, &'a i64, "i", Assoc::Int, Action::App);
infer_type_def!( i8, i64, &'a i64, "i", Assoc::Int, Action::App);

infer_type_def!(u64, u64, &'a u64, "u", Assoc::Uint, Action::App);
infer_type_def!(u32, u64, &'a u64, "u", Assoc::Uint, Action::App);
infer_type_def!(u16, u64, &'a u64, "u", Assoc::Uint, Action::App);
infer_type_def!( u8, u64, &'a u64, "u", Assoc::Uint, Action::App);

infer_type_def!(f64, i64, &'a f64, "f", Assoc::Flt, Action::App);
infer_type_def!(f32, i64, &'a f64, "f", Assoc::Flt, Action::App);

infer_reftype_def!(&'b i64, i64, &'a i64, "i", Assoc::Int, Action::App);
infer_reftype_def!(&'b i32, i64, &'a i64, "i", Assoc::Int, Action::App);
infer_reftype_def!(&'b i16, i64, &'a i64, "i", Assoc::Int, Action::App);
infer_reftype_def!(&'b  i8, i64, &'a i64, "i", Assoc::Int, Action::App);

infer_reftype_def!(&'b u64, u64, &'a u64, "u", Assoc::Uint, Action::App);
infer_reftype_def!(&'b u32, u64, &'a u64, "u", Assoc::Uint, Action::App);
infer_reftype_def!(&'b u16, u64, &'a u64, "u", Assoc::Uint, Action::App);
infer_reftype_def!(&'b  u8, u64, &'a u64, "u", Assoc::Uint, Action::App);

infer_reftype_def!(&'b f64, i64, &'a f64, "f", Assoc::Flt, Action::App);
infer_reftype_def!(&'b f32, i64, &'a f64, "f", Assoc::Flt, Action::App);