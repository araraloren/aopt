use async_trait::async_trait;
use std::fmt::Debug;

use crate::err::Result;
use crate::opt::Opt;
use crate::set::Set;
use crate::uid::Uid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallbackType {
    Index, 

    IndexMut,

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
    pub fn is_index(&self) -> bool {
        match self {
            Self::Index => true,
            _ => false,
        }
    }

    pub fn is_index_mut(&self) -> bool {
        match self {
            Self::IndexMut => true,
            _ => false,
        }
    }

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

#[async_trait::async_trait(?Send)]
pub trait ValueCallback: Debug {
    #[cfg(not(feature="async"))]
    fn call(&mut self, uid: Uid, set: &dyn Set) -> Result<bool>;

    #[cfg(feature="async")]
    async fn call(&mut self, uid: Uid, set: &dyn Set) -> Result<bool>;
}

#[async_trait::async_trait(?Send)]
pub trait ValueMutCallback: Debug {
    #[cfg(not(feature="async"))]
    fn call(&mut self, uid: Uid, set: &mut dyn Set) -> Result<bool>;

    #[cfg(feature="async")]
    async fn call(&mut self, uid: Uid, set: &mut dyn Set) -> Result<bool>;
}

#[async_trait::async_trait(?Send)]
pub trait IndexCallback: Debug {
    #[cfg(not(feature="async"))]
    fn call(&mut self, uid: Uid, set: &dyn Set, arg: &String) -> Result<bool>;

    #[cfg(feature="async")]
    async fn call(&mut self, uid: Uid, set: &dyn Set, arg: &String) -> Result<bool>;
}

#[async_trait::async_trait(?Send)]
pub trait IndexMutCallback: Debug {
    #[cfg(not(feature="async"))]
    fn call(&mut self, uid: Uid, set: &mut dyn Set, arg: &String) -> Result<bool>;

    #[cfg(feature="async")]
    async fn call(&mut self, uid: Uid, set: &mut dyn Set, arg: &String) -> Result<bool>;
}

#[async_trait::async_trait(?Send)]
pub trait MainCallback: Debug {
    #[cfg(not(feature="async"))]
    fn call(&mut self, uid: Uid, set: &dyn Set, args: &Vec<String>) -> Result<bool>;

    #[cfg(feature="async")]
    async fn call(&mut self, uid: Uid, set: &dyn Set, args: &Vec<String>) -> Result<bool>;
}

#[async_trait::async_trait(?Send)]
pub trait MainMutCallback: Debug {
    #[cfg(not(feature="async"))]
    fn call(&mut self, uid: Uid, set: &mut dyn Set, args: &Vec<String>) -> Result<bool>;

    #[cfg(feature="async")]
    async fn call(&mut self, uid: Uid, set: &mut dyn Set, args: &Vec<String>) -> Result<bool>;
}

#[derive(Debug)]
pub enum Callback {
    Value(Box<dyn ValueCallback>),

    ValueMut(Box<dyn ValueMutCallback>),

    Index(Box<dyn IndexCallback>),

    IndexMut(Box<dyn IndexMutCallback>),

    Main(Box<dyn MainCallback>),

    MainMut(Box<dyn MainMutCallback>),

    Null,
}

impl Default for Callback {
    fn default() -> Self {
        Self::Null
    }
}

impl Callback {
    pub fn match_callback(&self, callback_type: CallbackType) -> bool {
        match self {
            Callback::Value(_) => {
                callback_type == CallbackType::Value
            },
            Callback::ValueMut(_) => {
                callback_type == CallbackType::ValueMut
            },
            Callback::Index(_) => {
                callback_type == CallbackType::Index
            },
            Callback::IndexMut(_) => {
                callback_type == CallbackType::IndexMut
            },
            Callback::Main(_) =>  {
                callback_type == CallbackType::Main
            },
            Callback::MainMut(_) => {
                callback_type == CallbackType::MainMut
            },
            Callback::Null => {
                false
            },
        }
    }
}

impl From<Box<dyn ValueCallback>> for Callback {
    fn from(cb: Box<dyn ValueCallback>) -> Self {
        Callback::Value(cb)
    }
}

impl From<Box<dyn ValueMutCallback>> for Callback {
    fn from(cb: Box<dyn ValueMutCallback>) -> Self {
        Callback::ValueMut(cb)
    }
}

impl From<Box<dyn IndexCallback>> for Callback {
    fn from(cb: Box<dyn IndexCallback>) -> Self {
        Callback::Index(cb)
    }
}

impl From<Box<dyn IndexMutCallback>> for Callback {
    fn from(cb: Box<dyn IndexMutCallback>) -> Self {
        Callback::IndexMut(cb)
    }
}

impl From<Box<dyn MainCallback>> for Callback {
    fn from(cb: Box<dyn MainCallback>) -> Self {
        Callback::Main(cb)
    }
}

impl From<Box<dyn MainMutCallback>> for Callback {
    fn from(cb: Box<dyn MainMutCallback>) -> Self {
        Callback::MainMut(cb)
    }
}