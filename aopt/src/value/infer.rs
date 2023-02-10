use std::io::Stdin;
use std::path::PathBuf;

use crate::map::ErasedTy;
use crate::opt::Action;
use crate::opt::Any;
use crate::opt::Cmd;
use crate::opt::Index;
use crate::opt::Main;
use crate::opt::Noa;
use crate::opt::Pos;
use crate::opt::Style;
use crate::value::ValInitializer;
use crate::value::ValValidator;
use crate::Str;

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
    type Val = Noa;

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
        Some(ValInitializer::new_value(Noa::new(false)))
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
}

impl Infer for Main {
    type Val = ();

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

impl Infer for Any {
    type Val = ();

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

impl<'a> Infer for &'a str {
    type Val = String;
}

impl Infer for std::path::Path {
    type Val = PathBuf;
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
