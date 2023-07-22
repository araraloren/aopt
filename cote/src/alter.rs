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

/// Using for generate code for procedural macro.
///
/// Cote using [`Alter`] modify some configure value when using field
/// with `Option`, `Vec`.
pub trait Alter {
    fn alter(hint: Hint, cfg: &mut impl ConfigValue) {
        let (action, force) = match hint {
            Hint::Opt => (crate::Action::Set, false),
            Hint::Vec => (crate::Action::App, true),
            Hint::OptVec => (crate::Action::App, false),
            Hint::Null => (crate::Action::Set, true),
        };

        (!cfg.has_force()).then(|| cfg.set_force(force));
        (!cfg.has_action()).then(|| cfg.set_action(action));
    }
}

impl Alter for Cmd {
    fn alter(_: Hint, cfg: &mut impl ConfigValue) {
        (!cfg.has_force()).then(|| cfg.set_force(true));
        (!cfg.has_action()).then(|| cfg.set_action(crate::Action::Set));
    }
}

impl Alter for bool {
    fn alter(hint: Hint, cfg: &mut impl ConfigValue) {
        let action = match hint {
            Hint::Opt | Hint::Null => crate::Action::Set,
            Hint::Vec | Hint::OptVec => crate::Action::App,
        };
        (!cfg.has_force()).then(|| cfg.set_force(false));
        (!cfg.has_action()).then(|| cfg.set_action(action));
    }
}

#[macro_export]
macro_rules! impl_alter {
    ($what:path) => {
        impl Alter for $what { }
    };
    ($what:ident, $inner:ident) => {
        impl<$inner> Alter for $what<$inner> { }
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

impl_alter!(Pos, T);

impl_alter!(Main, T);

impl_alter!(Any, T);

impl_alter!(MutOpt, T);

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
    fn alter(hint: Hint, cfg: &mut impl ConfigValue) {
        let (action, force) = match hint {
            Hint::Opt => (crate::Action::Set, false),
            Hint::Vec => (crate::Action::App, true),
            Hint::OptVec => (crate::Action::App, false),
            Hint::Null => (crate::Action::Set, true),
        };

        (!cfg.has_force()).then(|| cfg.set_force(force));
        (!cfg.has_action()).then(|| cfg.set_action(action));
    }
}
