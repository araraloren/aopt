use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Stdin;
use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::map::ErasedTy;
use crate::opt::Action;
use crate::opt::Any;
use crate::opt::BuiltInCtor;
use crate::opt::Cmd;
use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::opt::Main;
use crate::opt::Pos;
use crate::opt::Style;
use crate::trace_log;
use crate::value::ValInitializer;
use crate::value::ValValidator;
use crate::RawVal;
use crate::Str;

use super::AnyValue;
use super::RawValParser;
use super::ValStorer;

/// Implement this if you want the type can used for create option.
pub trait Infer {
    type Val: ErasedTy + 'static;

    fn infer_act() -> Action {
        Action::App
    }

    fn infer_force() -> bool {
        false
    }

    fn infer_ctor() -> Str {
        crate::set::ctor_default_name()
    }

    fn infer_index() -> Option<Index> {
        None
    }

    fn infer_style() -> Vec<Style> {
        vec![Style::Argument]
    }

    fn infer_ignore_name() -> bool {
        false
    }

    fn infer_ignore_alias() -> bool {
        false
    }

    fn infer_ignore_index() -> bool {
        true
    }

    fn infer_validator() -> Option<ValValidator<Self::Val>> {
        None
    }

    fn infer_initializer() -> Option<ValInitializer> {
        Some(ValInitializer::fallback())
    }

    fn infer_tweak_info<C>(_cfg: &mut C)
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
    }

    fn infer_fill_info<C>(cfg: &mut C, ignore_infer: bool)
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        let act = Self::infer_act();
        let style = Self::infer_style();
        let index = Self::infer_index();
        let ignore_name = Self::infer_ignore_name();
        let ignore_alias = Self::infer_ignore_alias();
        let ignore_index = Self::infer_ignore_index();
        let force = Self::infer_force();
        let ctor = Self::infer_ctor();
        let initializer = Self::infer_initializer();
        let storer = if let Some(validator) = Self::infer_validator() {
            Some(ValStorer::from(validator))
        } else {
            Some(ValStorer::fallback::<Self::Val>())
        };

        Self::infer_tweak_info(cfg);
        (!cfg.has_ctor()).then(|| cfg.set_ctor(ctor));
        (!cfg.has_index()).then(|| index.map(|idx| cfg.set_index(idx)));
        (!cfg.has_type()).then(|| cfg.set_type::<Self>());
        (!cfg.has_action()).then(|| cfg.set_action(act));
        (!cfg.has_style()).then(|| cfg.set_style(style));
        (!cfg.has_force()).then(|| cfg.set_force(force));
        (!cfg.has_action()).then(|| cfg.set_action(act));
        if ignore_infer || !cfg.has_infer() {
            cfg.set_infer(true);
            if let Some(storer) = storer {
                (!cfg.has_storer()).then(|| cfg.set_storer(storer));
            }
            if let Some(initializer) = initializer {
                (!cfg.has_initializer()).then(|| cfg.set_initializer(initializer));
            }
        }
        cfg.set_ignore_name(ignore_name);
        cfg.set_ignore_alias(ignore_alias);
        cfg.set_ignore_index(ignore_index);
    }
}

impl Infer for bool {
    type Val = bool;

    fn infer_act() -> Action {
        Action::Set
    }

    fn infer_style() -> Vec<Style> {
        vec![Style::Combined, Style::Boolean]
    }

    fn infer_initializer() -> Option<ValInitializer> {
        Some(ValInitializer::new_value(false))
    }
}

impl Infer for Cmd {
    type Val = bool;

    fn infer_act() -> Action {
        Action::Set
    }

    fn infer_force() -> bool {
        true
    }

    fn infer_index() -> Option<Index> {
        Some(Index::forward(1))
    }

    fn infer_style() -> Vec<Style> {
        vec![Style::Cmd]
    }

    fn infer_ignore_index() -> bool {
        false
    }

    fn infer_initializer() -> Option<ValInitializer> {
        Some(ValInitializer::new_value(false))
    }
}

