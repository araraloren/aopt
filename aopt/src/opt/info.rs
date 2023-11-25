use super::Index;

pub trait Information {
    fn has_name(&self) -> bool;

    fn has_force(&self) -> bool;

    fn has_alias(&self) -> bool;

    fn has_index(&self) -> bool;

    fn has_help(&self) -> bool;

    fn has_ctor(&self) -> bool;

    fn name(&self) -> Option<&str>;

    fn force(&self) -> Option<bool>;

    fn alias(&self) -> Option<&Vec<String>>;

    fn index(&self) -> Option<&Index>;

    fn help(&self) -> Option<&str>;

    fn ctor(&self) -> Option<&str>;

    fn take_name(&mut self) -> Option<String>;

    fn take_force(&mut self) -> Option<bool>;

    fn take_alias(&mut self) -> Option<Vec<String>>;

    fn take_index(&mut self) -> Option<Index>;

    fn take_help(&mut self) -> Option<String>;

    fn take_ctor(&mut self) -> Option<String>;
}

/// Parsing result of option constructor string.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstrctInfo<'a> {
    pub(crate) pattern: &'a str,

    pub(crate) name: Option<String>,

    pub(crate) alias: Option<Vec<String>>,

    pub(crate) force: Option<bool>,

    pub(crate) index: Option<Index>,

    pub(crate) help: Option<String>,

    pub(crate) ctor: Option<String>,
}

impl<'a> ConstrctInfo<'a> {
    pub fn with_pat(mut self, pattern: &'a str) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_help(mut self, help: Option<String>) -> Self {
        self.help = help;
        self
    }

    pub fn with_alias(mut self, alias: Option<Vec<String>>) -> Self {
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

    pub fn with_ctor(mut self, ctor: Option<String>) -> Self {
        self.ctor = ctor;
        self
    }
}

impl<'a> Information for ConstrctInfo<'a> {
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

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn force(&self) -> Option<bool> {
        self.force
    }

    fn alias(&self) -> Option<&Vec<String>> {
        self.alias.as_ref()
    }

    fn index(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    fn ctor(&self) -> Option<&str> {
        self.ctor.as_deref()
    }

    fn take_name(&mut self) -> Option<String> {
        self.name.take()
    }

    fn take_force(&mut self) -> Option<bool> {
        self.force.take()
    }

    fn take_alias(&mut self) -> Option<Vec<String>> {
        self.alias.take()
    }

    fn take_index(&mut self) -> Option<Index> {
        self.index.take()
    }

    fn take_help(&mut self) -> Option<String> {
        self.help.take()
    }

    fn take_ctor(&mut self) -> Option<String> {
        self.ctor.take()
    }
}
