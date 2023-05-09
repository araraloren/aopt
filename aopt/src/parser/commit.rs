use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::HandlerCollection;
use crate::ctx::HandlerEntry;
use crate::map::ErasedTy;
use crate::opt::Any;
use crate::opt::Cmd;
use crate::opt::ConfigValue;
use crate::opt::Main;
use crate::opt::Opt;
use crate::opt::Pos;
use crate::raise_error;
use crate::set::Commit;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetCommit;
use crate::set::SetCommitWithValue;
use crate::set::SetOpt;
use crate::value::Infer;
use crate::value::Placeholder;
use crate::value::RawValParser;
use crate::value::ValInitializer;
use crate::value::ValStorer;
use crate::value::ValValidator;
use crate::Error;
use crate::Uid;

/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommit<'a, 'b, I, S, Ser, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    inner: Option<SetCommit<'b, S, U>>,

    inv_ser: Option<&'b mut I>,

    marker: PhantomData<(&'a (), Ser)>,
}

impl<'a, 'b, I, S, Ser, U> Debug for ParserCommit<'a, 'b, I, S, Ser, U>
where
    U: Infer + 'static,
    U::Val: RawValParser,
    S: Set + Debug,
    I: HandlerCollection<'a, S, Ser> + Debug,
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

impl<'a, 'b, I, S, Ser, U> Commit<S> for ParserCommit<'a, 'b, I, S, Ser, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
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

macro_rules! add_interface {
    ($ty:ty, $name1:ident, $name2:ident) => {
        #[doc = concat!("Set the infer type to [`", stringify!($ty), "`]\\<T\\>.")]
        pub fn $name1<T>(
            mut self,
        ) -> ParserCommit<'a, 'b, I, S, Ser, $ty> where T::Val: RawValParser, T: ErasedTy + Infer {
            let inner = self.inner.take().unwrap();
            let inv_ser = self.inv_ser.take().unwrap();

            ParserCommit::new(inner.$name1::<T>(), inv_ser)
        }

        #[doc = concat!("Set the infer type to [`", stringify!($ty) ,"`]\\<T\\>, add default initializer and default storer.")]
        pub fn $name2<T>(
            mut self,
        ) -> ParserCommit<'a, 'b, I, S, Ser, $ty> where T::Val: RawValParser + Clone, T: ErasedTy + Infer {
            let inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        ParserCommit::new(inner.$name2::<T>(), inv_ser)
        }
    }
}

impl<'a, 'b, I, S, Ser> ParserCommit<'a, 'b, I, S, Ser, Placeholder>
where
    S: Set,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
    I: HandlerCollection<'a, S, Ser>,
{
    add_interface!(Option<Pos<T>>, set_pos_type_only, set_pos_type);

    add_interface!(Main<T>, set_main_type_only, set_main_type);

    add_interface!(Any<T>, set_any_type_only, set_any_type);
}

impl<'a, 'b, I, S, Ser, U> ParserCommit<'a, 'b, I, S, Ser, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(inner: SetCommit<'b, S, U>, inv_ser: &'b mut I) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
            marker: PhantomData::default(),
        }
    }

    pub fn inner(&self) -> Result<&SetCommit<'b, S, U>, Error> {
        self.inner
            .as_ref()
            .ok_or_else(|| raise_error!("Must set inner data of ParserCommit(ref)"))
    }

    pub fn inner_mut(&mut self) -> Result<&mut SetCommit<'b, S, U>, Error> {
        self.inner
            .as_mut()
            .ok_or_else(|| raise_error!("Must set inner data of ParserCommit(mut)"))
    }

    /// Set the infer type of option.
    pub fn set_infer<O: Infer>(mut self) -> ParserCommit<'a, 'b, I, S, Ser, O>
    where
        O::Val: RawValParser,
    {
        let inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        ParserCommit::new(inner.set_infer::<O>(), inv_ser)
    }

    #[cfg(not(feature = "sync"))]
    /// Register the handler which will be called when option is set.
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn on<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, 'b, I, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + 'a,
        A: Extract<S, Ser, Error = Error> + 'a,
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
    pub fn on<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, 'b, I, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'a,
        A: Extract<S, Ser, Error = Error> + Send + Sync + 'a,
    {
        let uid = self.commit_inner_change()?;
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
    ) -> Result<HandlerEntry<'a, 'b, I, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + 'a,
        A: Extract<S, Ser, Error = Error> + 'a,
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
    ) -> Result<HandlerEntry<'a, 'b, I, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'a,
        A: Extract<S, Ser, Error = Error> + Send + Sync + 'a,
    {
        let uid = self.commit_inner_change()?;
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

impl<'a, 'b, I, S, Ser, U> ParserCommit<'a, 'b, I, S, Ser, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }

    /// Add default [`storer`](ValStorer::fallback) of type [`U::Val`](Infer::Val).
    pub fn add_default_storer(self) -> Self {
        self.set_storer(ValStorer::fallback::<U::Val>())
    }
}

