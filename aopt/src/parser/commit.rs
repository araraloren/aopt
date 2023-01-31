use std::fmt::Debug;

use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::HandlerEntry;
use crate::ctx::Invoker;
use crate::map::ErasedTy;
use crate::opt::ConfigValue;
use crate::opt::Opt;
use crate::set::Commit;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetCommit;
use crate::set::SetCommitWithValue;
use crate::set::SetOpt;
use crate::value::Infer;
use crate::value::RawValParser;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::value::ValValidator;
use crate::Error;
use crate::Uid;

/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommit<'a, S, Ser, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    inner: Option<SetCommit<'a, S, U>>,

    inv_ser: Option<&'a mut Invoker<S, Ser>>,
}

impl<'a, S, Ser, U> Debug for ParserCommit<'a, S, Ser, U>
where
    U: Infer,
    U::Val: RawValParser,
    S: Set + Debug,
    SetOpt<S>: Opt + Debug,
    SetCfg<S>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserCommit")
            .field("inner", &self.inner)
            .field("inv_ser", &self.inv_ser)
            .finish()
    }
}

impl<'a, S, Ser, U> Commit<S> for ParserCommit<'a, S, Ser, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    fn cfg(&self) -> &SetCfg<S> {
        self.inner.as_ref().unwrap().cfg()
    }

    fn cfg_mut(&mut self) -> &mut SetCfg<S> {
        self.inner.as_mut().unwrap().cfg_mut()
    }
}

