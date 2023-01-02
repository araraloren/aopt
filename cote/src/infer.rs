use std::marker::PhantomData;

use aopt::prelude::{Action, Assoc, OptConfig, ValInitiator, ValValidator};

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

    fn infer_ty() -> &'static str;

    fn infer_assoc() -> Assoc;

    fn infer_action() -> Action;

    fn infer_converter() -> InferConvert;

    fn infer_initiator() -> ValInitiator;

    fn infer_validator() -> ValValidator;
}

macro_rules! infer_type_def {
    ($ty:ident, $val:ident, $name:literal, $assoc:expr, $action:expr, $converter:expr, $initiator:block, $validator:block) => {
        impl Infer for $ty {
            type Val = $val;

            fn infer_ty() -> &'static str {
                $name
            }

            fn infer_assoc() -> Assoc {
                $assoc
            }

            fn infer_action() -> Action {
                $action
            }

            fn infer_converter() -> InferConvert {
                $converter
            }

            fn infer_initiator() -> ValInitiator {
                $initiator
            }

            fn infer_validator() -> ValValidator {
                $validator
            }
        }
    };
}

macro_rules! infer_reftype_def {
    (&$a:lifetime $ty:ident, $val:ident, $name:literal, $converter:expr) => {
            impl<$a> Infer for &$a $ty {
                type Val = $val;

                fn infer_ty() -> &'static str {
                    $name
                }

                fn infer_assoc() -> Assoc {
                    <Self::Val>::infer_assoc()
                }

                fn infer_action() -> Action {
                    <Self::Val>::infer_action()
                }

                fn infer_converter() -> InferConvert {
                    $converter
                }

                fn infer_initiator() -> ValInitiator {
                    <Self::Val>::infer_initiator()
                }

                fn infer_validator() -> ValValidator {
                    <Self::Val>::infer_validator()
                }
            }
    };
}

infer_type_def!(
    bool,
    bool,
    "b",
    Assoc::Bool,
    Action::Set,
    InferConvert::Copy,
    { ValInitiator::bool(false) },
    { ValValidator::bool() }
);
infer_reftype_def!(&'a bool, bool, "b", InferConvert::Null);

infer_type_def!(
    String,
    String,
    "s",
    Assoc::Str,
    Action::App,
    InferConvert::Clone,
    { ValInitiator::empty::<String>() },
    { ValValidator::str() }
);
infer_reftype_def!(&'a String, String, "s", InferConvert::Null);
infer_reftype_def!(&'a str, String, "s", InferConvert::AsRef);

infer_type_def!(
    i128,
    i128,
    "i",
    Assoc::Int128,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<i128>() },
    { ValValidator::i128() }
);
infer_type_def!(
    i64,
    i64,
    "i",
    Assoc::Int64,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<i64>() },
    { ValValidator::i64() }
);
infer_type_def!(
    i32,
    i32,
    "i",
    Assoc::Int32,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<i32>() },
    { ValValidator::i32() }
);
infer_type_def!(
    i16,
    i16,
    "i",
    Assoc::Int16,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<i16>() },
    { ValValidator::i16() }
);
infer_type_def!(
    i8,
    i8,
    "i",
    Assoc::Int8,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<i8>() },
    { ValValidator::i8() }
);

infer_type_def!(
    u128,
    u128,
    "u",
    Assoc::Uint128,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<u128>() },
    { ValValidator::u128() }
);
infer_type_def!(
    u64,
    u64,
    "u",
    Assoc::Uint64,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<u64>() },
    { ValValidator::u64() }
);
infer_type_def!(
    u32,
    u32,
    "u",
    Assoc::Uint32,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<u32>() },
    { ValValidator::u32() }
);
infer_type_def!(
    u16,
    u16,
    "u",
    Assoc::Uint16,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<u16>() },
    { ValValidator::u16() }
);
infer_type_def!(
    u8,
    u8,
    "u",
    Assoc::Uint8,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<u8>() },
    { ValValidator::u8() }
);

infer_type_def!(
    f64,
    f64,
    "f",
    Assoc::Flt64,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<f64>() },
    { ValValidator::f64() }
);
infer_type_def!(
    f32,
    f32,
    "f",
    Assoc::Flt32,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<f32>() },
    { ValValidator::f32() }
);

infer_type_def!(
    isize,
    isize,
    "i",
    Assoc::ISize,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<isize>() },
    { ValValidator::isize() }
);
infer_type_def!(
    usize,
    usize,
    "u",
    Assoc::USize,
    Action::App,
    InferConvert::Copy,
    { ValInitiator::empty::<usize>() },
    { ValValidator::usize() }
);

infer_reftype_def!(&'a i128, i128, "i", InferConvert::Copy);
infer_reftype_def!(&'a i64, i64, "i", InferConvert::Null);
infer_reftype_def!(&'a i32, i64, "i", InferConvert::Null);
infer_reftype_def!(&'a i16, i64, "i", InferConvert::Null);
infer_reftype_def!(&'a i8, i64, "i", InferConvert::Null);

infer_reftype_def!(&'a u128, u128, "u", InferConvert::Copy);
infer_reftype_def!(&'a u64, u64, "u", InferConvert::Null);
infer_reftype_def!(&'a u32, u64, "u", InferConvert::Null);
infer_reftype_def!(&'a u16, u64, "u", InferConvert::Null);
infer_reftype_def!(&'a u8, u64, "u", InferConvert::Null);

infer_reftype_def!(&'a f64, i64, "f", InferConvert::Null);
infer_reftype_def!(&'a f32, i64, "f", InferConvert::Null);

infer_reftype_def!(&'a isize, isize, "i", InferConvert::Null);
infer_reftype_def!(&'a usize, usize, "u", InferConvert::Null);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cmd;

infer_type_def!(
    Cmd,
    bool,
    "c",
    Assoc::Bool,
    Action::Set,
    InferConvert::Copy,
    { ValInitiator::bool(false) },
    { ValValidator::some() }
);

infer_reftype_def!(&'a Cmd, bool, "c", InferConvert::Null);

#[derive(Debug, Clone, Copy, Default)]
pub struct InferConfig<T: Infer>(PhantomData<T>);

impl<T: Infer> InferConfig<T> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }
}

impl<I: Infer> From<InferConfig<I>> for OptConfig {
    fn from(_: InferConfig<I>) -> Self {
        OptConfig::default()
            .with_type(I::infer_ty())
            .with_assoc(Some(I::infer_assoc()))
            .with_action(Some(I::infer_action()))
            .with_initiator(Some(I::infer_initiator()))
            .with_validator(Some(I::infer_validator()))
    }
}
