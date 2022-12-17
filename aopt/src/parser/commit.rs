use std::fmt::Debug;

use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::opt::Action;
use crate::opt::Assoc;
use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::opt::Opt;
use crate::opt::ValInitiator;
use crate::opt::ValValidator;
use crate::prelude::InvokeService;
use crate::ser::invoke::HandlerEntry;
use crate::set::Commit;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetOpt;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Simple wrapped the option create interface of [`Commit`],
/// and the handler register interface of [`HandlerEntry`].
pub struct ParserCommit<'a, S>
where
    S: Set,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    inner: Commit<'a, S>,

    inv_ser: Option<&'a mut InvokeService<S>>,

    drop_commit: bool,
}

impl<'a, S> Debug for ParserCommit<'a, S>
where
    S: Set + Debug,
    SetOpt<S>: Opt + Debug,
    SetCfg<S>: ConfigValue + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserCommit")
            .field("inner", &self.inner)
            .field("inv_ser", &self.inv_ser)
            .field("drop_commit", &self.drop_commit)
            .finish()
    }
}

impl<'a, S> ParserCommit<'a, S>
where
    S: Set,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    pub fn new(inner: Commit<'a, S>, inv_ser: &'a mut InvokeService<S>) -> Self {
        Self {
            inner,
            inv_ser: Some(inv_ser),
            drop_commit: false,
        }
    }

    pub fn cfg(&self) -> &SetCfg<S> {
        self.inner.cfg()
    }

    pub fn cfg_mut(&mut self) -> &mut SetCfg<S> {
        self.inner.cfg_mut()
    }

    /// Set the option index of commit configuration.
    pub fn set_idx(mut self, index: Index) -> Self {
        self.cfg_mut().set_idx(index);
        self
    }

    /// Set the option value assoc type.
    pub fn set_assoc(mut self, assoc: Assoc) -> Self {
        self.cfg_mut().set_assoc(assoc);
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

    /// Set the option prefix of commit configuration.
    pub fn set_prefix<T: Into<Str>>(mut self, prefix: T) -> Self {
        self.cfg_mut().set_prefix(prefix);
        self
    }

    /// Set the option type name of commit configuration.
    pub fn set_type<T: Into<Str>>(mut self, type_name: T) -> Self {
        self.cfg_mut().set_type(type_name);
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
    pub fn set_optional(mut self, optional: bool) -> Self {
        self.cfg_mut().set_optional(optional);
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
    pub fn set_initiator(mut self, initiator: ValInitiator) -> Self {
        self.cfg_mut().set_initiator(Some(initiator));
        self
    }

    /// Set the option value validator.
    pub fn set_validator(mut self, validator: ValValidator) -> Self {
        self.cfg_mut().set_validator(Some(validator));
        self
    }

    /// Set the option deactivate style of commit configuration.
    pub fn set_deactivate(mut self, deactivate_style: bool) -> Self {
        self.cfg_mut().set_deactivate(deactivate_style);
        self
    }

    /// Set the option default value.
    pub fn set_value<T: Clone + 'static>(mut self, value: T) -> Self {
        self.cfg_mut()
            .set_initiator(Some(ValInitiator::with(vec![value])));
        self
    }

    /// Set the option default value.
    pub fn set_values<T: Clone + 'static>(mut self, value: Vec<T>) -> Self {
        self.cfg_mut()
            .set_initiator(Some(ValInitiator::with(value)));
        self
    }

    /// Register the handler which will be called when option is set.
    /// The function will register the option to [`Set`] first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn on<H, O, A>(mut self, handler: H) -> Result<HandlerEntry<'a, S, H, A, O>, Error>
    where
        O: 'static,
        H: Handler<S, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<S, Error = Error> + 'static,
    {
        let uid = self.run_and_commit_the_change()?;
        // we don't need &'a mut InvokeServices, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        self.drop_commit = false;
        Ok(HandlerEntry::new(ser.unwrap(), uid).on(handler))
    }

    /// Register the handler which will be called when option is set.
    /// And the [`fallback`](crate::ser::InvokeService::fallback) will be called if
    /// the handler return None.
    /// The function will register the option to [`Set`] first,
    /// then pass the unqiue id to [`HandlerEntry`].
    pub fn fallback<H, O, A>(mut self, handler: H) -> Result<HandlerEntry<'a, S, H, A, O>, Error>
    where
        O: 'static,
        H: Handler<S, A, Output = Option<O>, Error = Error> + 'static,
        A: Extract<S, Error = Error> + 'static,
    {
        let uid = self.run_and_commit_the_change()?;
        // we don't need &'a mut InvokeServices, so just take it.
        let ser = std::mem::take(&mut self.inv_ser);

        self.drop_commit = false;
        Ok(HandlerEntry::new(ser.unwrap(), uid).fallback(handler))
    }

    pub(crate) fn run_and_commit_the_change(&mut self) -> Result<Uid, Error> {
        self.drop_commit = false;
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

impl<'a, S> Drop for ParserCommit<'a, S>
where
    S: crate::set::Set,
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
{
    fn drop(&mut self) {
        if self.drop_commit {
            let error =
                "Error when commit the option in ParserCommit::Drop, call `run` get the Result";

            self.run_and_commit_the_change().expect(error);
        }
    }
}
