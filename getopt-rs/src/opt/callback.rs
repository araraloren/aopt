use async_trait::async_trait;
use std::fmt::Debug;

use crate::arg;
use crate::err::{Error, Result};
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
    fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<bool>;

    #[cfg(feature="async")]
    async fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<bool>;
}

#[async_trait::async_trait(?Send)]
pub trait MainMutCallback: Debug {
    #[cfg(not(feature="async"))]
    fn call(&mut self, uid: Uid, set: &mut dyn Set, args: &[String]) -> Result<bool>;

    #[cfg(feature="async")]
    async fn call(&mut self, uid: Uid, set: &mut dyn Set, args: &[String]) -> Result<bool>;
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

    #[cfg(not(feature="async"))]
    pub fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<bool> {
        match self {
            Callback::Value(v) => {
                v.as_mut()
                 .call(uid, set)
            },
            Callback::Index(v) => {
                v.as_mut()
                 .call(uid, set, &args[0])
            },
            Callback::Main(v) =>  {
                v.as_mut()
                 .call(uid, set, args)
            },
            other => {
                Err(Error::InvalidCallbackType(format!("{:?}", other)))
            },
        }
    }

    #[cfg(not(feature="async"))]
    pub fn call_mut(&mut self, uid: Uid, set: &mut dyn Set, args: &[String]) -> Result<bool> {
        match self {
            Callback::ValueMut(v) => {
                v.as_mut()
                 .call(uid, set)
            },
            Callback::IndexMut(v) => {
                v.as_mut()
                 .call(uid, set, &args[0])
            },
            Callback::MainMut(v) =>  {
                v.as_mut()
                 .call(uid, set, args)
            },
            other => {
                Err(Error::InvalidCallbackType(format!("{:?}", other)))
            },
        }
    }

    #[cfg(feature="async")]
    pub async fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<bool> {
        match self {
            Callback::Value(v) => {
                v.as_mut()
                 .call(uid, set)
                 .await
            },
            Callback::Index(v) => {
                v.as_mut()
                 .call(uid, set, &args[0])
                 .await
            },
            Callback::Main(v) =>  {
                v.as_mut()
                 .call(uid, set, args)
                 .await
            },
            other => {
                Err(Error::InvalidCallbackType(format!("{:?}", other)))
            },
        }
    }

    #[cfg(feature="async")]
    pub async fn call_mut(&mut self, uid: Uid, set: &mut dyn Set, args: &[String]) -> Result<bool> {
        match self {
            Callback::ValueMut(v) => {
                v.as_mut()
                 .call(uid, set)
                 .await
            },
            Callback::IndexMut(v) => {
                v.as_mut()
                 .call(uid, set, &args[0])
                 .await
            },
            Callback::MainMut(v) =>  {
                v.as_mut()
                 .call(uid, set, args)
                 .await
            },
            other => {
                Err(Error::InvalidCallbackType(format!("{:?}", other)))
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


#[cfg(not(feature="async"))]
pub struct SimpleValueCallback<T: 'static + FnMut(Uid, &dyn Set) -> Result<bool>>(T);

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &dyn Set) -> Result<bool>> SimpleValueCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &dyn Set) -> Result<bool>> Debug for SimpleValueCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleValueCallback")
         .field("FnMut", &String::from("..."))
         .finish()
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &dyn Set) -> Result<bool>> ValueCallback for SimpleValueCallback<T> {
    fn call(&mut self, uid: Uid, set: &dyn Set) -> Result<bool> {
        self.0(uid, set)
    }
}

#[cfg(not(feature="async"))]
pub struct SimpleValueMutCallback<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<bool>>(T);

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<bool>> SimpleValueMutCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<bool>> Debug for SimpleValueMutCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleValueMutCallback")
         .field("FnMut", &String::from("..."))
         .finish()
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<bool>> ValueMutCallback for SimpleValueMutCallback<T> {
    fn call(&mut self, uid: Uid, set: &mut dyn Set) -> Result<bool> {
        self.0(uid, set)
    }
}

#[cfg(not(feature="async"))]
pub struct SimpleIndexCallback<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<bool>>(T);

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<bool>> SimpleIndexCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<bool>> Debug for SimpleIndexCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleIndexCallback")
         .field("FnMut", &String::from("..."))
         .finish()
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<bool>> IndexCallback for SimpleIndexCallback<T> {
    fn call(&mut self, uid: Uid, set: &dyn Set, arg: &String) -> Result<bool> {
        self.0(uid, set, arg)
    }
}

#[cfg(not(feature="async"))]
pub struct SimpleIndexMutCallback<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<bool>>(T);

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<bool>> SimpleIndexMutCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<bool>> Debug for SimpleIndexMutCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleIndexMutCallback")
         .field("FnMut", &String::from("..."))
         .finish()
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<bool>> IndexMutCallback for SimpleIndexMutCallback<T> {
    fn call(&mut self, uid: Uid, set: &mut dyn Set, arg: &String) -> Result<bool> {
        self.0(uid, set, arg)
    }
}

#[cfg(not(feature="async"))]
pub struct SimpleMainCallback<T: 'static + FnMut(Uid, &dyn Set, &[String]) -> Result<bool>>(T);

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &[String]) -> Result<bool>> SimpleMainCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &[String]) -> Result<bool>> Debug for SimpleMainCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleIndexMutCallback")
         .field("FnMut", &String::from("..."))
         .finish()
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &[String]) -> Result<bool>> MainCallback for SimpleMainCallback<T> {
    fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<bool> {
        self.0(uid, set, args)
    }
}

#[cfg(not(feature="async"))]
pub struct SimpleMainMutCallback<T: 'static + FnMut(Uid, &mut dyn Set, &[String]) -> Result<bool>>(T);

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &[String]) -> Result<bool>> SimpleMainMutCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &[String]) -> Result<bool>> Debug for SimpleMainMutCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleIndexMutCallback")
         .field("FnMut", &String::from("..."))
         .finish()
    }
}

#[cfg(not(feature="async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &[String]) -> Result<bool>> MainMutCallback for SimpleMainMutCallback<T> {
    fn call(&mut self, uid: Uid, set: &mut dyn Set, args: &[String]) -> Result<bool> {
        self.0(uid, set, args)
    }
}