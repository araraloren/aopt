
use super::info::FilterInfo;
use super::Set;
use crate::opt::index::Index;
use crate::opt::Opt;

#[derive(Debug)]
pub struct Filter<'a> {
    set: &'a dyn Set,

    info: FilterInfo,
}

impl<'a> Filter<'a> {
    pub fn new(set: &'a dyn Set, info: FilterInfo) -> Self {
        Self {
            set, info,
        }
    }

    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate_style(deactivate_style);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    pub fn set_type_name(&mut self, type_name: String) -> &mut Self {
        self.info.set_type_name(type_name);
        self
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.info.set_name(name);
        self
    }

    pub fn set_prefix(&mut self, prefix: String) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.info.set_index(index);
        self
    }

    pub fn find(&self) -> Option<&'a Box<dyn Opt>> {
        self.set.find_by_filter(&self.info)
    }

    pub fn find_all(&self) -> Vec<&'a Box<dyn Opt>> {
        self.set.find_all_by_filter(&self.info)
    }
}

#[derive(Debug)]
pub struct FilterMut<'a> {
    set: &'a mut dyn Set,

    info: FilterInfo,
}

impl<'a> FilterMut<'a> {
    pub fn new(set: &'a mut dyn Set, info: FilterInfo) -> Self {
        Self {
            set, info,
        }
    }

    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate_style(deactivate_style);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    pub fn set_type_name(&mut self, type_name: String) -> &mut Self {
        self.info.set_type_name(type_name);
        self
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.info.set_name(name);
        self
    }

    pub fn set_prefix(&mut self, prefix: String) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.info.set_index(index);
        self
    }

    pub fn find(&mut self) -> Option<&mut Box<dyn Opt>> {
        self.set.find_mut_by_filter(&self.info)
    }

    pub fn find_all(&mut self) -> Vec<&mut Box<dyn Opt>> {
        self.set.find_all_mut_by_filter(&self.info)
    }
}
