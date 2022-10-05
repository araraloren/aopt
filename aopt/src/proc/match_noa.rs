use std::marker::PhantomData;

use super::Match;
use crate::opt::Index;
use crate::opt::Name;
use crate::opt::Opt;
use crate::opt::OptStyle;
use crate::prelude::Prefix;
use crate::set::Set;
use crate::Error;
use crate::Str;
use crate::Uid;

#[derive(Debug)]
pub struct NOAMatch<S: Set> {
    name: Str,

    style: OptStyle,

    noa_index: usize,

    noa_total: usize,

    matched_uid: Option<Uid>,

    matched_index: Option<usize>,

    marker: PhantomData<S>,
}

impl<S> Default for NOAMatch<S>
where
    S: Set,
{
    fn default() -> Self {
        Self {
            name: Str::default(),
            style: OptStyle::default(),
            noa_index: 0,
            noa_total: 0,
            matched_uid: None,
            matched_index: None,
            marker: Default::default(),
        }
    }
}

impl<S> NOAMatch<S>
where
    S: Set,
{
    pub fn with_name(mut self, name: Str) -> Self {
        self.name = name;
        self
    }

    pub fn with_style(mut self, style: OptStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.noa_index = index;
        self
    }

    pub fn with_total(mut self, total: usize) -> Self {
        self.noa_total = total;
        self
    }

    pub fn get_name(&self) -> Str {
        self.name.clone()
    }

    pub fn get_prefix(&self) -> Option<Str> {
        None
    }

    pub fn get_argument(&self) -> Option<Str> {
        None
    }

    pub fn get_style(&self) -> OptStyle {
        self.style
    }

    pub fn get_deactivate(&self) -> bool {
        false
    }

    pub fn get_index(&self) -> usize {
        self.noa_index
    }

    pub fn get_total(&self) -> usize {
        self.noa_total
    }
}

impl<S: Set> Match for NOAMatch<S>
where
    S::Opt: Opt,
{
    type Set = S;

    type Error = Error;

    fn reset(&mut self) {
        self.matched_index = None;
        self.matched_uid = None;
    }

    fn is_matched(&self) -> bool {
        self.matched_uid.is_some()
    }

    fn get_matched_uid(&self) -> Option<Uid> {
        self.matched_uid
    }

    fn set_matched_uid(&mut self, uid: Uid) {
        self.matched_uid = Some(uid);
    }

    fn get_style(&self) -> OptStyle {
        self.style
    }

    fn get_argument(&self) -> Option<Str> {
        None
    }

    fn is_consume_argument(&self) -> bool {
        false
    }

    fn undo(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<(), Self::Error> {
        opt.set_setted(false);
        self.reset();
        Ok(())
    }

    /// Match the [`Opt`]'s name, prefix and style, index.
    /// Then call the [`check_value`](Opt::check_value) check the argument.
    /// If matched, set the setted of [`Opt`] and return true.
    fn process(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<bool, Self::Error> {
        let mut matched = opt.match_style(self.style);

        if matched {
            matched = matched
                && (opt.match_name(self.get_name())
                    && opt.match_prefix(self.get_prefix())
                    && opt.match_index(Some((self.noa_index as usize, self.noa_total as usize))));
        }
        if matched {
            // set the value of current option
            if opt.check_value(
                Some(self.get_name()),
                false,
                (self.noa_index, self.noa_total),
            )? {
                opt.set_setted(true);
                self.matched_index = Some(self.noa_index);
                self.matched_uid = Some(opt.get_uid());
            } else {
                matched = false;
            }
        }
        Ok(matched)
    }
}
