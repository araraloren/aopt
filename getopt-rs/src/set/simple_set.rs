use std::collections::HashMap;

use super::{Commit, Filter, FilterMut, Uid};
use super::{CreateInfo, Creator, FilterInfo, Set};
use super::{CreatorSet, OptionSet, PrefixSet};
use super::{Index, IndexMut, Iter, IterMut};
use crate::err::{Error, ParserError, Result};
use crate::opt::Opt;

#[derive(Debug, Default)]
pub struct SimpleSet {
    opt: Vec<Box<dyn Opt>>,

    creator: HashMap<String, Box<dyn Creator>>,

    prefix: Vec<String>,
}

impl SimpleSet {
    pub fn new() -> Self {
        let mut ret = Self::default();
        crate::tools::initialize_creator(&mut ret);
        crate::tools::initialize_prefix(&mut ret);
        ret
    }
}

impl Set for SimpleSet {}

impl OptionSet for SimpleSet {
    fn add_opt(&mut self, opt_str: &str) -> Result<Commit> {
        let info = CreateInfo::parse(opt_str, self.get_prefix())?;
        Ok(Commit::new(self, info))
    }

    fn add_opt_ci(&mut self, ci: CreateInfo) -> Result<Uid> {
        let uid = self.opt.len() as Uid;
        let mut ci = ci;

        match self.get_creator(ci.get_type_name()) {
            Some(creator) => {
                ci.set_uid(uid);

                let opt = creator.create_with(ci)?;

                self.opt.push(opt);
                Ok(uid)
            }
            None => {
                Err(ParserError::NotSupportOptionType(format!("{}", ci.get_type_name())).into())
            }
        }
    }

    fn add_opt_raw(&mut self, opt: Box<dyn Opt>) -> Result<Uid> {
        let mut opt = opt;
        let uid = self.opt.len() as Uid;

        opt.set_uid(uid);
        self.opt.push(opt);
        Ok(uid)
    }

    fn get_opt(&self, uid: Uid) -> Option<&Box<dyn Opt>> {
        self.opt.get(uid as usize)
    }

    fn get_opt_mut(&mut self, uid: Uid) -> Option<&mut Box<dyn Opt>> {
        self.opt.get_mut(uid as usize)
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

    fn reset(&mut self) {
        for opt in self.opt.iter_mut() {
            opt.reset_value();
        }
    }
}

impl CreatorSet for SimpleSet {
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
}

impl PrefixSet for SimpleSet {
    fn add_prefix(&mut self, prefix: String) {
        self.prefix.push(prefix);
        self.prefix.sort_by(|a, b| b.len().cmp(&a.len()));
    }

    fn get_prefix(&self) -> &[String] {
        &self.prefix
    }

    fn clr_prefix(&mut self) {
        self.prefix.clear();
    }
}

impl Index<Uid> for SimpleSet {
    type Output = Box<dyn Opt>;

    fn index(&self, index: Uid) -> &Self::Output {
        self.get_opt(index).unwrap()
    }
}

impl IndexMut<Uid> for SimpleSet {
    fn index_mut(&mut self, index: Uid) -> &mut Self::Output {
        self.get_opt_mut(index).unwrap()
    }
}

impl AsRef<[Box<dyn Opt>]> for SimpleSet {
    fn as_ref(&self) -> &[Box<dyn Opt>] {
        self.opt.as_ref()
    }
}

impl AsMut<[Box<dyn Opt>]> for SimpleSet {
    fn as_mut(&mut self) -> &mut [Box<dyn Opt>] {
        self.opt.as_mut()
    }
}
