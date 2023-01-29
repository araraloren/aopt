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

use super::ParserCommitInfered;

/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommit<'a, Set, Ser>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    inner: Option<SetCommit<'a, Set>>,

    inv_ser: Option<&'a mut Invoker<Set, Ser>>,
}

impl<'a, Set, Ser> Debug for ParserCommit<'a, Set, Ser>
where
    Set: crate::set::Set + Debug,
    SetOpt<Set>: Opt + Debug,
    Ser: Debug,
    SetCfg<Set>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserCommit")
            .field("inner", &self.inner)
            .field("inv_ser", &self.inv_ser)
            .finish()
    }
}

impl<'a, Set, Ser> Commit<Set> for ParserCommit<'a, Set, Ser>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    fn cfg(&self) -> &SetCfg<Set> {
        self.inner.as_ref().unwrap().cfg()
    }

    fn cfg_mut(&mut self) -> &mut SetCfg<Set> {
        self.inner.as_mut().unwrap().cfg_mut()
    }
}

impl<'a, Set, Ser> ParserCommit<'a, Set, Ser>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the type of option.
    pub fn set_type<U: Infer>(mut self) -> ParserCommitInfered<'a, Set, Ser, U>
    where
        U::Val: RawValParser,
    {
        let mut inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        inner.drop_commit = false;
        ParserCommitInfered::new(inner.set_type::<U>(), inv_ser)
    }

    /// Set the option value validator.
    pub fn set_validator<U: Infer>(
        self,
        validator: ValValidator<U::Val>,
    ) -> ParserCommitInfered<'a, Set, Ser, U>
    where
        U::Val: RawValParser,
    {
        self.set_type::<U>().set_validator(validator)
    }

    /// Set the option default value.
    pub fn set_value<U: Infer>(self, value: U::Val) -> ParserCommitInfered<'a, Set, Ser, U>
    where
        U::Val: Copy + RawValParser,
    {
        self.set_type::<U>().set_value(value)
    }

    /// Set the option default value.
    pub fn set_value_clone<U: Infer>(self, value: U::Val) -> ParserCommitInfered<'a, Set, Ser, U>
    where
        U::Val: Clone + RawValParser,
    {
        self.set_type::<U>().set_value_clone(value)
    }

    /// Set the option default value.
    pub fn set_values<U: Infer>(self, value: Vec<U::Val>) -> ParserCommitInfered<'a, Set, Ser, U>
    where
        U::Val: Clone + RawValParser,
    {
        self.set_type::<U>().set_values(value)
    }
}

