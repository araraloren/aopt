use std::fmt::Display;

use crate::args::Args;
use crate::opt::Style;
use crate::parser::Action;
use crate::parser::ReturnVal;
use crate::ARef;
use crate::AStr;
use crate::Error;
use crate::RawVal;
use crate::Uid;

#[derive(Debug, Clone, Default)]
pub struct InnerCtx {
    uid: Uid,

    name: Option<AStr>,

    style: Style,

    arg: Option<RawVal>,

    index: usize,

    total: usize,
}

impl InnerCtx {
    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = uid;
        self
    }

    pub fn with_idx(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn with_total(mut self, total: usize) -> Self {
        self.total = total;
        self
    }

    pub fn with_name(mut self, name: Option<AStr>) -> Self {
        self.name = name;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_arg(mut self, argument: Option<RawVal>) -> Self {
        self.arg = argument;
        self
    }

    /// The uid of matched option.
    pub fn uid(&self) -> Uid {
        self.uid
    }

    /// The index of matched option.
    pub fn idx(&self) -> usize {
        self.index
    }

    /// The total number of arguments.
    pub fn total(&self) -> usize {
        self.total
    }

    /// The name of matched option.
    /// For option it is the option name, for NOA it is the argument,
    /// which set in [`invoke`](crate::guess::InvokeGuess#method.invoke).
    pub fn name(&self) -> Option<&AStr> {
        self.name.as_ref()
    }

    /// The style of matched option.
    pub fn style(&self) -> Style {
        self.style
    }

    /// The argument which set in [`invoke`](crate::guess::InvokeGuess#method.invoke).
    pub fn arg(&self) -> Option<RawVal> {
        self.arg.clone()
    }

    pub fn set_uid(&mut self, uid: Uid) -> &mut Self {
        self.uid = uid;
        self
    }

    /// The index of matching context.
    pub fn set_index(&mut self, index: usize) -> &mut Self {
        self.index = index;
        self
    }

    /// The total of matching context.
    pub fn set_total(&mut self, total: usize) -> &mut Self {
        self.total = total;
        self
    }

    pub fn set_name(&mut self, name: Option<AStr>) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_arg(&mut self, argument: Option<RawVal>) -> &mut Self {
        self.arg = argument;
        self
    }
}

impl Display for InnerCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InnerCtx {{ uid: {}, name: {}, style: {}, arg: {}, index: {}, total: {} }}",
            self.uid,
            crate::display_option(&self.name),
            self.style,
            crate::display_option(&self.arg),
            self.index,
            self.total,
        )
    }
}

/// The invoke context of option handler.
/// It saved the option information and matched arguments.
#[derive(Debug, Clone, Default)]
pub struct Ctx {
    args: ARef<Args>,

    orig_args: ARef<Args>,

    inner_ctx: Option<InnerCtx>,

    #[cfg(not(feature = "sync"))]
    action: ARef<std::cell::RefCell<Option<Action>>>,

    #[cfg(feature = "sync")]
    action: ARef<std::sync::Mutex<Option<Action>>>,
}

impl Ctx {
    pub fn with_args(mut self, args: ARef<Args>) -> Self {
        self.args = args;
        self
    }

    pub fn with_orig_args(mut self, orig_args: ARef<Args>) -> Self {
        self.orig_args = orig_args;
        self
    }

    pub fn with_inner_ctx(mut self, inner_ctx: InnerCtx) -> Self {
        self.inner_ctx = Some(inner_ctx);
        self
    }
}

impl Ctx {
    /// The uid of matched option.
    pub fn uid(&self) -> Result<Uid, Error> {
        Ok(self.inner_ctx()?.uid())
    }

    /// The index of matched option.
    pub fn idx(&self) -> Result<usize, Error> {
        Ok(self.inner_ctx()?.idx())
    }

    /// The total number of arguments.
    pub fn total(&self) -> Result<usize, Error> {
        Ok(self.inner_ctx()?.total())
    }

    /// The name of matched option.
    /// For option it is the option name, for NOA it is the argument,
    /// which set in [`invoke`](crate::guess::InvokeGuess#method.invoke).
    pub fn name(&self) -> Result<Option<&AStr>, Error> {
        Ok(self.inner_ctx()?.name())
    }

