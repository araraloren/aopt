use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::Display;

use crate::args::Args;
use crate::opt::Style;
use crate::parser::Action;
use crate::str::display_of_osstr;
use crate::str::display_of_str;
use crate::value::RawValParser;
use crate::Error;
use crate::Uid;

#[derive(Debug, Clone, Default)]
pub struct InnerCtx<'a> {
    uid: Uid,

    name: Option<Cow<'a, str>>,

    style: Style,

    arg: Option<Cow<'a, OsStr>>,

    index: usize,

    total: usize,
}

impl<'a> InnerCtx<'a> {
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

    pub fn with_name(mut self, name: Option<Cow<'a, str>>) -> Self {
        self.name = name;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_arg(mut self, arg: Option<Cow<'a, OsStr>>) -> Self {
        self.arg = arg;
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
    pub fn name(&self) -> Option<&Cow<'a, str>> {
        self.name.as_ref()
    }

    /// The style of matched option.
    pub fn style(&self) -> Style {
        self.style
    }

    /// The argument which set in [`invoke`](crate::guess::InvokeGuess#method.invoke).
    pub fn arg(&self) -> Option<&Cow<'a, OsStr>> {
        self.arg.as_ref()
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

    pub fn set_name(&mut self, name: Option<Cow<'a, str>>) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_arg(&mut self, arg: Option<Cow<'a, OsStr>>) -> &mut Self {
        self.arg = arg;
        self
    }
}

impl Display for InnerCtx<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InnerCtx {{ uid: {}, name: {}, style: {}, arg: {}, index: {}, total: {} }}",
            self.uid,
            display_of_str(self.name.as_deref()),
            self.style,
            display_of_osstr(self.arg.as_deref()),
            self.index,
            self.total,
        )
    }
}

/// The invoke context of option handler.
/// It saved the option information and matched arguments.
#[derive(Debug, Default)]
pub struct Ctx<'a> {
    pub(crate) orig: Args,

    pub(crate) args: Vec<&'a OsStr>,

    pub(crate) inner_ctx: Option<InnerCtx<'a>>,

    #[cfg(not(feature = "sync"))]
    action: std::cell::RefCell<Action>,

    #[cfg(feature = "sync")]
    action: std::sync::Mutex<Action>,
}

impl Clone for Ctx<'_> {
    fn clone(&self) -> Self {
        Self {
            orig: self.orig.clone(),
            args: self.args.clone(),
            inner_ctx: self.inner_ctx.clone(),
            #[cfg(not(feature = "sync"))]
            action: self.action.clone(),
            #[cfg(feature = "sync")]
            action: std::sync::Mutex::new(*self.action.lock().unwrap()),
        }
    }
}

impl<'a> Ctx<'a> {
    pub fn with_args(mut self, args: Vec<&'a OsStr>) -> Self {
        self.args = args;
        self
    }

    pub fn with_orig(mut self, orig_args: Args) -> Self {
        self.orig = orig_args;
        self
    }

    pub fn with_inner_ctx(mut self, inner_ctx: InnerCtx<'a>) -> Self {
        self.inner_ctx = Some(inner_ctx);
        self
    }
}

impl<'a> Ctx<'a> {
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
    pub fn name(&self) -> Result<Option<&Cow<'a, str>>, Error> {
        Ok(self.inner_ctx()?.name())
    }

    /// The style of matched option.
    pub fn style(&self) -> Result<Style, Error> {
        Ok(self.inner_ctx()?.style())
    }

    /// The copy of [`Args`] when the option matched.
    /// It may be changing during parsing process.
    pub fn args(&self) -> &[&'a OsStr] {
        &self.args
    }

    /// The argument which set in [`invoke`](crate::guess::InvokeGuess#method.invoke).
    pub fn arg(&self) -> Result<Option<&Cow<'a, OsStr>>, Error> {
        Ok(self.inner_ctx()?.arg())
    }

    pub fn inner_ctx(&self) -> Result<&InnerCtx<'a>, Error> {
        self.inner_ctx
            .as_ref()
            .ok_or_else(|| crate::raise_error!("InnerCtx not exist, try create a new one"))
    }

    pub fn inner_ctx_mut(&mut self) -> Result<&mut InnerCtx<'a>, Error> {
        self.inner_ctx
            .as_mut()
            .ok_or_else(|| crate::raise_error!("InnerCtx(mutable) not exist, try create a new one"))
    }

    /// The original arguments passed by user.
    pub fn orig(&self) -> &Args {
        &self.orig
    }

    /// The current argument indexed by `self.idx()`.
    pub fn arg_at(&self, idx: usize) -> Result<Option<&'a OsStr>, Error> {
        Ok(self.args.get(idx).copied())
    }

    pub fn take_args(&mut self) -> Vec<&OsStr> {
        std::mem::take(&mut self.args)
    }

    pub fn take_orig_args(&mut self) -> Args {
        std::mem::take(&mut self.orig)
    }
}

impl<'a> Ctx<'a> {
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

    pub fn set_args(&mut self, args: Vec<&'a OsStr>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn set_name(&mut self, name: Option<Cow<'a, str>>) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_name(name);
        Ok(self)
    }

    pub fn set_style(&mut self, style: Style) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_style(style);
        Ok(self)
    }

    pub fn set_arg(&mut self, arg: Option<Cow<'a, OsStr>>) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_arg(arg);
        Ok(self)
    }

    pub fn set_orig_args(&mut self, orig_args: Args) -> &mut Self {
        self.orig = orig_args;
        self
    }

    pub fn set_inner_ctx(&mut self, inner_ctx: Option<InnerCtx<'a>>) -> &mut Self {
        crate::trace!("switching InnerCtx to {:?}", inner_ctx);
        self.inner_ctx = inner_ctx;
        self
    }
}

impl Ctx<'_> {
    #[cfg(not(feature = "sync"))]
    pub fn policy_act(&self) -> Action {
        *self.action.borrow()
    }

    #[cfg(feature = "sync")]
    pub fn policy_act(&self) -> Action {
        *self.action.lock().unwrap()
    }

    #[cfg(not(feature = "sync"))]
    pub fn set_policy_act(&self, act: Action) {
        *self.action.borrow_mut() = act;
    }

    #[cfg(feature = "sync")]
    pub fn set_policy_act(&self, act: Action) {
        *self.action.lock().unwrap() = act;
    }

    #[cfg(not(feature = "sync"))]
    pub fn reset_policy_act(&self) {
        *self.action.borrow_mut() = Action::Null;
    }

    #[cfg(feature = "sync")]
    pub fn reset_policy_act(&self) {
        *self.action.lock().unwrap() = Action::Null;
    }
}

impl Ctx<'_> {
    pub fn value<T: RawValParser>(&self) -> Result<T, Error> {
        let arg = self.arg()?.map(|v| v.as_ref());
        let uid = self.uid()?;

        T::parse(arg, self).map_err(|e| e.into().with_uid(uid))
    }
}
