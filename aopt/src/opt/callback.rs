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

/// Callback trait using for [`Callback::Opt`], associated callback type is [`CallbackType::Opt`].
pub trait OptFn: Debug {
    fn call(&mut self, uid: Uid, set: &dyn Set, value: OptValue) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::OptMut`], associated callback type is [`CallbackType::OptMut`].
pub trait OptFnMut: Debug {
    fn call(&mut self, uid: Uid, set: &mut dyn Set, value: OptValue) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::Pos`], associated callback type is [`CallbackType::Pos`].
pub trait PosFn: Debug {
    fn call(
        &mut self,
        uid: Uid,
        set: &dyn Set,
        arg: &str,
        noa_index: u64,
        value: OptValue,
    ) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::PosMut`], associated callback type is [`CallbackType::PosMut`].
pub trait PosFnMut: Debug {
    fn call(
        &mut self,
        uid: Uid,
        set: &mut dyn Set,
        arg: &str,
        noa_index: u64,
        value: OptValue,
    ) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::Main`], associated callback type is [`CallbackType::Main`].
pub trait MainFn: Debug {
    fn call(
        &mut self,
        uid: Uid,
        set: &dyn Set,
        args: &[&str],
        value: OptValue,
    ) -> Result<Option<OptValue>>;
}

/// Callback trait using for [`Callback::MainMut`], associated callback type is [`CallbackType::MainMut`].
pub trait MainFnMut: Debug {
    fn call(
        &mut self,
        uid: Uid,
        set: &mut dyn Set,
        args: &[&str],
        value: OptValue,
    ) -> Result<Option<OptValue>>;
}

/// The callback type hold block code.
#[derive(Debug)]
pub enum Callback {
    Opt(Box<dyn OptFn>),

    OptMut(Box<dyn OptFnMut>),

    Pos(Box<dyn PosFn>),

    PosMut(Box<dyn PosFnMut>),

    Main(Box<dyn MainFn>),

    MainMut(Box<dyn MainFnMut>),

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

impl From<Box<dyn OptFn>> for Callback {
    fn from(cb: Box<dyn OptFn>) -> Self {
        Callback::Opt(cb)
    }
}

impl From<Box<dyn OptFnMut>> for Callback {
    fn from(cb: Box<dyn OptFnMut>) -> Self {
        Callback::OptMut(cb)
    }
}

impl From<Box<dyn PosFn>> for Callback {
    fn from(cb: Box<dyn PosFn>) -> Self {
        Callback::Pos(cb)
    }
}

impl From<Box<dyn PosFnMut>> for Callback {
    fn from(cb: Box<dyn PosFnMut>) -> Self {
        Callback::PosMut(cb)
    }
}

impl From<Box<dyn MainFn>> for Callback {
    fn from(cb: Box<dyn MainFn>) -> Self {
        Callback::Main(cb)
    }
}

impl From<Box<dyn MainFnMut>> for Callback {
    fn from(cb: Box<dyn MainFnMut>) -> Self {
        Callback::MainMut(cb)
    }
}

/// Simple struct implemented [`OptFn`].
pub struct SimpleOptFn<'a, T>(T, PhantomData<&'a T>)
where
    T: 'a + Fn(Uid, &dyn Set, OptValue) -> Result<Option<OptValue>>;

impl<'a, T> SimpleOptFn<'a, T>
where
    T: 'a + Fn(Uid, &dyn Set, OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<'a, T> Debug for SimpleOptFn<'a, T>
where
    T: 'a + Fn(Uid, &dyn Set, OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOptFn")
            .field("Fn", &String::from("..."))
            .finish()
    }
}

impl<'a, T> OptFn for SimpleOptFn<'a, T>
where
    T: 'a + Fn(Uid, &dyn Set, OptValue) -> Result<Option<OptValue>>,
{
    fn call(&mut self, uid: Uid, set: &dyn Set, value: OptValue) -> Result<Option<OptValue>> {
        self.0(uid, set, value)
    }
}

/// Simple struct implemented [`OptFnMut`].
pub struct SimpleOptFnMut<'a, T>(T, PhantomData<&'a T>)
where
    T: 'a + FnMut(Uid, &mut dyn Set, OptValue) -> Result<Option<OptValue>>;

impl<'a, T> SimpleOptFnMut<'a, T>
where
    T: 'a + FnMut(Uid, &mut dyn Set, OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<'a, T> Debug for SimpleOptFnMut<'a, T>
