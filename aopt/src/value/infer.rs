use std::any::TypeId;
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
use crate::prelude::SetValueFindExt;
use crate::trace_log;
use crate::typeid;
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

    fn infer_type_id() -> TypeId {
        typeid::<Self::Val>()
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
        let type_id = Self::infer_type_id();
        let initializer = Self::infer_initializer();
        let storer = if let Some(validator) = Self::infer_validator() {
            Some(ValStorer::from(validator))
        } else {
            Some(ValStorer::fallback::<Self::Val>())
        };

        Self::infer_tweak_info(cfg);
        (!cfg.has_ctor()).then(|| cfg.set_ctor(ctor));
        (!cfg.has_index()).then(|| index.map(|idx| cfg.set_index(idx)));
        (!cfg.has_type()).then(|| cfg.set_type_id(type_id));
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

pub trait InferValueRef<'a> {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error>
    where
        Self: Sized;
}

pub trait InferValueMut<'a> {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized;
}

macro_rules! impl_for_bool {
    () => {
        fn infer_act() -> Action {
            Action::Set
        }

        fn infer_style() -> Vec<Style> {
            vec![Style::Combined, Style::Boolean]
        }
    };
}

impl Infer for bool {
    type Val = bool;

    impl_for_bool!();

    /// bool has a default value [`false`]
    fn infer_initializer() -> Option<ValInitializer> {
        Some(ValInitializer::new_value(false))
    }
}

impl<'a> InferValueMut<'a> for bool {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        set.take_val::<bool>(name)
    }
}

impl Infer for Option<bool> {
    type Val = bool;

    impl_for_bool!();
}

impl<'a> InferValueMut<'a> for Option<bool> {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(set.take_val::<bool>(name).ok())
    }
}

impl Infer for Vec<bool> {
    type Val = bool;

    impl_for_bool!();
}

impl<'a> InferValueMut<'a> for Vec<bool> {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(std::mem::take(set.find_vals_mut::<bool>(name)?))
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

    fn infer_type_id() -> TypeId {
        typeid::<Self>()
    }
}

impl<'a> InferValueMut<'a> for Cmd {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Cmd::new(set.take_val::<bool>(name)?))
    }
}

macro_rules! impl_pos_type {
    () => {
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
    };
}

impl<T> Infer for Pos<T>
where
    T: Infer + ErasedTy,
{
    type Val = T::Val;

    fn infer_force() -> bool {
        true
    }

    fn infer_type_id() -> TypeId {
        typeid::<Self>()
    }

    impl_pos_type!();
}

impl<'a, T> InferValueMut<'a> for Pos<T>
where
    T: ErasedTy + InferValueMut<'a>,
{
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Pos::new(<T as InferValueMut>::infer_fetch(name, set)?))
    }
}

impl<'a, T> InferValueRef<'a> for Pos<T>
where
    T: ErasedTy + InferValueRef<'a>,
{
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Pos::new(<T as InferValueRef>::infer_fetch(name, set)?))
    }
}

impl<T> Infer for Option<Pos<T>>
where
    T: Infer + ErasedTy,
{
    type Val = T::Val;

    fn infer_type_id() -> TypeId {
        typeid::<Pos<T>>()
    }

    impl_pos_type!();
}

impl<'a, T> InferValueMut<'a> for Option<Pos<T>>
where
    T: InferValueMut<'a> + ErasedTy,
{
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(<T as InferValueMut>::infer_fetch(name, set)
            .ok()
            .map(|v| Pos::new(v)))
    }
}

impl<'a, T> InferValueRef<'a> for Option<Pos<T>>
where
    T: InferValueRef<'a> + ErasedTy,
{
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(<T as InferValueRef>::infer_fetch(name, set)
            .ok()
            .map(|v| Pos::new(v)))
    }
}

impl<T> Infer for Main<T>
where
    T: Infer + ErasedTy,
{
    type Val = T::Val;

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

    fn infer_type_id() -> TypeId {
        typeid::<Self>()
    }
}