impl<'a, S, Ser, U> ParserCommit<'a, S, Ser, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(inner: SetCommit<'a, S, U>, inv_ser: &'a mut Invoker<S, Ser>) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
        }
    }

    pub fn inner(&self) -> Result<&SetCommit<'a, S, U>, Error> {
        self.inner
            .as_ref()
            .ok_or_else(|| Error::raise_error("Must set inner data of ParserCommit(ref)"))
    }

    pub fn inner_mut(&mut self) -> Result<&mut SetCommit<'a, S, U>, Error> {
        self.inner
            .as_mut()
            .ok_or_else(|| Error::raise_error("Must set inner data of ParserCommit(mut)"))
    }

    /// Set the type of option.
    pub fn set_type<O: Infer>(mut self) -> ParserCommit<'a, S, Ser, O>
    where
        O::Val: RawValParser,
    {
        let inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        ParserCommit::new(inner.set_type::<O>(), inv_ser)
    }

    #[cfg(not(feature = "sync"))]
    /// Register the handler which will be called when option is set.
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn on<H, O, A>(mut self, handler: H) -> Result<HandlerEntry<'a, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<S, Ser, Error = Error> + 'static,
    {
        let uid = self.commit_inner_change()?;
        // we don't need &'a mut Invoker, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        Ok(HandlerEntry::new(ser.unwrap(), uid).on(handler))
    }

    #[cfg(feature = "sync")]
    /// Register the handler which will be called when option is set.
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn on<H, O, A>(mut self, handler: H) -> Result<HandlerEntry<'a, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
        A: Extract<S, Ser, Error = Error> + Send + Sync + 'static,
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
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn fallback<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<S, Ser, Error = Error> + 'static,
    {
        let uid = self.commit_inner_change()?;
        // we don't need &'a mut Invoker, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        Ok(HandlerEntry::new(ser.unwrap(), uid).fallback(handler))
    }

    #[cfg(feature = "sync")]
    /// Register the handler which will be called when option is set.
    /// And the [`fallback`](crate::ctx::Invoker::fallback) will be called if
    /// the handler return None.
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn fallback<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
        A: Extract<S, Ser, Error = Error> + Send + Sync + 'static,
    {
        let uid = self.run_and_commit_the_change()?;
        // we don't need &'a mut InvokeServices, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        //self.drop_commit = false;
        Ok(HandlerEntry::new(ser.unwrap(), uid).fallback(handler))
    }

    pub(crate) fn commit_inner_change(&mut self) -> Result<Uid, Error> {
        self.inner_mut()?.commit_change()
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`](crate::set::Ctor).
    /// And add it to referenced [`Set`](Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.commit_inner_change()
    }
}

impl<'a, S, Ser, U> ParserCommit<'a, S, Ser, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }
}

impl<'a, S, Ser, U> ParserCommit<'a, S, Ser, U>
where
    S: Set,
    U: Infer,
    U::Val: Copy + RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}

impl<'a, S, Ser, U> ParserCommit<'a, S, Ser, U>
where
    S: Set,
    U: Infer,
    U::Val: Clone + RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
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

/// Convert [`Commit`] to [`CommitWithValue`].
impl<'a, S, Ser, U> ParserCommit<'a, S, Ser, U>
where
    S: Set,
    U: Infer,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the type of option.
    pub(crate) fn set_value_type<T: ErasedTy>(mut self) -> ParserCommitWithValue<'a, S, Ser, U, T> {
        let inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        ParserCommitWithValue::new(inner.set_value_type::<T>(), inv_ser)
    }

    /// Set the option value validator.
    pub fn set_validator_t<T: ErasedTy + RawValParser>(
        self,
        validator: ValValidator<T>,
    ) -> ParserCommitWithValue<'a, S, Ser, U, T> {
        self.set_value_type::<T>().set_validator_t(validator)
    }

    /// Set the option default value.
    pub fn set_value_t<T: ErasedTy + Copy>(
        self,
        value: T,
    ) -> ParserCommitWithValue<'a, S, Ser, U, T> {
        self.set_value_type::<T>().set_value_t(value)
    }

    /// Set the option default value.
    pub fn set_value_clone_t<T: ErasedTy + Clone>(
        self,
        value: T,
    ) -> ParserCommitWithValue<'a, S, Ser, U, T> {
        self.set_value_type::<T>()
            .set_initializer(ValInitializer::with_clone(value))
    }

    /// Set the option default value.
    pub fn set_values_t<T: ErasedTy + Clone>(
        self,
        value: Vec<T>,
    ) -> ParserCommitWithValue<'a, S, Ser, U, T> {
        self.set_value_type::<T>()
            .set_initializer(ValInitializer::with_vec(value))
    }
}

/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommitWithValue<'a, S, Ser, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    inner: Option<SetCommitWithValue<'a, S, U, T>>,

    inv_ser: Option<&'a mut Invoker<S, Ser>>,
}

impl<'a, S, Ser, U, T> Debug for ParserCommitWithValue<'a, S, Ser, U, T>
where
    U: Infer,
    T: ErasedTy,
    S: Set + Debug,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserCommitInfered")
            .field("inner", &self.inner)
            .field("inv_ser", &self.inv_ser)
            .finish()
    }
}

impl<'a, S, Ser, U, T> ParserCommitWithValue<'a, S, Ser, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(inner: SetCommitWithValue<'a, S, U, T>, inv_ser: &'a mut Invoker<S, Ser>) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
        }
    }

    pub fn inner(&self) -> Result<&SetCommitWithValue<'a, S, U, T>, Error> {
        self.inner
            .as_ref()
            .ok_or_else(|| Error::raise_error("Must set inner data of ParserCommitWithValue(ref)"))
    }

    pub fn inner_mut(&mut self) -> Result<&mut SetCommitWithValue<'a, S, U, T>, Error> {
        self.inner
            .as_mut()
            .ok_or_else(|| Error::raise_error("Must set inner data of ParserCommitWithValue(mut)"))
    }

    /// Set the type of option.
    pub fn set_type<O: Infer>(mut self) -> ParserCommitWithValue<'a, S, Ser, O, T>
    where
        O::Val: RawValParser,
    {
        let inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        ParserCommitWithValue::new(inner.set_type::<O>(), inv_ser)
    }

    #[cfg(not(feature = "sync"))]
    /// Register the handler which will be called when option is set.
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn on<H, O, A>(mut self, handler: H) -> Result<HandlerEntry<'a, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<S, Ser, Error = Error> + 'static,
    {
        let uid = self.commit_inner_change()?;
        // we don't need &'a mut Invoker, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        Ok(HandlerEntry::new(ser.unwrap(), uid).on(handler))
    }

    #[cfg(feature = "sync")]
    /// Register the handler which will be called when option is set.
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn on<H, O, A>(mut self, handler: H) -> Result<HandlerEntry<'a, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
        A: Extract<S, Ser, Error = Error> + Send + Sync + 'static,
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
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn fallback<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<S, Ser, Error = Error> + 'static,
    {
        let uid = self.commit_inner_change()?;
        // we don't need &'a mut Invoker, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        Ok(HandlerEntry::new(ser.unwrap(), uid).fallback(handler))
    }

    #[cfg(feature = "sync")]
    /// Register the handler which will be called when option is set.
    /// And the [`fallback`](crate::ctx::Invoker::fallback) will be called if
    /// the handler return None.
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn fallback<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
        A: Extract<S, Ser, Error = Error> + Send + Sync + 'static,
    {
        let uid = self.run_and_commit_the_change()?;
        // we don't need &'a mut InvokeServices, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        //self.drop_commit = false;
        Ok(HandlerEntry::new(ser.unwrap(), uid).fallback(handler))
    }

    pub(crate) fn commit_inner_change(&mut self) -> Result<Uid, Error> {
        self.inner_mut()?.commit_inner_change()
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`](crate::set::Ctor).
    /// And add it to referenced [`Set`](Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.commit_inner_change()
    }
}

impl<'a, S, Ser, U, T> Commit<S> for ParserCommitWithValue<'a, S, Ser, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    fn cfg(&self) -> &SetCfg<S> {
        self.inner.as_ref().unwrap().cfg()
    }

    fn cfg_mut(&mut self) -> &mut SetCfg<S> {
        self.inner.as_mut().unwrap().cfg_mut()
    }
}

impl<'a, S, Ser, U, T> ParserCommitWithValue<'a, S, Ser, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }
}

impl<'a, S, Ser, U, T> ParserCommitWithValue<'a, S, Ser, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: Copy + RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}
impl<'a, S, Ser, U, T> ParserCommitWithValue<'a, S, Ser, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy,
    U::Val: Clone + RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
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

impl<'a, S, Ser, U, T> ParserCommitWithValue<'a, S, Ser, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy + RawValParser,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator_t(mut self, validator: ValValidator<T>) -> Self {
        self.cfg_mut()
            .set_storer(ValStorer::new_validator(validator));
        self
    }
}

impl<'a, S, Ser, U, T> ParserCommitWithValue<'a, S, Ser, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy + Copy,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_t(self, value: T) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}

impl<'a, S, Ser, U, T> ParserCommitWithValue<'a, S, Ser, U, T>
where
    S: Set,
    U: Infer,
    T: ErasedTy + Clone,
    U::Val: RawValParser,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
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