where
    T: 'a + FnMut(Uid, &mut dyn Set, OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleOptFnMut")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

impl<'a, T> OptFnMut for SimpleOptFnMut<'a, T>
where
    T: 'a + FnMut(Uid, &mut dyn Set, OptValue) -> Result<Option<OptValue>>,
{
    fn call(&mut self, uid: Uid, set: &mut dyn Set, value: OptValue) -> Result<Option<OptValue>> {
        self.0(uid, set, value)
    }
}

/// Simple struct implemented [`PosFn`].
pub struct SimplePosFn<'a, T>(T, PhantomData<&'a T>)
where
    T: 'a + Fn(Uid, &dyn Set, &str, u64, OptValue) -> Result<Option<OptValue>>;

impl<'a, T> SimplePosFn<'a, T>
where
    T: 'a + Fn(Uid, &dyn Set, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<'a, T> Debug for SimplePosFn<'a, T>
where
    T: 'a + Fn(Uid, &dyn Set, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePosFn")
            .field("Fn", &String::from("..."))
            .finish()
    }
}

impl<'a, T> PosFn for SimplePosFn<'a, T>
where
    T: 'a + Fn(Uid, &dyn Set, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    fn call(
        &mut self,
        uid: Uid,
        set: &dyn Set,
        arg: &str,
        noa_index: u64,
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        self.0(uid, set, arg, noa_index, value)
    }
}

/// Simple struct implemented [`PosFnMut`].
pub struct SimplePosFnMut<'a, T>(T, PhantomData<&'a T>)
where
    T: 'a + FnMut(Uid, &mut dyn Set, &str, u64, OptValue) -> Result<Option<OptValue>>;

impl<'a, T> SimplePosFnMut<'a, T>
where
    T: 'a + FnMut(Uid, &mut dyn Set, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<'a, T> Debug for SimplePosFnMut<'a, T>
where
    T: 'a + FnMut(Uid, &mut dyn Set, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePosFnMut")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

impl<'a, T> PosFnMut for SimplePosFnMut<'a, T>
where
    T: 'a + FnMut(Uid, &mut dyn Set, &str, u64, OptValue) -> Result<Option<OptValue>>,
{
    fn call(
        &mut self,
        uid: Uid,
        set: &mut dyn Set,
        arg: &str,
        noa_index: u64,
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        self.0(uid, set, arg, noa_index, value)
    }
}

/// Simple struct implemented [`MainFn`].
pub struct SimpleMainFn<'a, T>(T, PhantomData<&'a T>)
where
    T: 'a + Fn(Uid, &dyn Set, &[&str], OptValue) -> Result<Option<OptValue>>;

impl<'a, T> SimpleMainFn<'a, T>
where
    T: 'a + Fn(Uid, &dyn Set, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<'a, T> Debug for SimpleMainFn<'a, T>
where
    T: 'a + Fn(Uid, &dyn Set, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMainFn")
            .field("Fn", &String::from("..."))
            .finish()
    }
}

impl<'a, T> MainFn for SimpleMainFn<'a, T>
where
    T: 'a + Fn(Uid, &dyn Set, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    fn call(
        &mut self,
        uid: Uid,
        set: &dyn Set,
        args: &[&str],
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        self.0(uid, set, args, value)
    }
}

/// Simple struct implemented [`MainFnMut`].
pub struct SimpleMainFnMut<'a, T>(T, PhantomData<&'a T>)
where
    T: 'a + FnMut(Uid, &mut dyn Set, &[&str], OptValue) -> Result<Option<OptValue>>;

impl<'a, T> SimpleMainFnMut<'a, T>
where
    T: 'a + FnMut(Uid, &mut dyn Set, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    pub fn new(cb: T) -> Self {
        Self(cb, PhantomData::default())
    }
}

impl<'a, T> Debug for SimpleMainFnMut<'a, T>
where
    T: 'a + FnMut(Uid, &mut dyn Set, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMainFnMut")
            .field("FnMut", &String::from("..."))
            .finish()
    }
}

impl<'a, T> MainFnMut for SimpleMainFnMut<'a, T>
where
    T: 'a + FnMut(Uid, &mut dyn Set, &[&str], OptValue) -> Result<Option<OptValue>>,
{
    fn call(
        &mut self,
        uid: Uid,
        set: &mut dyn Set,
        args: &[&str],
        value: OptValue,
    ) -> Result<Option<OptValue>> {
        self.0(uid, set, args, value)
    }
}
