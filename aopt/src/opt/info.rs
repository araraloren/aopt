use super::Index;
use crate::Str;

pub trait Information {
    fn has_name(&self) -> bool;

    fn has_force(&self) -> bool;

    fn has_alias(&self) -> bool;

    fn has_idx(&self) -> bool;

    fn has_help(&self) -> bool;

    fn has_ctor(&self) -> bool;

    fn name(&self) -> Option<&Str>;

    fn force(&self) -> Option<bool>;

    fn alias(&self) -> Option<&Vec<Str>>;

    fn idx(&self) -> Option<&Index>;

    fn help(&self) -> Option<&Str>;

    fn ctor(&self) -> Option<&Str>;

    fn take_name(&mut self) -> Option<Str>;

    fn take_force(&mut self) -> Option<bool>;

    fn take_alias(&mut self) -> Option<Vec<Str>>;

    fn take_idx(&mut self) -> Option<Index>;

    fn take_help(&mut self) -> Option<Str>;

    fn take_ctor(&mut self) -> Option<Str>;
}

/// Parsing result of option constructor string.
#[derive(Debug, Default)]
pub struct ConstrctInfo {
    pub(crate) pattern: Str,

    pub(crate) name: Option<Str>,

    pub(crate) alias: Option<Vec<Str>>,

    pub(crate) force: Option<bool>,

    pub(crate) index: Option<Index>,

    pub(crate) help: Option<Str>,

    pub(crate) ctor: Option<Str>,
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

    pub fn with_help(mut self, help: Option<Str>) -> Self {
        self.help = help;
        self
    }

    pub fn with_alias(mut self, alias: Option<Vec<Str>>) -> Self {
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

    pub fn with_ctor(mut self, ctor: Option<Str>) -> Self {
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

    fn has_idx(&self) -> bool {
        self.index.is_some()
    }

    fn has_help(&self) -> bool {
        self.help.is_some()
    }

    fn has_ctor(&self) -> bool {
        self.ctor.is_some()
    }

    fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    fn force(&self) -> Option<bool> {
        self.force
    }

    fn alias(&self) -> Option<&Vec<Str>> {
        self.alias.as_ref()
    }

    fn idx(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn help(&self) -> Option<&Str> {
        self.help.as_ref()
    }

    fn ctor(&self) -> Option<&Str> {
        self.ctor.as_ref()
    }

    fn take_name(&mut self) -> Option<Str> {
        self.name.take()
    }

    fn take_force(&mut self) -> Option<bool> {
        self.force.take()
    }

    fn take_alias(&mut self) -> Option<Vec<Str>> {
        self.alias.take()
    }

    fn take_idx(&mut self) -> Option<Index> {
        self.index.take()
    }

    fn take_help(&mut self) -> Option<Str> {
        self.help.take()
    }

    fn take_ctor(&mut self) -> Option<Str> {
        self.ctor.take()
    }
}
