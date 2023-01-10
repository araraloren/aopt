use crate::args::Args;
use crate::opt::Style;
use crate::parser::ReturnVal;
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

#[derive(Debug, Clone, Default)]
pub struct InnerCtx {
    uid: Uid,

    name: Option<Str>,

    style: Style,

    arg: Option<Arc<RawVal>>,

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

    pub fn with_name(mut self, name: Option<Str>) -> Self {
        self.name = name;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_arg(mut self, argument: Option<Arc<RawVal>>) -> Self {
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
    /// which set in [`guess`](crate::parser::Guess::guess).
    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    /// The style of matched option.
    pub fn style(&self) -> Style {
        self.style
    }

    /// The argument which set in [`guess`](crate::parser::Guess::guess).
    pub fn arg(&self) -> Option<Arc<RawVal>> {
        self.arg.clone()
    }

    pub fn set_uid(&mut self, uid: Uid) -> &mut Self {
        self.uid = uid;
        self
    }

    /// The index of matching context.
    pub fn set_idx(&mut self, index: usize) -> &mut Self {
        self.index = index;
        self
    }

    /// The total of matching context.
    pub fn set_total(&mut self, total: usize) -> &mut Self {
        self.total = total;
        self
    }

    pub fn set_name(&mut self, name: Option<Str>) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_arg(&mut self, argument: Option<Arc<RawVal>>) -> &mut Self {
        self.arg = argument;
        self
    }
}

/// The invoke context of option handler.
/// It saved the option information and matched arguments.
#[derive(Debug, Clone, Default)]
pub struct Ctx {
    args: Arc<Args>,

    orig_args: Arc<Args>,

    inner_ctx: Option<InnerCtx>,
}

impl Ctx {
    pub fn with_args(mut self, args: Arc<Args>) -> Self {
        self.args = args;
        self
    }

    pub fn with_orig_args(mut self, orig_args: Arc<Args>) -> Self {
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
    /// which set in [`guess`](crate::parser::Guess::guess).
    pub fn name(&self) -> Result<Option<&Str>, Error> {
        Ok(self.inner_ctx()?.name())
    }

    /// The style of matched option.
    pub fn style(&self) -> Result<Style, Error> {
        Ok(self.inner_ctx()?.style())
    }

    /// The copy of [`Args`] when the option matched.
    /// It may be changing during parsing process.
    pub fn args(&self) -> &Arc<Args> {
        &self.args
    }

    /// The argument which set in [`guess`](crate::parser::Guess::guess).
    pub fn arg(&self) -> Result<Option<Arc<RawVal>>, Error> {
        Ok(self.inner_ctx()?.arg())
    }

    pub fn inner_ctx(&self) -> Result<&InnerCtx, Error> {
        self.inner_ctx.as_ref().ok_or_else(|| {
            Error::raise_error("InnerCtx(read only) not exist, try create a new one")
        })
    }

    pub fn inner_ctx_mut(&mut self) -> Result<&mut InnerCtx, Error> {
        self.inner_ctx
            .as_mut()
            .ok_or_else(|| Error::raise_error("InnerCtx(mutable) not exist, try create a new one"))
    }

    /// The original arguments passed by user.
    pub fn orig_args(&self) -> &Arc<Args> {
        &self.orig_args
    }

    /// The current argument indexed by `self.idx()`.
    pub fn curr_arg(&self) -> Result<Option<&RawVal>, Error> {
        let idx = self.idx()?;
        Ok((idx > 0).then(|| self.orig_args().get(idx)).flatten())
    }

    pub fn take_args(&mut self) -> Arc<Args> {
        std::mem::take(&mut self.args)
    }

    pub fn take_orig_args(&mut self) -> Arc<Args> {
        std::mem::take(&mut self.orig_args)
    }
}

impl Ctx {
    pub fn set_uid(&mut self, uid: Uid) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_uid(uid);
        Ok(self)
    }

    /// The index of matching context.
    pub fn set_idx(&mut self, index: usize) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_idx(index);
        Ok(self)
    }

    /// The total of matching context.
    pub fn set_total(&mut self, total: usize) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_total(total);
        Ok(self)
    }

    pub fn set_args(&mut self, args: Arc<Args>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn set_name(&mut self, name: Option<Str>) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_name(name);
        Ok(self)
    }

    pub fn set_style(&mut self, style: Style) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_style(style);
        Ok(self)
    }

    pub fn set_arg(&mut self, argument: Option<Arc<RawVal>>) -> Result<&mut Self, Error> {
        self.inner_ctx_mut()?.set_arg(argument);
        Ok(self)
    }

    pub fn set_orig_args(&mut self, orig_args: Arc<Args>) -> &mut Self {
        self.orig_args = orig_args;
        self
    }

    pub fn set_inner_ctx(&mut self, inner_ctx: Option<InnerCtx>) -> &mut Self {
        self.inner_ctx = inner_ctx;
        self
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
