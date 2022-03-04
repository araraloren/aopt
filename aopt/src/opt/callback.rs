use std::fmt::Debug;
use std::marker::PhantomData;

use super::OptValue;
use crate::err::Result;
use crate::set::Set;
use crate::uid::Uid;

/// The callback type of option.
///
/// Since rust has a lot of restrict on reference.
/// So we can't store block code into option itself of [`Set`](crate::set::Set).
/// Instead we put the callback code into [`Service`](crate::parser::Service).
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

impl<S: Set> From<Callback<S>> for CallbackType {
    fn from(cb: Callback<S>) -> Self {
        (&cb).into()
    }
}

impl<'a, S: Set> From<&'a Callback<S>> for CallbackType {
    fn from(cb: &'a Callback<S>) -> Self {
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

/// Callback trait using for [`Callback::Opt`], associated callback type is [`CallbackType::Opt`].
pub trait OptFn<S: Set>: Debug {
    fn call(&mut self, uid: Uid, set: &S, value: OptValue) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::OptMut`], associated callback type is [`CallbackType::OptMut`].
pub trait OptFnMut<S: Set>: Debug {
    fn call(&mut self, uid: Uid, set: &mut S, value: OptValue) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::Pos`], associated callback type is [`CallbackType::Pos`].
pub trait PosFn<S: Set>: Debug {
    fn call(
        &mut self,
        uid: Uid,
        set: &S,
        arg: &str,
        noa_index: u64,
        value: OptValue,
    ) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::PosMut`], associated callback type is [`CallbackType::PosMut`].
pub trait PosFnMut<S: Set>: Debug {
    fn call(
        &mut self,
        uid: Uid,
        set: &mut S,
        arg: &str,
        noa_index: u64,
        value: OptValue,
    ) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::Main`], associated callback type is [`CallbackType::Main`].
pub trait MainFn<S: Set>: Debug {
    fn call(
        &mut self,
        uid: Uid,
        set: &S,
        args: &[&str],
        value: OptValue,
    ) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::MainMut`], associated callback type is [`CallbackType::MainMut`].
pub trait MainFnMut<S: Set>: Debug {
    fn call(
        &mut self,
        uid: Uid,
        set: &mut S,
        args: &[&str],
        value: OptValue,
    ) -> Result<Option<OptValue>>;
}

/// The callback type hold block code.
#[derive(Debug)]
pub enum Callback<S: Set> {
    Opt(Box<dyn OptFn<S>>),

    OptMut(Box<dyn OptFnMut<S>>),

    Pos(Box<dyn PosFn<S>>),

    PosMut(Box<dyn PosFnMut<S>>),

    Main(Box<dyn MainFn<S>>),

    MainMut(Box<dyn MainFnMut<S>>),

    Null,
}

impl<S: Set> Default for Callback<S> {
    fn default() -> Self {
        Self::Null
    }
}

impl<S: Set> Callback<S> {
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
            Callback::Opt(_) => callback_type == CallbackType::Opt,
            Callback::OptMut(_) => callback_type == CallbackType::OptMut,
            Callback::Pos(_) => callback_type == CallbackType::Pos,
            Callback::PosMut(_) => callback_type == CallbackType::PosMut,
            Callback::Main(_) => callback_type == CallbackType::Main,
            Callback::MainMut(_) => callback_type == CallbackType::MainMut,
            Callback::Null => false,
        }
    }
}

impl<S: Set> From<Box<dyn OptFn<S>>> for Callback<S> {
    fn from(cb: Box<dyn OptFn<S>>) -> Self {
        Callback::Opt(cb)
    }
}

impl<S: Set> From<Box<dyn OptFnMut<S>>> for Callback<S> {
    fn from(cb: Box<dyn OptFnMut<S>>) -> Self {
        Callback::OptMut(cb)
    }
}

impl<S: Set> From<Box<dyn PosFn<S>>> for Callback<S> {
    fn from(cb: Box<dyn PosFn<S>>) -> Self {
        Callback::Pos(cb)
    }
}

impl<S: Set> From<Box<dyn PosFnMut<S>>> for Callback<S> {
    fn from(cb: Box<dyn PosFnMut<S>>) -> Self {
        Callback::PosMut(cb)
    }
}

impl<S: Set> From<Box<dyn MainFn<S>>> for Callback<S> {
    fn from(cb: Box<dyn MainFn<S>>) -> Self {
        Callback::Main(cb)
    }
}

impl<S: Set> From<Box<dyn MainFnMut<S>>> for Callback<S> {
    fn from(cb: Box<dyn MainFnMut<S>>) -> Self {
        Callback::MainMut(cb)
    }
}

/// Simple struct implemented [`OptFn`].
pub struct SimpleOptFn<S, T>(T, PhantomData<S>)
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, OptValue) -> Result<Option<OptValue>>;

impl<S, T> SimpleOptFn<S, T>
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<S, T> Debug for SimpleOptFn<S, T>
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOptFn")
            .field("Fn", &String::from("..."))
            .finish()
    }
}

impl<S, T> OptFn<S> for SimpleOptFn<S, T>
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, OptValue) -> Result<Option<OptValue>>,
{
    fn call(&mut self, uid: Uid, set: &S, value: OptValue) -> Result<Option<OptValue>> {
        self.0(uid, set, value)
    }
}

/// Simple struct implemented [`OptFnMut`].
pub struct SimpleOptFnMut<S, T>(T, PhantomData<S>)
where
    S: 'static + Set,
    T: 'static + for<'b> FnMut(Uid, &'b mut S, OptValue) -> Result<Option<OptValue>>;

impl<S, T> SimpleOptFnMut<S, T>
where
    S: 'static + Set,
    T: 'static + for<'b> FnMut(Uid, &'b mut S, OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<S, T> Debug for SimpleOptFnMut<S, T>
where
    S: 'static + Set,
    T: 'static + FnMut(Uid, &mut S, OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOptFnMut")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

impl<S, T> OptFnMut<S> for SimpleOptFnMut<S, T>
where
    S: 'static + Set,
    T: 'static + for<'b> FnMut(Uid, &'b mut S, OptValue) -> Result<Option<OptValue>>,
{
    fn call(&mut self, uid: Uid, set: &mut S, value: OptValue) -> Result<Option<OptValue>> {
        self.0(uid, set, value)
    }
}

/// Simple struct implemented [`PosFn`].
pub struct SimplePosFn<S, T>(T, PhantomData<S>)
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, &str, u64, OptValue) -> Result<Option<OptValue>>;

impl<S, T> SimplePosFn<S, T>
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<S, T> Debug for SimplePosFn<S, T>
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePosFn")
            .field("Fn", &String::from("..."))
            .finish()
    }
}

impl<S, T> PosFn<S> for SimplePosFn<S, T>
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    fn call(
        &mut self,
        uid: Uid,
        set: &S,
        arg: &str,
        noa_index: u64,
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        self.0(uid, set, arg, noa_index, value)
    }
}

/// Simple struct implemented [`PosFnMut`].
pub struct SimplePosFnMut<S, T>(T, PhantomData<S>)
where
    S: 'static + Set,
    T: 'static + FnMut(Uid, &mut S, &str, u64, OptValue) -> Result<Option<OptValue>>;

impl<S, T> SimplePosFnMut<S, T>
where
    S: 'static + Set,
    T: 'static + FnMut(Uid, &mut S, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<S, T> Debug for SimplePosFnMut<S, T>
where
    S: 'static + Set,
    T: 'static + FnMut(Uid, &mut S, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePosFnMut")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

impl<S, T> PosFnMut<S> for SimplePosFnMut<S, T>
where
    S: 'static + Set,
    T: 'static + FnMut(Uid, &mut S, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    fn call(
        &mut self,
        uid: Uid,
        set: &mut S,
        arg: &str,
        noa_index: u64,
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        self.0(uid, set, arg, noa_index, value)
    }
}

/// Simple struct implemented [`MainFn`].
pub struct SimpleMainFn<S, T>(T, PhantomData<S>)
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, &[&str], OptValue) -> Result<Option<OptValue>>;

impl<S, T> SimpleMainFn<S, T>
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<S, T> Debug for SimpleMainFn<S, T>
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMainFn")
            .field("Fn", &String::from("..."))
            .finish()
    }
}

impl<S, T> MainFn<S> for SimpleMainFn<S, T>
where
    S: 'static + Set,
    T: 'static + Fn(Uid, &S, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    fn call(
        &mut self,
        uid: Uid,
        set: &S,
        args: &[&str],
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        self.0(uid, set, args, value)
    }
}

/// Simple struct implemented [`MainFnMut`].
pub struct SimpleMainFnMut<S, T>(T, PhantomData<S>)
where
    S: 'static + Set,
    T: 'static + FnMut(Uid, &mut S, &[&str], OptValue) -> Result<Option<OptValue>>;

impl<S, T> SimpleMainFnMut<S, T>
where
    S: 'static + Set,
    T: 'static + FnMut(Uid, &mut S, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<S, T> Debug for SimpleMainFnMut<S, T>
where
    S: 'static + Set,
    T: 'static + FnMut(Uid, &mut S, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMainFnMut")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

impl<S, T> MainFnMut<S> for SimpleMainFnMut<S, T>
where
    S: 'static + Set,
    T: 'static + FnMut(Uid, &mut S, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    fn call(
        &mut self,
        uid: Uid,
        set: &mut S,
        args: &[&str],
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        self.0(uid, set, args, value)
    }
}
