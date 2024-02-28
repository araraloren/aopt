use super::Index;
use crate::AStr;

pub trait Information {
    fn has_name(&self) -> bool;

    fn has_force(&self) -> bool;

    fn has_alias(&self) -> bool;

    fn has_index(&self) -> bool;

    fn has_help(&self) -> bool;

    fn has_ctor(&self) -> bool;

    fn name(&self) -> Option<&AStr>;

    fn force(&self) -> Option<bool>;

    fn alias(&self) -> Option<&Vec<AStr>>;

    fn index(&self) -> Option<&Index>;

    fn help(&self) -> Option<&AStr>;

    fn ctor(&self) -> Option<&AStr>;

    fn take_name(&mut self) -> Option<AStr>;

    fn take_force(&mut self) -> Option<bool>;

    fn take_alias(&mut self) -> Option<Vec<AStr>>;

    fn take_index(&mut self) -> Option<Index>;

    fn take_help(&mut self) -> Option<AStr>;

    fn take_ctor(&mut self) -> Option<AStr>;
}

/// Parsing result of option constructor string.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstrctInfo {
    pub(crate) name: Option<AStr>,

    pub(crate) alias: Option<Vec<AStr>>,

    pub(crate) force: Option<bool>,

    pub(crate) index: Option<Index>,

    pub(crate) help: Option<AStr>,

    pub(crate) ctor: Option<AStr>,
}

impl ConstrctInfo {
    pub fn with_name(mut self, name: Option<AStr>) -> Self {
        self.name = name;
        self
    }

    pub fn with_help(mut self, help: Option<AStr>) -> Self {
        self.help = help;
        self
    }

    pub fn with_alias(mut self, alias: Option<Vec<AStr>>) -> Self {
        self.alias = alias;
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

    pub fn with_ctor(mut self, ctor: Option<AStr>) -> Self {
        self.ctor = ctor;
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

    fn has_alias(&self) -> bool {
        self.alias.is_some()
    }

    fn has_index(&self) -> bool {
        self.index.is_some()
    }

    fn has_help(&self) -> bool {
        self.help.is_some()
    }

    fn has_ctor(&self) -> bool {
        self.ctor.is_some()
    }

    fn name(&self) -> Option<&AStr> {
        self.name.as_ref()
    }

    fn force(&self) -> Option<bool> {
        self.force
    }

    fn alias(&self) -> Option<&Vec<AStr>> {
        self.alias.as_ref()
    }

    fn index(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn help(&self) -> Option<&AStr> {
        self.help.as_ref()
    }

    fn ctor(&self) -> Option<&AStr> {
        self.ctor.as_ref()
    }

    fn take_name(&mut self) -> Option<AStr> {
        self.name.take()
    }

    fn take_force(&mut self) -> Option<bool> {
        self.force.take()
    }

    fn take_alias(&mut self) -> Option<Vec<AStr>> {
        self.alias.take()
    }

    fn take_index(&mut self) -> Option<Index> {
        self.index.take()
    }

    fn take_help(&mut self) -> Option<AStr> {
        self.help.take()
    }

    fn take_ctor(&mut self) -> Option<AStr> {
        self.ctor.take()
    }
}
