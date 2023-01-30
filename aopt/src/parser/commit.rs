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
use crate::set::SetCommitW;
use crate::set::SetCommitWT;
use crate::set::SetOpt;
use crate::value::Infer;
use crate::value::Placeholder;
use crate::value::RawValParser;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::value::ValValidator;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    U::Val: RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    inner: Option<SetCommitW<'a, Set, U>>,

    inv_ser: Option<&'a mut Invoker<Set, Ser>>,
}

impl<'a, Set, Ser, U> Debug for ParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    U::Val: RawValParser,
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

impl<'a, Set, Ser, U> Commit<Set> for ParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    U::Val: RawValParser,
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

impl<'a, Set, Ser> ParserCommit<'a, Set, Ser, Placeholder>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn new_placeholder(
        inner: SetCommitW<'a, Set, Placeholder>,
        inv_ser: &'a mut Invoker<Set, Ser>,
    ) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
        }
    }
}

impl<'a, Set, Ser, U> ParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    U::Val: RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn new(inner: SetCommitW<'a, Set, U>, inv_ser: &'a mut Invoker<Set, Ser>) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
        }
    }

    pub fn inner_mut(&mut self) -> &mut SetCommitW<'a, Set, U> {
        self.inner.as_mut().unwrap()
    }

    /// Set the type of option.
    pub fn set_type<O: Infer>(mut self) -> ParserCommit<'a, Set, Ser, O>
    where
        O::Val: RawValParser,
    {
        let mut inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        inner.drop = false;
        ParserCommit::new(inner.set_type::<O>(), inv_ser)
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

impl<'a, Set, Ser, U> ParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    U::Val: RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }
}

impl<'a, Set, Ser, U> ParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    U::Val: Copy + RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}

impl<'a, Set, Ser, U> ParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    U::Val: Clone + RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
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

/// Convert [`Commit`] to [`CommitWithValue`].
impl<'a, Set, Ser, U> ParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    U::Val: RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the type of option.
    pub fn set_value_type<T: ErasedTy>(mut self) -> ParserCommitInfered<'a, Set, Ser, U, T> {
        let mut inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        inner.drop = false;
        ParserCommitInfered::new(inner.set_value_type::<T>(), inv_ser)
    }

    /// Set the option value validator.
    pub fn set_validator_t<T: ErasedTy + RawValParser>(
        self,
        validator: ValValidator<T>,
    ) -> ParserCommitInfered<'a, Set, Ser, U, T> {
        self.set_value_type::<T>().set_validator_t(validator)
    }

    /// Set the option default value.
    pub fn set_value_t<T: ErasedTy + Copy>(
        self,
        value: T,
    ) -> ParserCommitInfered<'a, Set, Ser, U, T> {
        self.set_value_type::<T>().set_value_t(value)
    }

    /// Set the option default value.
    pub fn set_value_clone_t<T: ErasedTy + Clone>(
        self,
        value: T,
    ) -> ParserCommitInfered<'a, Set, Ser, U, T> {
        self.set_value_type::<T>()
            .set_initializer(ValInitializer::with_clone(value))
    }

    /// Set the option default value.
    pub fn set_values_t<T: ErasedTy + Clone>(
        self,
        value: Vec<T>,
    ) -> ParserCommitInfered<'a, Set, Ser, U, T> {
        self.set_value_type::<T>()
            .set_initializer(ValInitializer::with_vec(value))
    }
}

impl<'a, Set, Ser, U> Drop for ParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    U::Val: RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_ref() {
            if inner.drop {
                let error =
                    "Error when commit the option in ParserCommit::Drop, call `run` get the Result";

                self.run_and_commit_the_change().expect(error);
            }
        }
    }
}
/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    inner: Option<SetCommitWT<'a, Set, U, T>>,

    inv_ser: Option<&'a mut Invoker<Set, Ser>>,
}

impl<'a, Set, Ser, T> ParserCommitInfered<'a, Set, Ser, Placeholder, T>
where
    T: ErasedTy,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn new_placeholder(
        inner: SetCommitWT<'a, Set, Placeholder, T>,
        inv_ser: &'a mut Invoker<Set, Ser>,
    ) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
        }
    }
}

impl<'a, Set, Ser, U, T> ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn new(inner: SetCommitWT<'a, Set, U, T>, inv_ser: &'a mut Invoker<Set, Ser>) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
        }
    }

    pub fn inner_mut(&mut self) -> &mut SetCommitWT<'a, Set, U, T> {
        self.inner.as_mut().unwrap()
    }

    /// Set the type of option.
    pub fn set_type<O: Infer>(mut self) -> ParserCommitInfered<'a, Set, Ser, O, T>
    where
        O::Val: RawValParser,
    {
        let mut inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        inner.drop = false;
        ParserCommitInfered::new(inner.set_type::<O>(), inv_ser)
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

impl<'a, Set, Ser, U, T> Commit<Set> for ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
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

impl<'a, Set, Ser, U, T> ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    T: ErasedTy,
    U::Val: RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }
}

impl<'a, Set, Ser, U, T> ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    T: ErasedTy,
    U::Val: Copy + RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}
impl<'a, Set, Ser, U, T> ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    T: ErasedTy,
    U::Val: Clone + RawValParser,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
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

impl<'a, Set, Ser, U, T> ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    U::Val: RawValParser,
    Set: crate::set::Set,
    T: ErasedTy + RawValParser,
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

impl<'a, Set, Ser, U, T> ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    U::Val: RawValParser,
    Set: crate::set::Set,
    T: ErasedTy + Copy,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_t(self, value: T) -> Self {
        self.set_initializer(ValInitializer::with(value))
    }
}

impl<'a, Set, Ser, U, T> ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    U::Val: RawValParser,
    Set: crate::set::Set,
    T: ErasedTy + Clone,
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

impl<'a, Set, Ser, U, T> ParserCommitInfered<'a, Set, Ser, U, T>
where
    U: Infer,
    U::Val: RawValParser,
    Set: crate::set::Set,
    T: ErasedTy + Clone,
    SetOpt<Set>: Opt,
    SetCfg<Set>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_ref() {
            if inner.drop {
                let error =
                    "Error when commit the option in ParserCommit::Drop, call `run` get the Result";

                self.run_and_commit_the_change().expect(error);
            }
        }
    }
}
