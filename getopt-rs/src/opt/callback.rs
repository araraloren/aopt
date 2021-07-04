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

    pub fn is_cmd(&self) -> bool {
        match self {
            Self::Pos => true,
            _ => false,
        }
    }

    pub fn is_cmd_mut(&self) -> bool {
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

#[async_trait::async_trait(?Send)]
pub trait CmdCallback: Debug {
    #[cfg(not(feature = "async"))]
    fn call(&mut self, uid: Uid, set: &dyn Set) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn call(&mut self, uid: Uid, set: &dyn Set) -> Result<Option<OptValue>>;
}

#[async_trait::async_trait(?Send)]
pub trait CmdMutCallback: Debug {
    #[cfg(not(feature = "async"))]
    fn call(&mut self, uid: Uid, set: &mut dyn Set) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn call(&mut self, uid: Uid, set: &mut dyn Set) -> Result<Option<OptValue>>;
}

#[async_trait::async_trait(?Send)]
pub trait OptCallback: Debug {
    #[cfg(not(feature = "async"))]
    fn call(&mut self, uid: Uid, set: &dyn Set, arg: &String) -> Result<Option<OptValue>>;

    #[cfg(feature = "async")]
    async fn call(&mut self, uid: Uid, set: &dyn Set, arg: &String) -> Result<Option<OptValue>>;
}

#[async_trait::async_trait(?Send)]
pub trait OptMutCallback: Debug {
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
    Cmd(Box<dyn CmdCallback>),

    CmdMut(Box<dyn CmdMutCallback>),

    Opt(Box<dyn OptCallback>),

    OptMut(Box<dyn OptMutCallback>),

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
            Callback::Cmd(_) => callback_type == CallbackType::Pos,
            Callback::CmdMut(_) => callback_type == CallbackType::PosMut,
            Callback::Opt(_) => callback_type == CallbackType::Opt,
            Callback::OptMut(_) => callback_type == CallbackType::OptMut,
            Callback::Main(_) => callback_type == CallbackType::Main,
            Callback::MainMut(_) => callback_type == CallbackType::MainMut,
            Callback::Null => false,
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn call(&mut self, uid: Uid, set: &dyn Set, args: &[String]) -> Result<Option<OptValue>> {
        match self {
            Callback::Cmd(v) => v.as_mut().call(uid, set),
            Callback::Opt(v) => v.as_mut().call(uid, set, &args[0]),
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
            Callback::CmdMut(v) => v.as_mut().call(uid, set),
            Callback::OptMut(v) => v.as_mut().call(uid, set, &args[0]),
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

impl From<Box<dyn CmdCallback>> for Callback {
    fn from(cb: Box<dyn CmdCallback>) -> Self {
        Callback::Cmd(cb)
    }
}

impl From<Box<dyn CmdMutCallback>> for Callback {
    fn from(cb: Box<dyn CmdMutCallback>) -> Self {
        Callback::CmdMut(cb)
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
pub struct SimpleCmdCallback<T: 'static + FnMut(Uid, &dyn Set) -> Result<Option<OptValue>>>(T);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set) -> Result<Option<OptValue>>> SimpleCmdCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set) -> Result<Option<OptValue>>> Debug for SimpleCmdCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleCmdCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set) -> Result<Option<OptValue>>> CmdCallback
    for SimpleCmdCallback<T>
{
    fn call(&mut self, uid: Uid, set: &dyn Set) -> Result<Option<OptValue>> {
        self.0(uid, set)
    }
}

#[cfg(not(feature = "async"))]
pub struct SimpleCmdMutCallback<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<Option<OptValue>>>(
    T,
);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<Option<OptValue>>> SimpleCmdMutCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<Option<OptValue>>> Debug
    for SimpleCmdMutCallback<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleCmdMutCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set) -> Result<Option<OptValue>>> CmdMutCallback
    for SimpleCmdMutCallback<T>
{
    fn call(&mut self, uid: Uid, set: &mut dyn Set) -> Result<Option<OptValue>> {
        self.0(uid, set)
    }
}

#[cfg(not(feature = "async"))]
pub struct SimpleOptCallback<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<Option<OptValue>>>(
    T,
);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<Option<OptValue>>> SimpleOptCallback<T> {
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<Option<OptValue>>> Debug
    for SimpleOptCallback<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOptCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &dyn Set, &String) -> Result<Option<OptValue>>> OptCallback
    for SimpleOptCallback<T>
{
    fn call(&mut self, uid: Uid, set: &dyn Set, arg: &String) -> Result<Option<OptValue>> {
        self.0(uid, set, arg)
    }
}

#[cfg(not(feature = "async"))]
pub struct SimpleOptMutCallback<
    T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<Option<OptValue>>,
>(T);

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<Option<OptValue>>>
    SimpleOptMutCallback<T>
{
    pub fn new(cb: T) -> Self {
        Self(cb)
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<Option<OptValue>>> Debug
    for SimpleOptMutCallback<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOptMutCallback")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

#[cfg(not(feature = "async"))]
impl<T: 'static + FnMut(Uid, &mut dyn Set, &String) -> Result<Option<OptValue>>> OptMutCallback
    for SimpleOptMutCallback<T>
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