impl<'a, 'b, I, S, Ser, U> ParserCommit<'a, 'b, I, S, Ser, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: Clone + RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::new_value(value))
    }

    /// Set the option default value.
    pub fn set_values(self, value: Vec<U::Val>) -> Self {
        self.set_initializer(ValInitializer::new_values(value))
    }

    /// Add a default [`initializer`](ValInitializer::fallback).
    pub fn add_default_initializer(self) -> Self {
        self.set_initializer(ValInitializer::fallback())
    }
}

/// Convert [`ParserCommit`] to [`ParserCommitWithValue`].
impl<'a, 'b, I, S, Ser, U> ParserCommit<'a, 'b, I, S, Ser, U>
where
    S: Set,
    U: Infer + 'static,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the value type of option(except for [`Cmd`]).
    pub fn set_value_type_only<T: ErasedTy>(
        mut self,
    ) -> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T> {
        let inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        debug_assert!(
            TypeId::of::<U>() != TypeId::of::<Cmd>() || TypeId::of::<T>() == TypeId::of::<bool>(),
            "For Cmd, you can't have other value type!"
        );
        ParserCommitWithValue::new(inner.set_value_type_only::<T>(), inv_ser)
    }

    /// Set the value type of option, add default initializer and default value storer.
    pub fn set_value_type<T: ErasedTy + Clone + RawValParser>(
        mut self,
    ) -> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T> {
        let inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        ParserCommitWithValue::new(
            inner
                .set_value_type_only::<T>()
                .add_default_initializer_t()
                .add_default_storer_t(),
            inv_ser,
        )
    }

    /// Set the option value validator.
    pub fn set_validator_t<T: ErasedTy + RawValParser>(
        self,
        validator: ValValidator<T>,
    ) -> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T> {
        self.set_value_type_only::<T>().set_validator_t(validator)
    }

    /// Set the option default value.
    pub fn set_value_t<T: ErasedTy + Clone>(
        self,
        value: T,
    ) -> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T> {
        self.set_value_type_only::<T>()
            .set_initializer(ValInitializer::new_value(value))
    }

    /// Set the option default value.
    pub fn set_values_t<T: ErasedTy + Clone>(
        self,
        value: Vec<T>,
    ) -> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T> {
        self.set_value_type_only::<T>()
            .set_initializer(ValInitializer::new_values(value))
    }
}

/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommitWithValue<'a, 'b, I, S, Ser, U, T>
where
    S: Set,
    U: Infer + 'static,
    T: ErasedTy,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    inner: Option<SetCommitWithValue<'b, S, U, T>>,

    inv_ser: Option<&'b mut I>,

    marker: PhantomData<(&'a (), Ser)>,
}

impl<'a, 'b, I, S, Ser, U, T> Debug for ParserCommitWithValue<'a, 'b, I, S, Ser, U, T>
where
    U: Infer + 'static,
    T: ErasedTy,
    S: Set + Debug,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser> + Debug,
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

