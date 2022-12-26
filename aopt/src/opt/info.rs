use super::Index;
use crate::Str;

pub trait Information {
    fn has_name(&self) -> bool;

    fn has_force(&self) -> bool;

    fn has_ty(&self) -> bool;

    fn has_idx(&self) -> bool;

    fn name(&self) -> Option<&Str>;

    fn force(&self) -> Option<bool>;

    fn ty(&self) -> Option<&Str>;

    fn idx(&self) -> Option<&Index>;

    fn take_name(&mut self) -> Option<Str>;

    fn take_force(&mut self) -> Option<bool>;

    fn take_ty(&mut self) -> Option<Str>;

    fn take_idx(&mut self) -> Option<Index>;
}

/// Parsing result of option constructor string.
#[derive(Debug, Default)]
pub struct ConstrctInfo {
    pub pattern: Str,

    pub name: Option<Str>,

    pub type_name: Option<Str>,

    pub force: Option<bool>,

    pub index: Option<Index>,
}

impl ConstrctInfo {
    pub fn with_pat(mut self, pattern: Str) -> Self {
        self.pattern = pattern;
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

    pub fn with_force(mut self, force: Option<bool>) -> Self {
        self.force = force;
        self
    }

    pub fn with_index(mut self, index: Option<Index>) -> Self {
        self.index = index;
        self
    }
}

impl Information for ConstrctInfo {
    fn has_name(&self) -> bool {
        self.name.is_some()
    }

    fn has_force(&self) -> bool {
        self.force.is_some()
    }

    fn has_ty(&self) -> bool {
        self.type_name.is_some()
    }

    fn has_idx(&self) -> bool {
        self.index.is_some()
    }

    fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    fn force(&self) -> Option<bool> {
        self.force
    }

    fn ty(&self) -> Option<&Str> {
        self.type_name.as_ref()
    }

    fn idx(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn take_name(&mut self) -> Option<Str> {
        self.name.take()
    }

    fn take_force(&mut self) -> Option<bool> {
        self.force.take()
    }

    fn take_ty(&mut self) -> Option<Str> {
        self.type_name.take()
    }

    fn take_idx(&mut self) -> Option<Index> {
        self.index.take()
    }
}
