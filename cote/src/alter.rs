use aopt::prelude::RefOpt;

use crate::Any;
use crate::Cmd;
use crate::ConfigValue;
use crate::Main;
use crate::MutOpt;
use crate::Placeholder;
use crate::Pos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Hint {
    Opt,

    Vec,

    OptVec,

    Null,
}

pub trait Alter {
    fn alter(hint: Hint, has_force: bool, cfg: &mut impl ConfigValue) {
        match hint {
            Hint::Opt => {
                cfg.set_action(crate::Action::Set);
                if !has_force {
                    cfg.set_force(false);
                }
            }
            Hint::Vec => {
                cfg.set_action(crate::Action::App);
                if !has_force {
                    cfg.set_force(true);
                }
            }
            Hint::OptVec => {
                cfg.set_action(crate::Action::App);
                if !has_force {
                    cfg.set_force(false);
                }
            }
            Hint::Null => {
                cfg.set_action(crate::Action::Set);
                if !has_force {
                    cfg.set_force(true);
                }
            }
        }
    }
}

impl Alter for Cmd {
    fn alter(_: Hint, _: bool, cfg: &mut impl ConfigValue) {
        cfg.set_force(true);
        cfg.set_action(crate::Action::Set);
    }
}

impl Alter for bool {
    fn alter(hint: Hint, has_force: bool, cfg: &mut impl ConfigValue) {
        if !has_force {
            cfg.set_force(false);
        }
        match hint {
            Hint::Opt => {
                cfg.set_action(crate::Action::Set);
            }
            Hint::Vec => {
                cfg.set_action(crate::Action::App);
            }
            Hint::OptVec => {
                cfg.set_action(crate::Action::App);
            }
            Hint::Null => {
                cfg.set_action(crate::Action::Set);
            }
        }
    }
}

#[macro_export]
macro_rules! impl_alter {
    ($what:path) => {
        impl_alter!($what, $what);
    };
    ($what:path, $type:path) => {
        impl Alter for $type { }
    };
    (@ $wrapper:ident) => {
        impl<T> Alter for $wrapper<T> { }
    };
    (&$a:lifetime $what:path) => {
        impl<$a> Alter for &$a $what { }
    };
}

impl_alter!(Placeholder);

impl_alter!(f64);

impl_alter!(f32);

impl_alter!(u64);

impl_alter!(u32);

impl_alter!(u16);

impl_alter!(u8);

impl_alter!(i64);

impl_alter!(i32);

impl_alter!(i16);

impl_alter!(i8);

impl_alter!(i128);

impl_alter!(u128);

impl_alter!(isize);

impl_alter!(usize);

impl_alter!(String);

impl_alter!(std::path::PathBuf);

impl_alter!(std::ffi::OsString);

impl_alter!(std::io::Stdin);

impl_alter!(@ Pos);

impl_alter!(@ Main);

impl_alter!(@ Any);

impl_alter!(@ MutOpt);

impl_alter!(&'a f64);
impl_alter!(&'a f32);

impl_alter!(&'a i8);
impl_alter!(&'a i16);
impl_alter!(&'a i32);
impl_alter!(&'a i64);

impl_alter!(&'a u8);
impl_alter!(&'a u16);
impl_alter!(&'a u32);
impl_alter!(&'a u64);

impl_alter!(&'a i128);
impl_alter!(&'a u128);

impl_alter!(&'a isize);
impl_alter!(&'a usize);
impl_alter!(&'a String);
impl_alter!(&'a std::path::PathBuf);
impl_alter!(&'a std::ffi::OsString);
impl_alter!(&'a std::path::Path);
impl_alter!(&'a str);
impl_alter!(&'a std::ffi::OsStr);

impl<'a, T> Alter for RefOpt<'a, T> {
    fn alter(hint: Hint, has_force: bool, cfg: &mut impl ConfigValue) {
        match hint {
            Hint::Opt => {
                cfg.set_action(crate::Action::Set);
                if has_force {
                    cfg.set_force(false);
                }
            }
            Hint::Vec => {
                cfg.set_action(crate::Action::App);
                if has_force {
                    cfg.set_force(true);
                }
            }
            Hint::OptVec => {
                cfg.set_action(crate::Action::App);
                if has_force {
                    cfg.set_force(false);
                }
            }
            Hint::Null => {
                cfg.set_action(crate::Action::Set);
                if has_force {
                    cfg.set_force(true);
                }
            }
        }
    }
}
