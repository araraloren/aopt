use crate::{
    prelude::ErasedTy,
    value::{RawValParser, ValValidator},
};

use super::{Action, Style};
use crate::value::ValInitializer;

pub trait Infer {
    type Val: ErasedTy + RawValParser + 'static;

    fn infer_act() -> Action {
        Action::App
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
        None
    }
}

impl Infer for i64 {
    type Val = i64;
}
