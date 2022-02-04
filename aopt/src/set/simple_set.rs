use super::Commit;
use super::CreateInfo;
use super::Creator;
use super::CreatorSet;
use super::Filter;
use super::FilterInfo;
use super::FilterMut;
use super::Index;
use super::IndexMut;
use super::Iter;
use super::IterMut;
use super::OptionSet;
use super::PrefixSet;
use super::Set;
use super::Uid;
use crate::err::Error;
use crate::err::Result;
use crate::gstr;
use crate::opt::ArrayCreator;
use crate::opt::BoolCreator;
use crate::opt::CmdCreator;
use crate::opt::FltCreator;
use crate::opt::IntCreator;
use crate::opt::MainCreator;
use crate::opt::Opt;
use crate::opt::PosCreator;
use crate::opt::StrCreator;
use crate::opt::UintCreator;
use ustr::Ustr;
use ustr::UstrMap;

#[derive(Debug)]
pub struct SimpleSet {
    opt: Vec<Box<dyn Opt>>,

    creator: UstrMap<Box<dyn Creator>>,

    prefix: Vec<Ustr>,
}

impl Default for SimpleSet {
    fn default() -> Self {
        Self::new().with_default_creator().with_default_prefix()
    }
}

impl SimpleSet {
    pub fn new() -> Self {
        Self {
            opt: vec![],
            creator: UstrMap::default(),
            prefix: vec![],
        }
    }

    pub fn with_prefix(mut self, prefix: Ustr) -> Self {
        self.add_prefix(prefix);
        self
    }

    pub fn with_creator(mut self, creator: Box<dyn Creator>) -> Self {
        self.add_creator(creator);
        self
    }

    pub fn with_default_prefix(self) -> Self {
        self.with_prefix("--".into()).with_prefix("-".into())
    }

    pub fn with_default_creator(self) -> Self {
        self.with_creator(Box::new(ArrayCreator::default()))
            .with_creator(Box::new(BoolCreator::default()))
            .with_creator(Box::new(FltCreator::default()))
            .with_creator(Box::new(IntCreator::default()))
            .with_creator(Box::new(StrCreator::default()))
            .with_creator(Box::new(UintCreator::default()))
            .with_creator(Box::new(CmdCreator::default()))
            .with_creator(Box::new(MainCreator::default()))
            .with_creator(Box::new(PosCreator::default()))
    }
}

impl Set for SimpleSet {}

impl OptionSet for SimpleSet {
    fn add_opt(&mut self, opt_str: &str) -> Result<Commit<Self>> {
        let info = CreateInfo::parse(gstr(opt_str), self.get_prefix())?;

        debug!(%opt_str, "create option");
        Ok(Commit::new(self, info))
    }

    fn add_opt_ci(&mut self, ci: CreateInfo) -> Result<Uid> {
        let uid = self.len() as Uid;
        let mut ci = ci;

        trace!(?ci, "create option with ci");
        match self.get_creator(ci.get_type_name()) {
            Some(creator) => {
                ci.set_uid(uid);

                let opt = creator.create_with(ci)?;

                self.opt.push(opt);
                Ok(uid)
            }
            None => Err(Error::opt_unsupport_option_type(ci.get_type_name())),
        }
    }

    fn add_opt_raw(&mut self, mut opt: Box<dyn Opt>) -> Result<Uid> {
        let uid = self.len() as Uid;

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

    fn opt_iter(&self) -> Iter<Box<dyn Opt>> {
        self.opt.iter()
    }

    fn opt_iter_mut(&mut self) -> IterMut<Box<dyn Opt>> {
        self.opt.iter_mut()
    }

    fn find(&self, opt_str: &str) -> Result<Option<&Box<dyn Opt>>> {
        let fi = FilterInfo::parse(gstr(opt_str), self.get_prefix())?;
        for opt in self.opt_iter() {
            if fi.match_opt(opt.as_ref()) {
                return Ok(Some(opt));
            }
        }
        Ok(None)
    }

    fn find_mut(&mut self, opt_str: &str) -> Result<Option<&mut Box<dyn Opt>>> {
        let fi = FilterInfo::parse(gstr(opt_str), self.get_prefix())?;
        for opt in self.opt_iter_mut() {
            if fi.match_opt(opt.as_ref()) {
                return Ok(Some(opt));
            }
        }
        Ok(None)
    }

    fn filter(&self, opt_str: &str) -> Result<Filter> {
        Ok(Filter::new(
            self,
            FilterInfo::parse(gstr(opt_str), self.get_prefix())?,
        ))
    }

    fn filter_mut(&mut self, opt_str: &str) -> Result<FilterMut> {
        let info = FilterInfo::parse(gstr(opt_str), self.get_prefix())?;
        Ok(FilterMut::new(self, info))
    }

    fn reset(&mut self) {
        for opt in self.opt.iter_mut() {
            opt.reset_value();
        }
    }
}

impl CreatorSet for SimpleSet {
    fn has_creator(&self, opt_type: Ustr) -> bool {
        self.creator.contains_key(&opt_type)
    }

    fn add_creator(&mut self, creator: Box<dyn Creator>) {
        let opt_type = creator.get_type_name();
        self.creator.insert(opt_type.into(), creator);
    }

    fn app_creator(&mut self, creator: Vec<Box<dyn Creator>>) {
        for creator in creator {
            self.add_creator(creator);
        }
    }

    fn rem_creator(&mut self, opt_type: Ustr) -> bool {
        self.creator.remove(&opt_type.into()).is_some()
    }

    fn get_creator(&self, opt_type: Ustr) -> Option<&Box<dyn Creator>> {
        self.creator.get(&opt_type.into())
    }
}

impl PrefixSet for SimpleSet {
    fn add_prefix(&mut self, prefix: Ustr) {
        self.prefix.push(prefix);
        self.prefix.sort_by(|a, b| b.len().cmp(&a.len()));
    }

    fn get_prefix(&self) -> &[Ustr] {
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
