use crate::ext::ServicesExt;
use crate::opt::RawValParser;
use crate::Error;
use crate::{
    prelude::{Ctx, Services},
    RawVal,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ValAction {
    Set,

    App,

    Pop,

    Cnt,

    Null,
}

impl Default for ValAction {
    fn default() -> Self {
        ValAction::Null
    }
}

impl std::fmt::Display for ValAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValAction::Set => {
                write!(f, "ValAction::Set")
            }
            ValAction::App => {
                write!(f, "ValAction::App")
            }
            ValAction::Pop => {
                write!(f, "ValAction::Pop")
            }
            ValAction::Cnt => {
                write!(f, "ValAction::Cnt")
            }
            ValAction::Null => {
                write!(f, "ValAction::Null")
            }
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ValAssoc {
    Bool,

    Int,

    Uint,

    Flt,

    Str,

    Noa,

    Null,
}

impl Default for ValAssoc {
    fn default() -> Self {
        ValAssoc::Null
    }
}

impl std::fmt::Display for ValAssoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValAssoc::Bool => {
                write!(f, "ValAssoc::Bool")
            }
            ValAssoc::Int => {
                write!(f, "ValAssoc::Int")
            }
            ValAssoc::Uint => {
                write!(f, "ValAssoc::Uint")
            }
            ValAssoc::Flt => {
                write!(f, "ValAssoc::Flt")
            }
            ValAssoc::Str => {
                write!(f, "ValAssoc::Str")
            }
            ValAssoc::Noa => {
                write!(f, "ValAssoc::Noa")
            }
            ValAssoc::Null => {
                write!(f, "ValAssoc::Null")
            }
        }
    }
}

pub type ActionImpl<Opt> = Box<dyn FnMut(&Opt, &mut Services, &Ctx) -> Result<Option<()>, Error>>;

pub struct Action<Opt>(ActionImpl<Opt>);

macro_rules! def_action_for {
    ($num:ty, $name:ident) => {
        pub fn $name() -> Self {
            Self(Box::new(move |opt: &Opt, ser: &mut Services, ctx: &Ctx| {
                let arg = ctx.arg();
                let val = arg.as_ref().map(|v| v.as_ref());

                Self::apply_action_on(opt, ser, val, <$num>::parse(opt, val, ctx).ok())
            }))
        }
    };
}

impl<Opt> Action<Opt>
where
    Opt: crate::opt::Opt,
{
    pub fn apply_action_on<Value: 'static>(
        opt: &Opt,
        ser: &mut Services,
        raw: Option<&RawVal>,
        val: Option<Value>,
    ) -> Result<Option<()>, Error> {
        let uid = opt.uid();
        let has_value = val.is_some();

        // Set the value if return Some(Value)
        if let Some(val) = val {
            let raw_ser = ser.ser_rawval_mut()?;

            if let Some(raw) = raw {
                raw_ser.push(uid, raw.clone());
            }

            let action = opt.action();
            let val_ser = ser.ser_val_mut()?;

            match action {
                ValAction::Set => {
                    val_ser.set(uid, vec![val]);
                }
                ValAction::App => {
                    val_ser.push(uid, val);
                }
                ValAction::Pop => {
                    val_ser.pop::<Value>(uid);
                }
                ValAction::Cnt => {
                    val_ser.entry::<u64>(uid).or_insert(vec![0])[0] += 1;
                }
                ValAction::Null => {
                    //DO NOTHING
                }
            }
        }
        Ok(has_value.then(|| ()))
    }

    def_action_for!(f64, f64_action);

    def_action_for!(f32, f32_action);

    def_action_for!(i8, i8_action);

    def_action_for!(i16, i16_action);

    def_action_for!(i32, i32_action);

    def_action_for!(i64, i64_action);

    def_action_for!(u8, u8_action);

    def_action_for!(u16, u16_action);

    def_action_for!(u32, u32_action);

    def_action_for!(u64, u64_action);

    def_action_for!(bool, bool_action);

    def_action_for!(usize, usize_action);

    def_action_for!(isize, isize_action);

    def_action_for!(String, str_action);
}

pub trait ValActionExt {
    type Output<Opt>;

    fn val_action<Opt>() -> Action<Opt>
    where
        Opt: crate::opt::Opt;
}

macro_rules! impl_action_ext_for {
    ($num:ty, $name:ident) => {
        impl ValActionExt for $num {
            type Output<Opt> = Action<Opt>;

            fn val_action<Opt>() -> Action<Opt>
            where
                Opt: crate::opt::Opt,
            {
                Action::<Opt>::$name()
            }
        }
    };
}

impl_action_ext_for!(f64, f64_action);

impl_action_ext_for!(f32, f32_action);

impl_action_ext_for!(i8, i8_action);

impl_action_ext_for!(i16, i16_action);

impl_action_ext_for!(i32, i32_action);

impl_action_ext_for!(i64, i64_action);

impl_action_ext_for!(u8, u8_action);

impl_action_ext_for!(u16, u16_action);

impl_action_ext_for!(u32, u32_action);

impl_action_ext_for!(u64, u64_action);

impl_action_ext_for!(bool, bool_action);

impl_action_ext_for!(usize, usize_action);

impl_action_ext_for!(isize, isize_action);

impl_action_ext_for!(String, str_action);
