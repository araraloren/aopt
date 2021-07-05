use std::fmt::Debug;

use super::OptValue;
use crate::err::{Error, Result};
use crate::set::Set;
use crate::uid::Uid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallbackType {
    Opt,

    OptMut,

    Pos,

    PosMut,

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
    pub fn is_opt(&self) -> bool {
        match self {
            Self::Opt => true,
            _ => false,
        }
    }

    pub fn is_opt_mut(&self) -> bool {
        match self {
            Self::OptMut => true,
            _ => false,
        }
    }

    pub fn is_pos(&self) -> bool {
        match self {
            Self::Pos => true,
            _ => false,
        }
    }

    pub fn is_pos_mut(&self) -> bool {
        match self {
            Self::PosMut => true,
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

impl From<Callback> for CallbackType {
    fn from(cb: Callback) -> Self {
        (&cb).into()
    }
}

impl<'a> From<&'a Callback> for CallbackType {
    fn from(cb: &'a Callback) -> Self {
        match cb {
            Callback::Opt(_) => CallbackType::Opt,
            Callback::OptMut(_) => CallbackType::OptMut,
            Callback::Pos(_) => CallbackType::Pos,
            Callback::PosMut(_) => CallbackType::PosMut,
            Callback::Main(_) => CallbackType::Main,
            Callback::MainMut(_) => CallbackType::MainMut,
            Callback::Null => CallbackType::Null,
        }
    }
}

#[async_trait::async_trait(?Send)]
pub trait OptCallback: Debug {
    #[cfg(not(feature = "async"))]
    fn call(&mut self, uid: Uid, set: &dyn Set) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn call(&mut self, uid: Uid, set: &dyn Set) -> Result<Option<OptValue>>;
}

#[async_trait::async_trait(?Send)]
pub trait OptMutCallback: Debug {
    #[cfg(not(feature = "async"))]
    fn call(&mut self, uid: Uid, set: &mut dyn Set) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn call(&mut self, uid: Uid, set: &mut dyn Set) -> Result<Option<OptValue>>;
}

#[async_trait::async_trait(?Send)]
pub trait PosCallback: Debug {
    #[cfg(not(feature = "async"))]
    fn call(&mut self, uid: Uid, set: &dyn Set, arg: &String) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn call(&mut self, uid: Uid, set: &dyn Set, arg: &String) -> Result<Option<OptValue>>;
}

#[async_trait::async_trait(?Send)]
pub trait PosMutCallback: Debug {
    #[cfg(not(feature = "async"))]
    fn call(&mut self, uid: Uid, set: &mut dyn Set, arg: &String) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn call(&mut self, uid: Uid, set: &mut dyn Set, arg: &String)
        -> Result<Option<OptValue>>;
}

#[async_trait::async_trait(?Send)]
pub trait MainCallback: Debug {
    #[cfg(not(feature = "async"))]
    fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<Option<OptValue>>;
}

#[async_trait::async_trait(?Send)]
pub trait MainMutCallback: Debug {
    #[cfg(not(feature = "async"))]
    fn call(&mut self, uid: Uid, set: &mut dyn Set, args: &[String]) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn call(
        &mut self,
        uid: Uid,
        set: &mut dyn Set,
        args: &[String],
    ) -> Result<Option<OptValue>>;
}

#[derive(Debug)]
pub enum Callback {
    Opt(Box<dyn OptCallback>),

    OptMut(Box<dyn OptMutCallback>),

    Pos(Box<dyn PosCallback>),

    PosMut(Box<dyn PosMutCallback>),

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
    pub fn is_mut(&self) -> bool {
        match self {
            Callback::Opt(_) | Callback::Pos(_) | Callback::Main(_) => false,
            Callback::OptMut(_) | Callback::PosMut(_) | Callback::MainMut(_) => true,
            Callback::Null => false,
        }
    }

    pub fn to_callback_type(&self) -> CallbackType {
        self.into()
    }

    pub fn match_callback(&self, callback_type: CallbackType) -> bool {
        match self {
            Callback::Opt(_) => callback_type == CallbackType::Pos,
            Callback::OptMut(_) => callback_type == CallbackType::PosMut,
            Callback::Pos(_) => callback_type == CallbackType::Opt,
            Callback::PosMut(_) => callback_type == CallbackType::OptMut,
            Callback::Main(_) => callback_type == CallbackType::Main,
            Callback::MainMut(_) => callback_type == CallbackType::MainMut,
            Callback::Null => false,
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<Option<OptValue>> {
        match self {
            Callback::Opt(v) => v.as_mut().call(uid, set),
            Callback::Pos(v) => v.as_mut().call(uid, set, &args[0]),
            Callback::Main(v) => v.as_mut().call(uid, set, args),
            other => Err(Error::InvalidCallbackType(format!("{:?}", other))),
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn call_mut(
        &mut self,
        uid: Uid,
        set: &mut dyn Set,
        args: &[String],
    ) -> Result<Option<OptValue>> {
        match self {
            Callback::OptMut(v) => v.as_mut().call(uid, set),
            Callback::PosMut(v) => v.as_mut().call(uid, set, &args[0]),
            Callback::MainMut(v) => v.as_mut().call(uid, set, args),
            other => Err(Error::InvalidCallbackType(format!("{:?}", other))),
        }
    }

    #[cfg(feature = "async")]
    pub async fn call(
        &mut self,
        uid: Uid,
        set: &dyn Set,
        args: &[String],
    ) -> Result<Option<OptValue>> {
        match self {
            Callback::Cmd(v) => v.as_mut().call(uid, set).await,
            Callback::Opt(v) => v.as_mut().call(uid, set, &args[0]).await,
            Callback::Main(v) => v.as_mut().call(uid, set, args).await,
            other => Err(Error::InvalidCallbackType(format!("{:?}", other))),
        }
    }

    #[cfg(feature = "async")]
    pub async fn call_mut(
        &mut self,
        uid: Uid,
        set: &mut dyn Set,
        args: &[String],
    ) -> Result<Option<OptValue>> {
        match self {
            Callback::CmdMut(v) => v.as_mut().call(uid, set).await,
            Callback::OptMut(v) => v.as_mut().call(uid, set, &args[0]).await,
            Callback::MainMut(v) => v.as_mut().call(uid, set, args).await,
            other => Err(Error::InvalidCallbackType(format!("{:?}", other))),
        }
    }
}

impl From<Box<dyn OptCallback>> for Callback {
    fn from(cb: Box<dyn OptCallback>) -> Self {
        Callback::Opt(cb)
    }
}

impl From<Box<dyn OptMutCallback>> for Callback {
    fn from(cb: Box<dyn OptMutCallback>) -> Self {
        Callback::OptMut(cb)
    }
}

impl From<Box<dyn PosCallback>> for Callback {
    fn from(cb: Box<dyn PosCallback>) -> Self {
        Callback::Pos(cb)
    }
}

impl From<Box<dyn PosMutCallback>> for Callback {
    fn from(cb: Box<dyn PosMutCallback>) -> Self {
        Callback::PosMut(cb)
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

#[cfg(not(feature = "async"))]
pub struct SimpleOptCallback<T: 'static + FnMut(Uid, &dyn Set) -> Result<Option<OptValue>>>(T);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set) -> Result<Option<OptValue>>> SimpleOptCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set) -> Result<Option<OptValue>>> Debug for SimpleOptCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOptCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set) -> Result<Option<OptValue>>> OptCallback
    for SimpleOptCallback<T>
{
    fn call(&mut self, uid: Uid, set: &dyn Set) -> Result<Option<OptValue>> {
        self.0(uid, set)
    }
}

#[cfg(not(feature = "async"))]
pub struct SimpleOptMutCallback<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<Option<OptValue>>>(
    T,
);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<Option<OptValue>>> SimpleOptMutCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<Option<OptValue>>> Debug
    for SimpleOptMutCallback<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOptMutCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<Option<OptValue>>> OptMutCallback
    for SimpleOptMutCallback<T>
{
    fn call(&mut self, uid: Uid, set: &mut dyn Set) -> Result<Option<OptValue>> {
        self.0(uid, set)
    }
}

#[cfg(not(feature = "async"))]
pub struct SimplePosCallback<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<Option<OptValue>>>(
    T,
);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<Option<OptValue>>> SimplePosCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<Option<OptValue>>> Debug
    for SimplePosCallback<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePosCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<Option<OptValue>>> PosCallback
    for SimplePosCallback<T>
{
    fn call(&mut self, uid: Uid, set: &dyn Set, arg: &String) -> Result<Option<OptValue>> {
        self.0(uid, set, arg)
    }
}

#[cfg(not(feature = "async"))]
pub struct SimplePosMutCallback<
    T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<Option<OptValue>>,
>(T);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<Option<OptValue>>>
    SimplePosMutCallback<T>
{
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<Option<OptValue>>> Debug
    for SimplePosMutCallback<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePosMutCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<Option<OptValue>>> PosMutCallback
    for SimplePosMutCallback<T>
{
    fn call(&mut self, uid: Uid, set: &mut dyn Set, arg: &String) -> Result<Option<OptValue>> {
        self.0(uid, set, arg)
    }
}

#[cfg(not(feature = "async"))]
pub struct SimpleMainCallback<
    T: 'static + FnMut(Uid, &dyn Set, &[String]) -> Result<Option<OptValue>>,
>(T);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &[String]) -> Result<Option<OptValue>>>
    SimpleMainCallback<T>
{
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &[String]) -> Result<Option<OptValue>>> Debug
    for SimpleMainCallback<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMainCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &[String]) -> Result<Option<OptValue>>> MainCallback
    for SimpleMainCallback<T>
{
    fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<Option<OptValue>> {
        self.0(uid, set, args)
    }
}

#[cfg(not(feature = "async"))]
pub struct SimpleMainMutCallback<
    T: 'static + FnMut(Uid, &mut dyn Set, &[String]) -> Result<Option<OptValue>>,
>(T);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &[String]) -> Result<Option<OptValue>>>
    SimpleMainMutCallback<T>
{
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &[String]) -> Result<Option<OptValue>>> Debug
    for SimpleMainMutCallback<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMainMutCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &[String]) -> Result<Option<OptValue>>> MainMutCallback
    for SimpleMainMutCallback<T>
{
    fn call(&mut self, uid: Uid, set: &mut dyn Set, args: &[String]) -> Result<Option<OptValue>> {
        self.0(uid, set, args)
    }
}