impl<'a, T> InferValueMut<'a> for Main<T>
where
    T: InferValueMut<'a> + ErasedTy,
{
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Main::new(<T as InferValueMut>::infer_fetch(name, set)?))
    }
}

impl<'a, T> InferValueRef<'a> for Main<T>
where
    T: InferValueRef<'a> + ErasedTy,
{
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Main::new(<T as InferValueRef>::infer_fetch(name, set)?))
    }
}

impl<T> Infer for Any<T>
where
    T: Infer + ErasedTy,
{
    type Val = T::Val;

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

    fn infer_type_id() -> TypeId {
        typeid::<Self>()
    }
}

impl<'a, T> InferValueMut<'a> for Any<T>
where
    T: InferValueMut<'a> + ErasedTy,
{
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Any::new(<T as InferValueMut>::infer_fetch(name, set)?))
    }
}

impl<'a, T> InferValueRef<'a> for Any<T>
where
    T: InferValueRef<'a> + ErasedTy,
{
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Any::new(<T as InferValueRef>::infer_fetch(name, set)?))
    }
}

macro_rules! impl_for_stdin {
    () => {
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
    };
}

impl Infer for Stdin {
    type Val = Stdin;

    impl_for_stdin!();
}

impl<'a> InferValueMut<'a> for Stdin {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(set.take_val::<Stdin>(name)?)
    }
}

impl Infer for Option<Stdin> {
    type Val = Stdin;

    impl_for_stdin!();
}

impl<'a> InferValueMut<'a> for Option<Stdin> {
    fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(set.take_val::<Stdin>(name).ok())
    }
}