    /// The style of matched option.
    pub fn style(&self) -> Result<Style, Error> {
        Ok(self.inner_ctx()?.style())
    }

    /// The copy of [`Args`] when the option matched.
    /// It may be changing during parsing process.
    pub fn args(&self) -> &ARef<Args> {
        &self.args
    }

    /// The argument which set in [`invoke`](crate::guess::InvokeGuess#method.invoke).
    pub fn arg(&self) -> Result<Option<RawVal>, Error> {
        Ok(self.inner_ctx()?.arg())
    }

    pub fn inner_ctx(&self) -> Result<&InnerCtx, Error> {
        self.inner_ctx.as_ref().ok_or_else(|| {
            crate::raise_error!("InnerCtx(read only) not exist, try create a new one")
        })
    }

    pub fn inner_ctx_mut(&mut self) -> Result<&mut InnerCtx, Error> {
        self.inner_ctx
            .as_mut()
            .ok_or_else(|| crate::raise_error!("InnerCtx(mutable) not exist, try create a new one"))
    }

    /// The original arguments passed by user.
    pub fn orig_args(&self) -> &ARef<Args> {
        &self.orig_args
    }

    /// The current argument indexed by `self.idx()`.
    pub fn curr_arg(&self) -> Result<Option<&RawVal>, Error> {
        let idx = self.idx()?;
        Ok((idx > 0).then(|| self.orig_args().get(idx)).flatten())
    }

    pub fn take_args(&mut self) -> ARef<Args> {
        std::mem::take(&mut self.args)
    }

    pub fn take_orig_args(&mut self) -> ARef<Args> {
        std::mem::take(&mut self.orig_args)
    }
}

impl Ctx {
    pub fn set_uid(&mut self, uid: Uid) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_uid(uid);
        Ok(self)
    }

    /// The index of matching context.
    pub fn set_index(&mut self, index: usize) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_index(index);
        Ok(self)
    }

    /// The total of matching context.
    pub fn set_total(&mut self, total: usize) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_total(total);
        Ok(self)
    }

    pub fn set_args(&mut self, args: ARef<Args>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn set_name(&mut self, name: Option<AStr>) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_name(name);
        Ok(self)
    }

    pub fn set_style(&mut self, style: Style) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_style(style);
        Ok(self)
    }

    pub fn set_arg(&mut self, argument: Option<RawVal>) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_arg(argument);
        Ok(self)
    }

    pub fn set_orig_args(&mut self, orig_args: ARef<Args>) -> &mut Self {
        self.orig_args = orig_args;
        self
    }

    pub fn set_inner_ctx(&mut self, inner_ctx: Option<InnerCtx>) -> &mut Self {
        crate::trace!("Switching InnerCtx to {:?}", inner_ctx);
        self.inner_ctx = inner_ctx;
        self
    }
}

impl Ctx {
    #[cfg(not(feature = "sync"))]
    pub fn policy_act(&self) -> Option<Action> {
        *self.action.borrow()
    }

    #[cfg(feature = "sync")]
    pub fn policy_act(&self) -> Option<Action> {
        *self.action.lock().unwrap()
    }

    #[cfg(not(feature = "sync"))]
    pub fn set_policy_act(&self, act: Action) {
        *self.action.borrow_mut() = Some(act);
    }

    #[cfg(feature = "sync")]
    pub fn set_policy_act(&self, act: Action) {
        *self.action.lock().unwrap() = Some(act);
    }

    #[cfg(not(feature = "sync"))]
    pub fn reset_policy_act(&self) {
        *self.action.borrow_mut() = None;
    }

    #[cfg(feature = "sync")]
    pub fn reset_policy_act(&self) {
        *self.action.lock().unwrap() = None;
    }
}

impl From<ReturnVal> for Ctx {
    fn from(mut value: ReturnVal) -> Self {
        value.take_ctx()
    }
}

impl<'a> From<&'a ReturnVal> for Ctx {
    fn from(value: &'a ReturnVal) -> Self {
        value.ctx().clone()
    }
}

impl<'a> From<&'a mut ReturnVal> for Ctx {
    fn from(value: &'a mut ReturnVal) -> Self {
        value.take_ctx()
    }
}
