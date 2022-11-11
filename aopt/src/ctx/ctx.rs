use crate::args::Args;
use crate::opt::Style;
use crate::Arc;
use crate::RawVal;
use crate::Str;
use crate::Uid;

#[derive(Debug, Clone, Default)]
pub struct Ctx {
    uid: Uid,

    name: Option<Str>,

    prefix: Option<Str>,

    style: Style,

    disable: bool,

    arg: Option<Arc<RawVal>>,

    index: usize,

    total: usize,

    args: Arc<Args>,
}

impl Ctx {
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

    pub fn with_args(mut self, args: Arc<Args>) -> Self {
        self.args = args;
        self
    }

    pub fn with_name(mut self, name: Option<Str>) -> Self {
        self.name = name;
        self
    }

    pub fn with_prefix(mut self, prefix: Option<Str>) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_disable(mut self, disable: bool) -> Self {
        self.disable = disable;
        self
    }

    pub fn with_arg(mut self, arg: Option<Arc<RawVal>>) -> Self {
        self.arg = arg;
        self
    }
}

impl Ctx {
    pub fn uid(&self) -> Uid {
        self.uid
    }

    pub fn idx(&self) -> usize {
        self.index
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn args(&self) -> &Arc<Args> {
        &self.args
    }

    pub fn prefix(&self) -> Option<&Str> {
        self.prefix.as_ref()
    }

    pub fn disable(&self) -> bool {
        self.disable
    }

    pub fn arg(&self) -> Option<Arc<RawVal>> {
        self.arg.clone()
    }

    pub fn orig_arg(&self) -> Option<&RawVal> {
        (self.idx() > 0)
            .then(|| self.args().get(self.idx().saturating_sub(1)))
            .flatten()
    }
}

impl Ctx {
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

    pub fn set_args(&mut self, args: Arc<Args>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn set_name(&mut self, name: Option<Str>) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_prefix(&mut self, prefix: Option<Str>) -> &mut Self {
        self.prefix = prefix;
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_disable(&mut self, disable: bool) -> &mut Self {
        self.disable = disable;
        self
    }

    pub fn set_arg(&mut self, argument: Option<Arc<RawVal>>) -> &mut Self {
        self.arg = argument;
        self
    }
}
