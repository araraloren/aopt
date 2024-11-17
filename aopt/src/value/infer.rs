use std::any::TypeId;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Stdin;
use std::path::PathBuf;

use crate::ctx::Ctx;
use crate::map::ErasedTy;
use crate::opt::Action;
use crate::opt::Any;
use crate::opt::Cid;
use crate::opt::Cmd;
use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::opt::Main;
use crate::opt::MutOpt;
use crate::opt::Pos;
use crate::opt::RefOpt;
use crate::opt::Style;
use crate::trace;
use crate::typeid;
use crate::value::ValInitializer;
use crate::value::ValValidator;
use crate::Error;

use super::AnyValue;
use super::RawValParser;
use super::Stop;
use super::ValStorer;

/// Implement this if you want the type can used for create option.
pub trait Infer {
    type Val: ErasedTy;

    fn infer_act() -> Action {
        Action::App
    }

    fn infer_force() -> bool {
        false
    }

    fn infer_ctor() -> String {
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

    fn infer_tweak_info<C>(_cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        Ok(())
    }

    fn infer_fill_info<C>(cfg: &mut C) -> Result<(), Error>
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

        Self::infer_tweak_info(cfg)?;
        (!cfg.has_ctor()).then(|| cfg.set_ctor(ctor));
        (!cfg.has_index()).then(|| index.map(|idx| cfg.set_index(idx)));
        (!cfg.has_type()).then(|| cfg.set_type_id(type_id));
        (!cfg.has_action()).then(|| cfg.set_action(act));
        (!cfg.has_style()).then(|| cfg.set_style(style));
        (!cfg.has_force()).then(|| cfg.set_force(force));
        (!cfg.has_action()).then(|| cfg.set_action(act));
        if let Some(storer) = storer {
            (!cfg.has_storer()).then(|| cfg.set_storer(storer));
        }
        if let Some(initializer) = initializer {
            (!cfg.has_initializer()).then(|| cfg.set_initializer(initializer));
        }
        cfg.set_ignore_name(ignore_name);
        cfg.set_ignore_alias(ignore_alias);
        cfg.set_ignore_index(ignore_index);
        Ok(())
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

    /// bool has a default value [`false`]
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

    fn infer_type_id() -> TypeId {
        typeid::<Self>()
    }
}

impl<T> Infer for Pos<T>
where
    T: Infer + ErasedTy,
{
    type Val = T::Val;

    fn infer_type_id() -> TypeId {
        typeid::<Self>()
    }

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
    /// ```ignore
    /// Box::new(
    ///     |raw: Option<&OsStr>, _: &Ctx, act: &Action, handler: &mut AnyValue| {
    ///         let val = raw.is_some();
    ///
    ///         trace!("Pos value storer, parsing {:?} -> {:?}", raw, val);
    ///         act.store1(Some(val), handler);
    ///         Ok(())
    ///     },
    /// );
    /// ```
    fn infer_tweak_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        if !cfg.has_storer() {
            let type_id = std::any::TypeId::of::<T>();
            let bool_type = std::any::TypeId::of::<bool>();

            trace!(
                "tweak the storer for Pos<bool> for {:?}?: type = {:?}",
                cfg.name(),
                std::any::type_name::<T>()
            );
            // add default storer when value type is bool.
            if type_id == bool_type {
                cfg.set_storer(ValStorer::new(Box::new(
                    |raw: Option<&OsStr>, _: &Ctx, act: &Action, handler: &mut AnyValue| {
                        let val = raw.is_some();

                        trace!("in pos<bool> value storer, parsing {:?} -> {:?}", raw, val);
                        act.store1(Some(val), handler);
                        Ok(())
                    },
                )));
            }
        }
        Ok(())
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

impl<T: ErasedTy + RawValParser> Infer for MutOpt<T> {
    type Val = T;
}

impl<'a, T: ErasedTy + RawValParser> Infer for RefOpt<'a, T> {
    type Val = T;
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

impl Infer for Stdin {
    type Val = Stdin;

    fn infer_act() -> Action {
        Action::Set
    }

    fn infer_style() -> Vec<Style> {
        vec![Style::Boolean]
    }

    fn infer_ignore_alias() -> bool {
        true
    }

    /// For type Stdin, swap the name and default alias(`-`) when build configuration.
    fn infer_tweak_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        if let Some(name) = cfg.name() {
            cfg.add_alias(name.to_string());
        }
        cfg.set_name("-");
        Ok(())
    }
}

impl Infer for Stop {
    type Val = Stop;

    fn infer_act() -> Action {
        Action::Set
    }

    fn infer_style() -> Vec<Style> {
        vec![Style::Boolean]
    }

    fn infer_ignore_alias() -> bool {
        true
    }

    /// For type Stop, swap the name and default alias(`--`) when build configuration.
    fn infer_tweak_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        if let Some(name) = cfg.name() {
            cfg.add_alias(name.to_string());
        }
        cfg.set_name("--");
        Ok(())
    }
}

macro_rules! impl_infer_for {
    ($name:path) => {
        impl Infer for $name {
            type Val = $name;
        }
    };
    (&$a:lifetime $name:path) => {
        impl<$a> Infer for &$a $name {
            type Val = $name;
        }
    };
    (&$a:lifetime $name:path, $inner_type:path) => {
        impl<$a> Infer for &$a $name {
            type Val = $inner_type;
        }
    };
    ($name:path, $force:literal { type Val = $val_type:ty; $( fn $fn_name:ident() -> $ret_type:ty $fn_block:block )+ }) => {
        impl Infer for $name {
            type Val = $val_type;

            $(
                fn $fn_name() -> $ret_type $fn_block
            )+
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

    fn infer_fill_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        // it must have ctor here
        let ctor = cfg
            .ctor()
            .ok_or_else(|| crate::raise_error!("incomplete configuration: missing `ctor`"))?;
        let cid = Cid::from(ctor);

        trace!("in default, fill info in Placeholder");
        match cid {
            Cid::Int => <i64>::infer_fill_info(cfg),
            Cid::Str => <String>::infer_fill_info(cfg),
            Cid::Flt => <f64>::infer_fill_info(cfg),
            Cid::Uint => <u64>::infer_fill_info(cfg),
            Cid::Bool => bool::infer_fill_info(cfg),
            Cid::Cmd => Cmd::infer_fill_info(cfg),
            Cid::Pos => <Pos<bool>>::infer_fill_info(cfg),
            Cid::Main => Main::<()>::infer_fill_info(cfg),
            Cid::Any => Any::<()>::infer_fill_info(cfg),
            Cid::Raw => <OsString>::infer_fill_info(cfg),
            _ => Ok(()),
        }
    }
}

impl Infer for () {
    type Val = ();
}

impl<T: Infer> Infer for Option<T> {
    type Val = <T as Infer>::Val;

    fn infer_act() -> Action {
        <T as Infer>::infer_act()
    }

    fn infer_force() -> bool {
        false
    }

    fn infer_ctor() -> String {
        <T as Infer>::infer_ctor()
    }

    fn infer_index() -> Option<Index> {
        <T as Infer>::infer_index()
    }

    fn infer_style() -> Vec<Style> {
        <T as Infer>::infer_style()
    }

    fn infer_ignore_name() -> bool {
        <T as Infer>::infer_ignore_name()
    }

    fn infer_ignore_alias() -> bool {
        <T as Infer>::infer_ignore_alias()
    }

    fn infer_ignore_index() -> bool {
        <T as Infer>::infer_ignore_index()
    }

    fn infer_validator() -> Option<ValValidator<Self::Val>> {
        <T as Infer>::infer_validator()
    }

    fn infer_initializer() -> Option<ValInitializer> {
        <T as Infer>::infer_initializer()
    }

    fn infer_type_id() -> TypeId {
        <T as Infer>::infer_type_id()
    }

    fn infer_tweak_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        <T as Infer>::infer_tweak_info(cfg)
    }

    fn infer_fill_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        <T as Infer>::infer_fill_info(cfg)
    }
}

