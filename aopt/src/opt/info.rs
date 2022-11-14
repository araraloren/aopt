use super::Index;
use crate::Str;

pub trait Information {
    fn has_name(&self) -> bool;

    fn has_pre(&self) -> bool;

    fn has_opt(&self) -> bool;

    fn has_ty(&self) -> bool;

    fn has_idx(&self) -> bool;

    fn has_deact(&self) -> bool;

    fn name(&self) -> Option<&Str>;

    fn pre(&self) -> Option<&Str>;

    fn opt(&self) -> Option<bool>;

    fn ty(&self) -> Option<&Str>;

    fn idx(&self) -> Option<&Index>;

    fn deact(&self) -> Option<bool>;

    fn take_name(&mut self) -> Option<Str>;

    fn take_pre(&mut self) -> Option<Str>;

    fn take_opt(&mut self) -> Option<bool>;

    fn take_ty(&mut self) -> Option<Str>;

    fn take_idx(&mut self) -> Option<Index>;

    fn take_deact(&mut self) -> Option<bool>;
}

/// Parsing result of option constructor string.
#[derive(Debug, Default)]
pub struct ConstrctInfo {
    pub pattern: Str,

    pub prefix: Option<Str>,

    pub name: Option<Str>,

    pub type_name: Option<Str>,

    pub deactivate: Option<bool>,

    pub optional: Option<bool>,

    pub forward_index: Option<usize>,

    pub backward_index: Option<usize>,

    pub anywhere: Option<bool>,

    pub list: Vec<usize>,

    pub except: Vec<usize>,

    pub range: Option<(Option<usize>, Option<usize>)>,

    index: Option<Index>,
}

impl ConstrctInfo {
    pub fn with_pat(mut self, pattern: Str) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn with_pre(mut self, prefix: Option<Str>) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_name(mut self, name: Option<Str>) -> Self {
        self.name = name;
        self
    }

    pub fn with_ty(mut self, type_name: Option<Str>) -> Self {
        self.type_name = type_name;
        self
    }

    pub fn with_deact(mut self, deactivate: Option<bool>) -> Self {
        self.deactivate = deactivate;
        self
    }

    pub fn with_opt(mut self, optional: Option<bool>) -> Self {
        self.optional = optional;
        self
    }

    pub fn with_fwd(mut self, forward_index: Option<usize>) -> Self {
        self.forward_index = forward_index;
        self
    }

    pub fn with_bwd(mut self, backward_index: Option<usize>) -> Self {
        self.backward_index = backward_index;
        self
    }

    pub fn with_aw(mut self, anywhere: Option<bool>) -> Self {
        self.anywhere = anywhere;
        self
    }

    pub fn with_ls(mut self, list: Vec<usize>) -> Self {
        self.list = list;
        self
    }

    pub fn with_exp(mut self, except: Vec<usize>) -> Self {
        self.except = except;
        self
    }

    pub fn with_range(mut self, range: Option<(Option<usize>, Option<usize>)>) -> Self {
        self.range = range;
        self
    }

    pub fn gen_idx(&mut self) {
        if self.has_idx() {
            self.index = if self.forward_index.is_some() {
                Some(Index::Forward(self.forward_index.unwrap()))
            } else if self.backward_index.is_some() {
                Some(Index::Backward(self.backward_index.unwrap()))
            } else if self.anywhere.unwrap_or(false) {
                Some(Index::AnyWhere)
            } else if !self.list.is_empty() {
                Some(Index::List(std::mem::take(&mut self.list)))
            } else if !self.except.is_empty() {
                Some(Index::Except(std::mem::take(&mut self.except)))
            } else if self.range.is_some() {
                if let Some(range) = self.range {
                    Some(Index::range(range.0, range.1))
                } else {
                    panic!("Can not unwrap data from Some ?!!")
                }
            } else {
                None
            };
        } else {
            self.index = None;
        }
    }
}

impl Information for ConstrctInfo {
    fn has_name(&self) -> bool {
        self.name.is_some()
    }

    fn has_pre(&self) -> bool {
        self.prefix.is_some()
    }

    fn has_opt(&self) -> bool {
        self.optional.is_some()
    }

    fn has_ty(&self) -> bool {
        self.type_name.is_some()
    }

    fn has_idx(&self) -> bool {
        self.forward_index.is_some()
            || self.backward_index.is_some()
            || self.anywhere.is_some()
            || !self.list.is_empty()
            || !self.except.is_empty()
            || self.range.is_some()
    }

    fn has_deact(&self) -> bool {
        self.deactivate.is_some()
    }

    fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    fn pre(&self) -> Option<&Str> {
        self.prefix.as_ref()
    }

    fn opt(&self) -> Option<bool> {
        self.optional
    }

    fn ty(&self) -> Option<&Str> {
        self.type_name.as_ref()
    }

    fn idx(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn deact(&self) -> Option<bool> {
        self.deactivate
    }

    fn take_name(&mut self) -> Option<Str> {
        self.name.take()
    }

    fn take_pre(&mut self) -> Option<Str> {
        self.prefix.take()
    }

    fn take_opt(&mut self) -> Option<bool> {
        self.optional.take()
    }

    fn take_ty(&mut self) -> Option<Str> {
        self.type_name.take()
    }

    fn take_idx(&mut self) -> Option<Index> {
        self.index.take()
    }

    fn take_deact(&mut self) -> Option<bool> {
        self.deactivate.take()
    }
}
