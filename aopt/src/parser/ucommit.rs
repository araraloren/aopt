use std::fmt::Debug;

use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::HandlerEntry;
use crate::ctx::Invoker;
use crate::map::ErasedTy;
use crate::opt::Action;
use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::opt::Opt;
use crate::set::SetCfg;
use crate::set::SetOpt;
use crate::set::UCommit;
use crate::value::Infer;
use crate::value::RawValParser;
use crate::value::ValInitializer;
use crate::value::ValValidator;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Simple wrapped the option create interface of [`TyCommit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct UParserCommit<'a, Set, Ser, U>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    inner: UCommit<'a, Set, U>,

    inv_ser: Option<&'a mut Invoker<Set, Ser>>,
}

impl<'a, Set, Ser, U> Debug for UParserCommit<'a, Set, Ser, U>
where
    Set: crate::set::Set + Debug,
    SetOpt<Set>: Opt + Debug,
    Ser: Debug,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<Set>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserCommit")
            .field("inner", &self.inner)
            .field("inv_ser", &self.inv_ser)
            .finish()
    }
}

impl<'a, Set, Ser, U> UParserCommit<'a, Set, Ser, U>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn new(inner: UCommit<'a, Set, U>, inv_ser: &'a mut Invoker<Set, Ser>) -> Self {
        Self {
            inner,
            inv_ser: Some(inv_ser),
        }
    }

    pub fn cfg(&self) -> &SetCfg<Set> {
        self.inner.cfg()
    }

    pub fn cfg_mut(&mut self) -> &mut SetCfg<Set> {
        self.inner.cfg_mut()
    }

    /// Set the option index of commit configuration.
    pub fn set_idx(mut self, index: Index) -> Self {
        self.cfg_mut().set_idx(index);
        self
    }

    /// Set the option value action.
    pub fn set_action(mut self, action: Action) -> Self {
        self.cfg_mut().set_action(action);
        self
    }

    /// Set the option name of commit configuration.
    pub fn set_name<T: Into<Str>>(mut self, name: T) -> Self {
        self.cfg_mut().set_name(name);
        self
    }

    /// Set the option type name of commit configuration.
    pub fn set_type(mut self) -> Self {
        self.cfg_mut().set_type::<U::Val>();
        self
    }

    /// Clear all the alias of commit configuration.
    pub fn clr_alias(mut self) -> Self {
        self.cfg_mut().clr_alias();
        self
    }

    /// Remove the given alias of commit configuration.
    pub fn rem_alias<T: Into<Str>>(mut self, alias: T) -> Self {
        self.cfg_mut().rem_alias(alias);
        self
    }

    /// Add given alias into the commit configuration.
    pub fn add_alias<T: Into<Str>>(mut self, alias: T) -> Self {
        self.cfg_mut().add_alias(alias);
        self
    }

    /// Set the option optional of commit configuration.
    pub fn set_force(mut self, force: bool) -> Self {
        self.cfg_mut().set_force(force);
        self
    }

    /// Set the option hint message of commit configuration.
    pub fn set_hint<T: Into<Str>>(mut self, hint: T) -> Self {
        self.cfg_mut().set_hint(hint);
        self
    }

    /// Set the option help message of commit configuration.
    pub fn set_help<T: Into<Str>>(mut self, help: T) -> Self {
        self.cfg_mut().set_help(help);
        self
    }

    /// Set the option value initiator.
    pub fn set_initializer(mut self, initializer: ValInitializer) -> Self {
        self.inner.initializer = Some(initializer);
        self
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
        self.inner.run_and_commit_the_change()
    }

    /// Run the commit.
    ///
    /// It create an option using given type [`Ctor`](crate::set::Ctor).
    /// And add it to referenced [`Set`](crate::set::Set), return the new option [`Uid`].
    pub fn run(mut self) -> Result<Uid, Error> {
        self.run_and_commit_the_change()
    }
}

impl<'a, Set, Ser, U> UParserCommit<'a, Set, Ser, U>
where
    U: Infer,
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U::Val: RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Set the option value validator.
    pub fn set_validator(mut self, validator: ValValidator<U::Val>) -> Self {
        self.inner.validator = Some(validator);
        self
    }
}

impl<'a, Set, Ser, U> UParserCommit<'a, Set, Ser, U>
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

impl<'a, Set, Ser, U> UParserCommit<'a, Set, Ser, U>
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

impl<'a, Set, Ser, U> Drop for UParserCommit<'a, Set, Ser, U>
where
    Set: crate::set::Set,
    SetOpt<Set>: Opt,
    U: Infer,
    U::Val: RawValParser,
    SetCfg<Set>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.inner.drop_commit {
            let error =
                "Error when commit the option in ParserCommit::Drop, call `run` get the Result";

            self.run_and_commit_the_change().expect(error);
        }
    }
}