impl<'a, Set, Ser> ParserCommit<'a, Set, Ser>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn new(inner: SetCommit<'a, Set>, inv_ser: &'a mut Invoker<Set, Ser>) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
        }
    }

    pub fn inner_mut(&mut self) -> &mut SetCommit<'a, Set> {
        self.inner.as_mut().unwrap()
    }

    #[cfg(not(feature = "sync"))]
    /// Register the handler which will be called when option is set.
    /// The function will register the option to [`Set`](crate::set::Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn on<H, O, A>(mut self, handler: H) -> Result<HandlerEntry<'a, Set, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<Set, Ser, Error = Error> + 'static,
    {
        let uid = self.run_and_commit_the_change()?;
        // we don't need &'a mut Invoker, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        Ok(HandlerEntry::new(ser.unwrap(), uid).on(handler))
    }

    #[cfg(feature = "sync")]
    /// Register the handler which will be called when option is set.
    /// The function will register the option to [`Set`](crate::set::Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn on<H, O, A>(mut self, handler: H) -> Result<HandlerEntry<'a, Set, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
        A: Extract<Set, Ser, Error = Error> + Send + Sync + 'static,
    {
        let uid = self.run_and_commit_the_change()?;
        // we don't need &'a mut InvokeServices, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        Ok(HandlerEntry::new(ser.unwrap(), uid).on(handler))
    }

    #[cfg(not(feature = "sync"))]
    /// Register the handler which will be called when option is set.
    /// And the [`fallback`](crate::ctx::Invoker::fallback) will be called if
    /// the handler return None.
    /// The function will register the option to [`Set`](crate::set::Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn fallback<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, Set, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<Set, Ser, Error = Error> + 'static,
    {
        let uid = self.run_and_commit_the_change()?;
        // we don't need &'a mut Invoker, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        Ok(HandlerEntry::new(ser.unwrap(), uid).fallback(handler))
    }

    #[cfg(feature = "sync")]
    /// Register the handler which will be called when option is set.
    /// And the [`fallback`](crate::ctx::Invoker::fallback) will be called if
    /// the handler return None.
    /// The function will register the option to [`Set`](crate::set::Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn fallback<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, Set, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
        A: Extract<Set, Ser, Error = Error> + Send + Sync + 'static,
    {
        let uid = self.run_and_commit_the_change()?;
        // we don't need &'a mut InvokeServices, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        //self.drop_commit = false;
        Ok(HandlerEntry::new(ser.unwrap(), uid).fallback(handler))
    }

    pub(crate) fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
        self.inner_mut().run_and_commit_the_change()
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`](crate::set::Ctor).
    /// And add it to referenced [`Set`](crate::set::Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.run_and_commit_the_change()
    }
}

/// Convert [`Commit`] to [`CommitWithValue`].
impl<'a, Set, Ser> ParserCommit<'a, Set, Ser>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the type of option.
    fn set_value_type<T: ErasedTy>(mut self) -> ParserCommitWithValue<'a, Set, Ser, T> {
        let mut inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        inner.drop_commit = false;
        ParserCommitWithValue::new(inner, inv_ser)
    }

    /// Set the option value validator.
    pub fn set_validator_t<T: ErasedTy + RawValParser>(
        self,
        validator: ValValidator<T>,
    ) -> ParserCommitWithValue<'a, Set, Ser, T> {
        self.set_value_type::<T>().set_validator_t(validator)
    }

    /// Set the option default value.
    pub fn set_value_t<T: ErasedTy + Copy>(
        self,
        value: T,
    ) -> ParserCommitWithValue<'a, Set, Ser, T> {
        self.set_value_type::<T>().set_value_t(value)
    }

    /// Set the option default value.
    pub fn set_value_clone_t<T: ErasedTy + Clone>(
        self,
        value: T,
    ) -> ParserCommitWithValue<'a, Set, Ser, T> {
        self.set_value_type::<T>()
            .set_initializer(ValInitializer::with_clone(value))
    }

    /// Set the option default value.
    pub fn set_values_t<T: ErasedTy + Clone>(
        self,
        value: Vec<T>,
    ) -> ParserCommitWithValue<'a, Set, Ser, T> {
        self.set_value_type::<T>()
            .set_initializer(ValInitializer::with_vec(value))
    }
}

impl<'a, Set, Ser> Drop for ParserCommit<'a, Set, Ser>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_ref() {
            if inner.drop_commit {
                let error =
                    "Error when commit the option in ParserCommit::Drop, call `run` get the Result";

                self.run_and_commit_the_change().expect(error);
            }
        }
    }
}

/// Create option using given configurations.
pub struct ParserCommitWithValue<'a, Set, Ser, T>
where
    T: ErasedTy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    inner: Option<SetCommit<'a, Set>>,

    inv_ser: Option<&'a mut Invoker<Set, Ser>>,

    marker: PhantomData<T>,
}

impl<'a, Set, Ser, T> ParserCommitWithValue<'a, Set, Ser, T>
where
    T: ErasedTy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn new(inner: SetCommit<'a, Set>, inv_ser: &'a mut Invoker<Set, Ser>) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
            marker: PhantomData::default(),
        }
    }
}

impl<'a, Set, Ser, T> Commit<Set> for ParserCommitWithValue<'a, Set, Ser, T>
where
    T: ErasedTy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    fn cfg(&self) -> &SetCfg<Set> {
        self.inner.as_ref().unwrap().cfg()
    }

    fn cfg_mut(&mut self) -> &mut SetCfg<Set> {
        self.inner.as_mut().unwrap().cfg_mut()
    }
}

impl<'a, Set, Ser, T> ParserCommitWithValue<'a, Set, Ser, T>
where
    T: ErasedTy + RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator_t(mut self, validator: ValValidator<T>) -> Self {
        self.cfg_mut()
            .set_storer(ValStorer::new_validator(validator));
        self
    }
}

impl<'a, Set, Ser, T> ParserCommitWithValue<'a, Set, Ser, T>
where
    T: ErasedTy + Copy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_t(self, value: T) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}
impl<'a, Set, Ser, T> ParserCommitWithValue<'a, Set, Ser, T>
where
    T: ErasedTy + Clone,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_clone_t(self, value: T) -> Self {
        self.set_initializer(ValInitializer::with_clone(value))
    }

    /// Set the option default value.
    pub fn set_values_t(self, value: Vec<T>) -> Self {
        self.set_initializer(ValInitializer::with_vec(value))
    }
}
