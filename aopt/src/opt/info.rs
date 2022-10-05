use super::OptIndex;
use crate::Str;

pub trait Information {
    fn has_name(&self) -> bool;

    fn has_prefix(&self) -> bool;

    fn has_optional(&self) -> bool;

    fn has_type_name(&self) -> bool;

    fn has_index(&self) -> bool;

    fn has_deactivate_style(&self) -> bool;

    fn get_name(&self) -> Option<Str>;

    fn get_prefix(&self) -> Option<Str>;

    fn get_optional(&self) -> Option<bool>;

    fn get_type_name(&self) -> Option<Str>;

    fn get_index(&self) -> Option<&OptIndex>;

    fn get_deactivate_style(&self) -> Option<bool>;

    fn take_name(&mut self) -> Option<Str>;

    fn take_prefix(&mut self) -> Option<Str>;

    fn take_optional(&mut self) -> Option<bool>;

    fn take_type_name(&mut self) -> Option<Str>;

    fn take_index(&mut self) -> Option<OptIndex>;

    fn take_deactivate_style(&mut self) -> Option<bool>;
}

/// Parsing result of option constructor string.
#[derive(Debug, Default)]
pub struct OptConstrctInfo {
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

    pub greater: Option<usize>,

    pub less: Option<usize>,

    index: Option<OptIndex>,
}

impl OptConstrctInfo {
    pub fn with_pattern(mut self, pattern: Str) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn with_prefix(mut self, prefix: Option<Str>) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_name(mut self, name: Option<Str>) -> Self {
        self.name = name;
        self
    }

    pub fn with_type_name(mut self, type_name: Option<Str>) -> Self {
        self.type_name = type_name;
        self
    }

    pub fn with_deactivate(mut self, deactivate: Option<bool>) -> Self {
        self.deactivate = deactivate;
        self
    }

    pub fn with_optional(mut self, optional: Option<bool>) -> Self {
        self.optional = optional;
        self
    }

    pub fn with_forward(mut self, forward_index: Option<usize>) -> Self {
        self.forward_index = forward_index;
        self
    }

    pub fn with_backward(mut self, backward_index: Option<usize>) -> Self {
        self.backward_index = backward_index;
        self
    }

    pub fn with_anywhere(mut self, anywhere: Option<bool>) -> Self {
        self.anywhere = anywhere;
        self
    }

    pub fn with_list(mut self, list: Vec<usize>) -> Self {
        self.list = list;
        self
    }

    pub fn with_except(mut self, except: Vec<usize>) -> Self {
        self.except = except;
        self
    }

    pub fn with_greater(mut self, greater: Option<usize>) -> Self {
        self.greater = greater;
        self
    }

    pub fn with_less(mut self, less: Option<usize>) -> Self {
        self.less = less;
        self
    }

    pub fn gen_index(&mut self) {
        if self.has_index() {
            self.index = if self.forward_index.is_some() {
                Some(OptIndex::Forward(self.forward_index.unwrap()))
            } else if self.backward_index.is_some() {
                Some(OptIndex::Backward(self.backward_index.unwrap()))
            } else if self.anywhere.unwrap_or(false) {
                Some(OptIndex::AnyWhere)
            } else if !self.list.is_empty() {
                Some(OptIndex::List(std::mem::take(&mut self.list)))
            } else if !self.except.is_empty() {
                Some(OptIndex::Except(std::mem::take(&mut self.except)))
            } else if self.greater.is_some() {
                Some(OptIndex::Greater(self.greater.unwrap()))
            } else if self.less.is_some() {
                Some(OptIndex::Less(self.less.unwrap()))
            } else {
                None
            };
        } else {
            self.index = None;
        }
    }
}

impl Information for OptConstrctInfo {
    fn has_name(&self) -> bool {
        self.name.is_some()
    }

    fn has_prefix(&self) -> bool {
        self.prefix.is_some()
    }

    fn has_optional(&self) -> bool {
        self.optional.is_some()
    }

    fn has_type_name(&self) -> bool {
        self.type_name.is_some()
    }

    fn has_index(&self) -> bool {
        self.forward_index.is_some()
            || self.backward_index.is_some()
            || self.anywhere.is_some()
            || !self.list.is_empty()
            || !self.except.is_empty()
            || self.greater.is_some()
            || self.less.is_some()
    }

    fn has_deactivate_style(&self) -> bool {
        self.deactivate.is_some()
    }

    fn get_name(&self) -> Option<Str> {
        self.name.clone()
    }

    fn get_prefix(&self) -> Option<Str> {
        self.prefix.clone()
    }

    fn get_optional(&self) -> Option<bool> {
        self.optional
    }

    fn get_type_name(&self) -> Option<Str> {
        self.type_name.clone()
    }

    fn get_index(&self) -> Option<&OptIndex> {
        self.index.as_ref()
    }

    fn get_deactivate_style(&self) -> Option<bool> {
        self.deactivate
    }

    fn take_name(&mut self) -> Option<Str> {
        self.name.take()
    }

    fn take_prefix(&mut self) -> Option<Str> {
        self.prefix.take()
    }

    fn take_optional(&mut self) -> Option<bool> {
        self.optional.take()
    }

    fn take_type_name(&mut self) -> Option<Str> {
        self.type_name.take()
    }

    fn take_index(&mut self) -> Option<OptIndex> {
        self.index.take()
    }

    fn take_deactivate_style(&mut self) -> Option<bool> {
        self.deactivate.take()
    }
}