impl<T> Infer for Pos<T>
where
    T: ErasedTy + 'static,
{
    type Val = T;

    fn infer_style() -> Vec<Style> {
        vec![Style::Pos]
    }

    fn infer_ignore_name() -> bool {
        true
    }

    fn infer_ignore_alias() -> bool {
        true
    }

    fn infer_ignore_index() -> bool {
        false
    }

    /// Will add default type storer when value type is bool.
    ///
    /// # Storer
    /// ```!
    /// Box::new(
    ///     |raw: Option<&RawVal>, _: &Ctx, act: &Action, handler: &mut AnyValue| {
    ///         let val = raw.is_some();
    ///
    ///         trace_log!("Cmd value storer, parsing {:?} -> {:?}", raw, val);
    ///         act.store1(Some(val), handler);
    ///         Ok(())
    ///     },
    /// );
    /// ```
    fn infer_tweak_info<C>(cfg: &mut C)
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        if !cfg.has_storer() {
            let type_id = cfg.r#type();
            let ctor = cfg.ctor().map(|v| BuiltInCtor::from_name(v));
            let bool_type = std::any::TypeId::of::<bool>();

            // add default storer when value type is bool.
            if type_id == Some(&bool_type) || ctor == Some(BuiltInCtor::Bool) {
                cfg.set_storer(ValStorer::new(Box::new(
                    |raw: Option<&RawVal>, _: &Ctx, act: &Action, handler: &mut AnyValue| {
                        let val = raw.is_some();

                        trace_log!("Cmd value storer, parsing {:?} -> {:?}", raw, val);
                        act.store1(Some(val), handler);
                        Ok(())
                    },
                )));
            }
        }
    }
}

impl<T> Infer for Main<T>
where
    T: ErasedTy + 'static,
{
    type Val = T;

    fn infer_act() -> Action {
        Action::Null
    }

    fn infer_index() -> Option<Index> {
        Some(Index::anywhere())
    }

    fn infer_style() -> Vec<Style> {
        vec![Style::Main]
    }

    fn infer_ignore_name() -> bool {
        true
    }

    fn infer_ignore_alias() -> bool {
        true
    }

    fn infer_ignore_index() -> bool {
        false
    }
}

impl<T> Infer for Any<T>
where
    T: ErasedTy + 'static,
{
    type Val = T;

    fn infer_act() -> Action {
        Action::Null
    }

    fn infer_style() -> Vec<Style> {
        vec![
            Style::Argument,
            Style::Boolean,
            Style::Combined,
            Style::Pos,
            Style::Cmd,
            Style::Main,
        ]
    }

    fn infer_ignore_index() -> bool {
        false
    }
}

macro_rules! impl_infer_for {
    ($name:ident) => {
        impl Infer for $name {
            type Val = $name;
        }

        impl Infer for std::option::Option<$name> {
            type Val = $name;
        }

        impl Infer for std::vec::Vec<$name> {
            type Val = $name;
        }
    };
    (&$a:lifetime $name:ident) => {
        impl<$a> Infer for &$a $name {
            type Val = $name;
        }

        impl<$a> Infer for std::option::Option<&$a $name> {
            type Val = $name;
        }

        impl<$a> Infer for &$a std::vec::Vec<$name> {
            type Val = $name;
        }
    };
}

impl_infer_for!(f64);
impl_infer_for!(f32);

impl_infer_for!(i8);
impl_infer_for!(i16);
impl_infer_for!(i32);
impl_infer_for!(i64);

impl_infer_for!(u8);
impl_infer_for!(u16);
impl_infer_for!(u32);
impl_infer_for!(u64);

impl_infer_for!(i128);
impl_infer_for!(u128);

impl_infer_for!(isize);
impl_infer_for!(usize);
impl_infer_for!(String);
impl_infer_for!(PathBuf);
impl_infer_for!(OsString);

impl_infer_for!(&'a f64);
impl_infer_for!(&'a f32);

impl_infer_for!(&'a i8);
impl_infer_for!(&'a i16);
impl_infer_for!(&'a i32);
impl_infer_for!(&'a i64);

impl_infer_for!(&'a u8);
impl_infer_for!(&'a u16);
impl_infer_for!(&'a u32);
impl_infer_for!(&'a u64);

impl_infer_for!(&'a i128);
impl_infer_for!(&'a u128);

impl_infer_for!(&'a isize);
impl_infer_for!(&'a usize);
impl_infer_for!(&'a String);
impl_infer_for!(&'a PathBuf);
impl_infer_for!(&'a OsString);

impl<'a> Infer for &'a str {
    type Val = String;
}

impl Infer for std::path::Path {
    type Val = PathBuf;
}

impl<'a> Infer for &'a OsStr {
    type Val = OsString;
}

impl Infer for Stdin {
    type Val = Stdin;

    fn infer_act() -> Action {
        Action::Set
    }

    fn infer_index() -> Option<Index> {
        Some(Index::anywhere())
    }

    fn infer_style() -> Vec<Style> {
        vec![Style::Pos]
    }

    fn infer_ignore_name() -> bool {
        true
    }

    fn infer_ignore_index() -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Placeholder;

impl Infer for Placeholder {
    type Val = ();
}
