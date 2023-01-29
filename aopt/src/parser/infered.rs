use std::fmt::Debug;
use std::marker::PhantomData;

use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::HandlerEntry;
use crate::ctx::Invoker;
use crate::map::ErasedTy;
use crate::opt::Action;
use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::opt::Opt;
use crate::set::Commit;
use crate::set::SetCfg;
use crate::set::SetCommit;
use crate::set::SetCommitInfered;
use crate::set::SetOpt;
use crate::value::Infer;
use crate::value::RawValParser;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::value::ValValidator;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommitInfered<'a, Set, Ser, U>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    inner: Option<SetCommitInfered<'a, Set, U>>,

    inv_ser: Option<&'a mut Invoker<Set, Ser>>,

    marker: PhantomData<U>,
}

impl<'a, Set, Ser, U> ParserCommitInfered<'a, Set, Ser, U>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn new(inner: SetCommitInfered<'a, Set, U>, inv_ser: &'a mut Invoker<Set, Ser>) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
            marker: PhantomData::default(),
        }
    }
}

impl<'a, Set, Ser, U> Commit<Set> for ParserCommitInfered<'a, Set, Ser, U>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    fn cfg(&self) -> &SetCfg<Set> {
        self.inner.as_ref().unwrap().cfg()
    }

    fn cfg_mut(&mut self) -> &mut SetCfg<Set> {
        self.inner.as_mut().unwrap().cfg_mut()
    }
}

impl<'a, Set, Ser, U> ParserCommitInfered<'a, Set, Ser, U>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }
}

impl<'a, Set, Ser, U> ParserCommitInfered<'a, Set, Ser, U>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: Copy + RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}
impl<'a, Set, Ser, U> ParserCommitInfered<'a, Set, Ser, U>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: Clone + RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_clone(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with_clone(value))
    }

    /// Set the option default value.
    pub fn set_values(self, value: Vec<U::Val>) -> Self {
        self.set_initializer(ValInitializer::with_vec(value))
    }
}

/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommitInferedWithValue<'a, Set, Ser, U, T>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: RawValParser,
    T: ErasedTy,
    SetCfg<Set>: ConfigValue + Default,
{
    inner: Option<SetCommitInfered<'a, Set, U>>,

    inv_ser: Option<&'a mut Invoker<Set, Ser>>,

    marker: PhantomData<(U, T)>,
}
