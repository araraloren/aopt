mod commit;
mod filter;
mod index;
mod info;
mod simple_set;

use std::fmt::Debug;
use std::slice::{Iter, IterMut};
use ustr::Ustr;

use crate::err::Result;
use crate::opt::Opt;
use crate::opt::OptValue;
use crate::uid::Uid;

pub use self::commit::Commit;
pub use self::filter::Filter;
pub use self::filter::FilterMut;
pub use self::index::SetIndex;
pub use self::info::CreateInfo;
pub use self::info::FilterInfo;
pub use self::info::OptionInfo;
pub use self::simple_set::SimpleSet;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        /// Trait using for create [`Opt`] with given [`CreateInfo`].
        pub trait Creator: Debug + Send + Sync {
            fn get_type_name(&self) -> Ustr;

            fn is_support_deactivate_style(&self) -> bool;

            fn create_with(&self, create_info: CreateInfo) -> Result<Box<dyn Opt>>;
        }

        pub trait Set: Debug + PrefixSet + OptionSet + CreatorSet + Send + Sync {}
    }
    else {
        /// Trait using for create [`Opt`] with given [`CreateInfo`].
        pub trait Creator: Debug {
            fn get_type_name(&self) -> Ustr;

            fn is_support_deactivate_style(&self) -> bool;

            fn create_with(&self, create_info: CreateInfo) -> Result<Box<dyn Opt>>;
        }

        pub trait Set: Debug + PrefixSet + OptionSet + CreatorSet {}
    }
}

pub trait PrefixSet {
    fn add_prefix(&mut self, prefix: Ustr);

    fn get_prefix(&self) -> &[Ustr];

    fn clr_prefix(&mut self);
}

pub trait OptionSet {
    fn add_opt(&mut self, opt_str: &str) -> Result<Commit<'_, Self>>
    where
        Self: Sized;

    fn add_opt_ci(&mut self, ci: CreateInfo) -> Result<Uid>;

    fn add_opt_raw(&mut self, opt: Box<dyn Opt>) -> Result<Uid>;

    fn get_opt(&self, uid: Uid) -> Option<&Box<dyn Opt>>;

    fn get_opt_mut(&mut self, uid: Uid) -> Option<&mut Box<dyn Opt>>;

    fn len(&self) -> usize;

    fn opt_iter(&self) -> Iter<Box<dyn Opt>>;

    fn opt_iter_mut(&mut self) -> IterMut<Box<dyn Opt>>;

    fn find(&self, opt_str: &str) -> Result<Option<&Box<dyn Opt>>>;

    fn find_mut(&mut self, opt_str: &str) -> Result<Option<&mut Box<dyn Opt>>>;

    fn filter(&self, opt_str: &str) -> Result<Filter>;

    fn filter_mut(&mut self, opt_str: &str) -> Result<FilterMut>;

    fn reset(&mut self);

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // some help functions access option data

    fn get_value(&self, opt_str: &str) -> Result<Option<&OptValue>> {
        Ok(self.find(opt_str)?.map(|v| v.get_value()))
    }

    fn get_value_mut(&mut self, opt_str: &str) -> Result<Option<&mut OptValue>> {
        Ok(self.find_mut(opt_str)?.map(|v| v.get_value_mut()))
    }

    fn set_value(&mut self, opt_str: &str, value: OptValue) -> Result<Option<&mut Box<dyn Opt>>> {
        Ok(self.find_mut(opt_str)?.map(|v| {
            v.set_value(value);
            v
        }))
    }
}

pub trait CreatorSet {
    fn has_creator(&self, type_name: Ustr) -> bool;

    fn add_creator(&mut self, creator: Box<dyn Creator>);

    fn app_creator(&mut self, creator: Vec<Box<dyn Creator>>);

    fn rem_creator(&mut self, opt_type: Ustr) -> bool;

    fn get_creator(&self, opt_type: Ustr) -> Option<&dyn Creator>;
}
