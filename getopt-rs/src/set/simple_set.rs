use std::collections::HashMap;

use super::{Commit, Filter, FilterMut, Uid};
use super::{CreateInfo, Creator, FilterInfo, Set};
use super::{Index, IndexMut, Iter, IterMut};
use crate::err::{Error, Result};
use crate::opt::Opt;

#[derive(Debug, Default)]
pub struct SimpleSet {
    opt: Vec<Box<dyn Opt>>,

    creator: HashMap<String, Box<dyn Creator>>,

    prefix: Vec<String>,
}

impl Set for SimpleSet {
    fn has_creator(&self, opt_type: &str) -> bool {
        self.creator.contains_key(opt_type)
    }

    fn add_creator(&mut self, creator: Box<dyn Creator>) {
        let opt_type = creator.get_type_name();
        self.creator.insert(String::from(opt_type), creator);
    }

    fn app_creator(&mut self, creator: Vec<Box<dyn Creator>>) {
        for creator in creator {
            self.add_creator(creator);
        }
    }

    fn rem_creator(&mut self, opt_type: &str) -> bool {
        self.creator.remove(opt_type).is_some()
    }

    fn get_creator(&self, opt_type: &str) -> Option<&Box<dyn Creator>> {
        self.creator.get(opt_type)
    }

    fn add_opt(&mut self, opt_str: &str) -> Result<Commit> {
        let info = CreateInfo::parse(opt_str, self.get_prefix())?;
        Ok(Commit::new(self, info))
    }

    fn add_opt_ci(&mut self, ci: CreateInfo) -> Result<Uid> {
        let uid = self.opt.len() as Uid;

        match self.get_creator(ci.get_type_name()) {
            Some(creator) => {
                let opt = creator.create_with(uid, ci)?;

                self.opt.push(opt);
                Ok(uid)
            }
            None => Err(Error::InvalidOptionTypeName(format!(
                "{}",
                ci.get_type_name()
            ))),
        }
    }

    fn add_opt_raw(&mut self, opt: Box<dyn Opt>) -> Result<Uid> {
        let mut opt = opt;
        let uid = self.opt.len() as Uid;

        opt.set_uid(uid);
        self.opt.push(opt);
        Ok(uid)
    }

    fn get_opt(&self, id: Uid) -> Option<&Box<dyn Opt>> {
        self.opt.get(id as usize)
    }

    fn get_opt_mut(&mut self, id: Uid) -> Option<&mut Box<dyn Opt>> {
        self.opt.get_mut(id as usize)
    }

    fn len(&self) -> usize {
        self.opt.len()
    }

    fn iter(&self) -> Iter<Box<dyn Opt>> {
        self.opt.iter()
    }

    fn iter_mut(&mut self) -> IterMut<Box<dyn Opt>> {
        self.opt.iter_mut()
    }

    fn filter(&self, opt_str: &str) -> Result<Filter> {
        Ok(Filter::new(
            self,
            FilterInfo::parse(opt_str, self.get_prefix())?,
        ))
    }

    fn filter_mut(&mut self, opt_str: &str) -> Result<FilterMut> {
        let info = FilterInfo::parse(opt_str, self.get_prefix())?;
        Ok(FilterMut::new(self, info))
    }

    fn set_prefix(&mut self, prefix: Vec<String>) {
        self.prefix = prefix;
    }

    fn app_prefix(&mut self, prefix: String) {
        self.prefix.push(prefix);
    }

    fn get_prefix(&self) -> &Vec<String> {
        &self.prefix
    }

    fn get_opt_by_index(&self, index: usize) -> Option<&Box<dyn Opt>> {
        self.opt.get(index)
    }

    fn get_opt_mut_by_index(&mut self, index: usize) -> Option<&mut Box<dyn Opt>> {
        self.opt.get_mut(index)
    }

    fn find_by_filter(&self, info: &FilterInfo) -> Option<&Box<dyn Opt>> {
        for opt in self.opt.iter() {
            if info.match_opt(opt.as_ref()) {
                return Some(opt);
            }
        }
        None
    }

    fn find_mut_by_filter(&mut self, info: &FilterInfo) -> Option<&mut Box<dyn Opt>> {
        for opt in self.opt.iter_mut() {
            if info.match_opt(opt.as_ref()) {
                return Some(opt);
            }
        }
        None
    }

    fn find_all_by_filter(&self, info: &FilterInfo) -> Vec<&Box<dyn Opt>> {
        let mut ret = vec![];

        for opt in self.opt.iter() {
            if info.match_opt(opt.as_ref()) {
                ret.push(opt);
            }
        }
        ret
    }

    fn find_all_mut_by_filter(&mut self, info: &FilterInfo) -> Vec<&mut Box<dyn Opt>> {
        let mut ret = vec![];

        for opt in self.opt.iter_mut() {
            if info.match_opt(opt.as_ref()) {
                ret.push(opt);
            }
        }
        ret
    }
}

impl Index<Uid> for SimpleSet {
    type Output = Box<dyn Opt>;

    fn index(&self, index: Uid) -> &Self::Output {
        self.get_opt_by_index(index as usize).unwrap()
    }
}

impl IndexMut<Uid> for SimpleSet {
    fn index_mut(&mut self, index: Uid) -> &mut Self::Output {
        self.get_opt_mut_by_index(index as usize).unwrap()
    }
}