impl<Err, T: Infer> Infer for Result<T, Err> {
    type Val = <T as Infer>::Val;

    fn infer_act() -> Action {
        <T as Infer>::infer_act()
    }

    fn infer_force() -> bool {
        false
    }

    fn infer_ctor() -> String {
        <T as Infer>::infer_ctor()
    }

    fn infer_index() -> Option<Index> {
        <T as Infer>::infer_index()
    }

    fn infer_style() -> Vec<Style> {
        <T as Infer>::infer_style()
    }

    fn infer_ignore_name() -> bool {
        <T as Infer>::infer_ignore_name()
    }

    fn infer_ignore_alias() -> bool {
        <T as Infer>::infer_ignore_alias()
    }

    fn infer_ignore_index() -> bool {
        <T as Infer>::infer_ignore_index()
    }

    fn infer_validator() -> Option<ValValidator<Self::Val>> {
        <T as Infer>::infer_validator()
    }

    fn infer_initializer() -> Option<ValInitializer> {
        <T as Infer>::infer_initializer()
    }

    fn infer_type_id() -> TypeId {
        <T as Infer>::infer_type_id()
    }

    fn infer_tweak_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        <T as Infer>::infer_tweak_info(cfg)
    }

    fn infer_fill_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        <T as Infer>::infer_fill_info(cfg)
    }
}

impl<T: Infer> Infer for Vec<T> {
    type Val = <T as Infer>::Val;

    fn infer_act() -> Action {
        Action::App
    }

    fn infer_force() -> bool {
        true
    }

    fn infer_ctor() -> String {
        <T as Infer>::infer_ctor()
    }

    fn infer_index() -> Option<Index> {
        <T as Infer>::infer_index()
    }

    fn infer_style() -> Vec<Style> {
        <T as Infer>::infer_style()
    }

    fn infer_ignore_name() -> bool {
        <T as Infer>::infer_ignore_name()
    }

    fn infer_ignore_alias() -> bool {
        <T as Infer>::infer_ignore_alias()
    }

    fn infer_ignore_index() -> bool {
        <T as Infer>::infer_ignore_index()
    }

    fn infer_validator() -> Option<ValValidator<Self::Val>> {
        <T as Infer>::infer_validator()
    }

    fn infer_initializer() -> Option<ValInitializer> {
        <T as Infer>::infer_initializer()
    }

    fn infer_type_id() -> TypeId {
        <T as Infer>::infer_type_id()
    }

    fn infer_tweak_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        <T as Infer>::infer_tweak_info(cfg)
    }

    fn infer_fill_info<C>(cfg: &mut C) -> Result<(), Error>
    where
        Self: Sized + 'static,
        Self::Val: RawValParser,
        C: ConfigValue + Default,
    {
        <T as Infer>::infer_fill_info(cfg)
    }
}
