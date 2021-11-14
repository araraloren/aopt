use super::info::FilterInfo;
use super::Set;
use crate::gstr;
use crate::opt::{index::Index, Opt};

#[derive(Debug)]
pub struct Filter<'a> {
    set: &'a dyn Set,

    info: FilterInfo,
}

impl<'a> Filter<'a> {
    pub fn new(set: &'a dyn Set, info: FilterInfo) -> Self {
        Self { set, info }
    }

    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate_style(deactivate_style);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    pub fn set_type_name(&mut self, type_name: &str) -> &mut Self {
        self.info.set_type_name(gstr(type_name));
        self
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.info.set_name(gstr(name));
        self
    }

    pub fn set_prefix(&mut self, prefix: &str) -> &mut Self {
        self.info.set_prefix(gstr(prefix));
        self
    }

    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.info.set_index(index);
        self
    }

    pub fn find(&self) -> Option<&'a Box<dyn Opt>> {
        for opt in self.set.opt_iter() {
            if self.info.match_opt(opt.as_ref()) {
                return Some(opt);
            }
        }
        None
    }

    pub fn find_all(&self) -> Vec<&'a Box<dyn Opt>> {
        let mut ret = vec![];

        for opt in self.set.opt_iter() {
            if self.info.match_opt(opt.as_ref()) {
                ret.push(opt);
            }
        }
        ret
    }
}

#[derive(Debug)]
pub struct FilterMut<'a> {
    set: &'a mut dyn Set,

    info: FilterInfo,
}

impl<'a> FilterMut<'a> {
    pub fn new(set: &'a mut dyn Set, info: FilterInfo) -> Self {
        Self { set, info }
    }

    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate_style(deactivate_style);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    pub fn set_type_name(&mut self, type_name: &str) -> &mut Self {
        self.info.set_type_name(gstr(type_name));
        self
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.info.set_name(gstr(name));
        self
    }

    pub fn set_prefix(&mut self, prefix: &str) -> &mut Self {
        self.info.set_prefix(gstr(prefix));
        self
    }

    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.info.set_index(index);
        self
    }

    pub fn find(&mut self) -> Option<&mut Box<dyn Opt>> {
        for opt in self.set.opt_iter_mut() {
            if self.info.match_opt(opt.as_ref()) {
                return Some(opt);
            }
        }
        None
    }

    pub fn find_all(&mut self) -> Vec<&mut Box<dyn Opt>> {
        let mut ret = vec![];

        for opt in self.set.opt_iter_mut() {
            if self.info.match_opt(opt.as_ref()) {
                ret.push(opt);
            }
        }
        ret
    }
}
