use async_trait::async_trait;
use std::fmt::Debug;

use crate::err::Result;
use crate::opt::Opt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallbackType {
    Value,

    ValueMut,

    Main,

    MainMut,

    Null,
}

impl Default for CallbackType {
    fn default() -> Self {
        CallbackType::Null
    }
}

impl CallbackType {
    pub fn is_value(&self) -> bool {
        match self {
            Self::Value => true,
            _ => false,
        }
    }

    pub fn is_value_mut(&self) -> bool {
        match self {
            Self::ValueMut => true,
            _ => false,
        }
    }

    pub fn is_main(&self) -> bool {
        match self {
            Self::Main => true,
            _ => false,
        }
    }

    pub fn is_main_mut(&self) -> bool {
        match self {
            Self::MainMut => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Self::Null => true,
            _ => false,
        }
    }
}