impl<'a, 'b, I, S, Ser, U, T> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T>
where
    S: Set,
    U: Infer + 'static,
    T: ErasedTy,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(inner: SetCommitWithValue<'b, S, U, T>, inv_ser: &'b mut I) -> Self {
        Self {
            inner: Some(inner),
            inv_ser: Some(inv_ser),
            marker: PhantomData::default(),
        }
    }

    pub fn inner(&self) -> Result<&SetCommitWithValue<'b, S, U, T>, Error> {
        self.inner
            .as_ref()
            .ok_or_else(|| raise_error!("Must set inner data of ParserCommitWithValue(ref)"))
    }

    pub fn inner_mut(&mut self) -> Result<&mut SetCommitWithValue<'b, S, U, T>, Error> {
        self.inner
            .as_mut()
            .ok_or_else(|| raise_error!("Must set inner data of ParserCommitWithValue(mut)"))
    }

    /// Set the infer type of option.
    pub fn set_infer<O: Infer>(mut self) -> ParserCommitWithValue<'a, 'b, I, S, Ser, O, T>
    where
        O::Val: RawValParser,
    {
        let inner = self.inner.take().unwrap();
        let inv_ser = self.inv_ser.take().unwrap();

        ParserCommitWithValue::new(inner.set_infer::<O>(), inv_ser)
    }

    #[cfg(not(feature = "sync"))]
    /// Register the handler which will be called when option is set.
    /// The function will register the option to [`Set`](Set) first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn on<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, 'b, I, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + 'a,
        A: Extract<S, Ser, Error = Error> + 'a,
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
    pub fn on<H, O, A>(
        mut self,
        handler: H,
    ) -> Result<HandlerEntry<'a, 'b, I, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'a,
        A: Extract<S, Ser, Error = Error> + Send + Sync + 'a,
    {
        let uid = self.commit_inner_change()?;
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
    ) -> Result<HandlerEntry<'a, 'b, I, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + 'a,
        A: Extract<S, Ser, Error = Error> + 'a,
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
    ) -> Result<HandlerEntry<'a, 'b, I, S, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<S, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'a,
        A: Extract<S, Ser, Error = Error> + Send + Sync + 'a,
    {
        let uid = self.commit_inner_change()?;
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

impl<'a, 'b, I, S, Ser, U, T> Commit<S> for ParserCommitWithValue<'a, 'b, I, S, Ser, U, T>
where
    S: Set,
    U: Infer + 'static,
    T: ErasedTy,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
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

impl<'a, 'b, I, S, Ser, U, T> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T>
where
    S: Set,
    U: Infer + 'static,
    T: ErasedTy,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(self, validator: ValValidator<U::Val>) -> Self {
        self.set_storer(ValStorer::from(validator))
    }
}

impl<'a, 'b, I, S, Ser, U, T> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T>
where
    S: Set,
    U: Infer + 'static,
    T: ErasedTy,
    U::Val: Clone + RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value(self, value: U::Val) -> Self {
        self.set_initializer(ValInitializer::new_value(value))
    }

    /// Set the option default value.
    pub fn set_values(self, value: Vec<U::Val>) -> Self {
        self.set_initializer(ValInitializer::new_values(value))
    }
}

impl<'a, 'b, I, S, Ser, U, T> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T>
where
    S: Set,
    U: Infer + 'static,
    T: ErasedTy + RawValParser,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator_t(mut self, validator: ValValidator<T>) -> Self {
        self.cfg_mut()
            .set_storer(ValStorer::new_validator(validator));
        self
    }

    /// Add default [`storer`](ValStorer::fallback) of type `T`.
    pub fn add_default_storer_t(self) -> Self {
        self.set_storer(ValStorer::fallback::<T>())
    }
}

impl<'a, 'b, I, S, Ser, U, T> ParserCommitWithValue<'a, 'b, I, S, Ser, U, T>
where
    S: Set,
    U: Infer + 'static,
    T: ErasedTy + Clone,
    U::Val: RawValParser,
    I: HandlerCollection<'a, S, Ser>,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    /// Set the option default value.
    pub fn set_value_t(self, value: T) -> Self {
        self.set_initializer(ValInitializer::new_value(value))
    }

    /// Set the option default value.
    pub fn set_values_t(self, value: Vec<T>) -> Self {
        self.set_initializer(ValInitializer::new_values(value))
    }

    /// Add a default [`initializer`](ValInitializer::fallback).
    pub fn add_default_initializer_t(self) -> Self {
        self.set_initializer(ValInitializer::fallback())
    }
}