macro_rules! impl_infer_for {
    ($name:path) => {
        impl Infer for $name {
            type Val = $name;

            fn infer_force() -> bool {
                true
            }
        }

        impl<'a> InferValueMut<'a> for $name {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error> where Self: Sized {
                set.take_val::<$name>(name)
            }
        }

        impl Infer for std::option::Option<$name> {
            type Val = $name;
        }

        impl<'a> InferValueMut<'a> for std::option::Option<$name> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(set.take_val::<$name>(name).ok())
            }
        }

        impl Infer for std::vec::Vec<$name> {
            type Val = $name;

            fn infer_force() -> bool {
                true
            }
        }

        impl<'a> InferValueMut<'a> for std::vec::Vec<$name> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(std::mem::take(set.find_vals_mut::<$name>(name)?))
            }
        }

        impl Infer for std::option::Option<std::vec::Vec<$name>> {
            type Val = $name;
        }

        impl<'a> InferValueMut<'a> for std::option::Option<std::vec::Vec<$name>> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(set.find_vals_mut::<$name>(name).ok().map(|v|std::mem::take(v)))
            }
        }
    };
    (&$a:lifetime $name:path) => {
        impl<$a> Infer for &$a $name {
            type Val = $name;

            fn infer_force() -> bool {
                true
            }
        }

        impl<$a> InferValueRef<$a> for &$a $name {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error> where Self: Sized {
                set.find_val::<$name>(name)
            }
        }

        impl<$a> Infer for std::option::Option<&$a $name> {
            type Val = $name;
        }

        impl<$a> InferValueRef<$a> for std::option::Option<&$a $name> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(set.find_val::<$name>(name).ok())
            }
        }

        impl<$a> Infer for &$a std::vec::Vec<$name> {
            type Val = $name;

            fn infer_force() -> bool {
                true
            }
        }

        impl<$a> InferValueRef<$a> for &$a std::vec::Vec<$name> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error> where Self: Sized {
                set.find_vals::<$name>(name)
            }
        }

        impl<$a> Infer for Option<&$a std::vec::Vec<$name>> {
            type Val = $name;
        }

        impl<$a> InferValueRef<$a> for Option<&$a std::vec::Vec<$name>> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(set.find_vals::<$name>(name).ok())
            }
        }
    };
    (&$a:lifetime $name:path, $inner_type:path) => {
        impl<$a> Infer for &$a $name {
            type Val = $inner_type;

            fn infer_force() -> bool {
                true
            }
        }

        impl<$a> InferValueRef<$a> for &$a $name {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error> where Self: Sized {
                set.find_val::<$inner_type>(name).map(|v| v.as_ref())
            }
        }

        impl<$a> Infer for std::option::Option<&$a $name> {
            type Val = $inner_type;
        }

        impl<$a> InferValueRef<$a> for std::option::Option<&$a $name> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(set.find_val::<$inner_type>(name).map(|v| v.as_ref()).ok())
            }
        }

        impl<$a> Infer for std::vec::Vec<&$a $name> {
            type Val = $inner_type;

            fn infer_force() -> bool {
                true
            }
        }

        impl<$a> InferValueRef<$a> for std::vec::Vec<&$a $name> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(set.find_vals::<$inner_type>(name)?.iter().map(|v|v.as_ref()).collect())
            }
        }

        impl<$a> Infer for Option<std::vec::Vec<&$a $name>> {
            type Val = $inner_type;
        }

        impl<$a> InferValueRef<$a> for Option<std::vec::Vec<&$a $name>> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(Some(set.find_vals::<$inner_type>(name)?.iter().map(|v|v.as_ref()).collect()))
            }
        }
    };
    ($name:path, $force:literal { type Val = $val_type:ty; $( fn $fn_name:ident() -> $ret_type:ty $fn_block:block )+ }) => {
        impl Infer for $name {
            type Val = $val_type;

            $(
                fn $fn_name() -> $ret_type $fn_block
            )+

            fn infer_force() -> bool {
                $force
            }
        }

        impl<'a> InferValueMut<'a> for $name {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error> where Self: Sized {
                set.take_val::<$name>(name)
            }
        }

        impl Infer for std::option::Option<$name> {
            type Val = $val_type;

            $(
                fn $fn_name() -> $ret_type $fn_block
            )+
        }

        impl<'a> InferValueMut<'a> for std::option::Option<$name> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(set.take_val::<$name>(name).ok())
            }
        }

        impl Infer for std::vec::Vec<$name> {
            type Val = $val_type;

            $(
                fn $fn_name() -> $ret_type $fn_block
            )+

            fn infer_force() -> bool {
                $force
            }
        }

        impl<'a> InferValueMut<'a> for std::vec::Vec<$name> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(std::mem::take(set.find_vals_mut::<$name>(name)?))
            }
        }

        impl Infer for std::option::Option<std::vec::Vec<$name>> {
            type Val = $val_type;

            $(
                fn $fn_name() -> $ret_type $fn_block
            )+
        }

        impl<'a> InferValueMut<'a> for std::option::Option<std::vec::Vec<$name>> {
            fn infer_fetch<S: SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, crate::Error> where Self: Sized {
                Ok(set.find_vals_mut::<$name>(name).ok().map(|v|std::mem::take(v)))
            }
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
impl_infer_for!(&'a std::path::Path, PathBuf);
impl_infer_for!(&'a str, String);
impl_infer_for!(&'a OsStr, OsString);

#[derive(Debug, Clone, Copy)]
pub struct Placeholder;

impl Infer for Placeholder {
    type Val = ();

    fn infer_type_id() -> TypeId {
        typeid::<Self>()
    }
}

impl<'a> InferValueMut<'a> for Placeholder {
    fn infer_fetch<S: SetValueFindExt>(_: &str, _: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(Placeholder {})
    }
}

impl Infer for () {
    type Val = ();
}

impl<'a> InferValueMut<'a> for () {
    fn infer_fetch<S: SetValueFindExt>(_: &str, _: &'a mut S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl<'a> InferValueRef<'a> for () {
    fn infer_fetch<S: SetValueFindExt>(_: &str, _: &'a S) -> Result<Self, crate::Error>
    where
        Self: Sized,
    {
        Ok(())
    }
}
