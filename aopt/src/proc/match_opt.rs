use std::marker::PhantomData;

use super::Match;
use crate::opt::Alias;
use crate::opt::Help;
use crate::opt::Name;
use crate::opt::Opt;
use crate::opt::OptStyle;
use crate::opt::Prefix;
use crate::set::Set;
use crate::Error;
use crate::Str;
use crate::Uid;

#[derive(Debug)]
pub struct OptMatch<S: Set> {
    prefix: Str,

    name: Str,

    style: OptStyle,

    argument: Option<Str>,

    matched_uid: Option<Uid>,

    disbale: bool,

    consume_arg: bool,

    index: usize,

    total: usize,

    marker: PhantomData<S>,
}

impl<S> Default for OptMatch<S>
where
    S: Set,
{
    fn default() -> Self {
        Self {
            prefix: Str::default(),
            name: Str::default(),
            style: OptStyle::default(),
            argument: None,
            matched_uid: None,
            disbale: false,
            consume_arg: false,
            index: 0,
            total: 0,
            marker: PhantomData::default(),
        }
    }
}

impl<S> OptMatch<S>
where
    S: Set,
{
    pub fn with_name(mut self, name: Str) -> Self {
        self.name = name;
        self
    }

    pub fn with_prefix(mut self, prefix: Str) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_style(mut self, style: OptStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_argument(mut self, argument: Option<Str>) -> Self {
        self.argument = argument;
        self
    }

    pub fn with_disable(mut self, disbale: bool) -> Self {
        self.disbale = disbale;
        self
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn with_total(mut self, total: usize) -> Self {
        self.total = total;
        self
    }

    pub fn with_consume_arg(mut self, consume_arg: bool) -> Self {
        self.consume_arg = consume_arg;
        self
    }

    pub fn get_name(&self) -> Str {
        self.name.clone()
    }

    pub fn get_prefix(&self) -> Option<Str> {
        Some(self.prefix.clone())
    }

    pub fn get_argument(&self) -> Option<Str> {
        self.argument.clone()
    }

    pub fn get_style(&self) -> OptStyle {
        self.style
    }

    pub fn get_deactivate(&self) -> bool {
        self.disbale
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_total(&self) -> usize {
        self.total
    }
}

impl<S: Set> Match for OptMatch<S>
where
    S::Opt: Opt,
{
    type Set = S;

    type Error = Error;

    fn reset(&mut self) {
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
        self.argument.clone()
    }

    fn is_consume_argument(&self) -> bool {
        self.consume_arg
    }

    fn undo(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<(), Self::Error> {
        opt.set_setted(false);
        self.reset();
        Ok(())
    }

    /// Match the [`Opt`]'s name, prefix and style.
    /// Then call the [`check_value`](Opt::check_value) check the argument.
    /// If matched, set the setted of [`Opt`] and return true.
    fn process(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<bool, Self::Error> {
        let mut matched = opt.match_style(self.style);

        if matched {
            matched = matched
                && ((opt.match_name(self.get_name()) && opt.match_prefix(self.get_prefix()))
                    || opt.match_alias(self.prefix.clone(), self.get_name()));
        }
        if matched {
            if self.is_consume_argument() && self.argument.is_none() {
                return Err(Error::sp_missing_argument(opt.get_hint()));
            }
            // set the value of current option
            if opt.check_value(self.get_argument(), self.disbale, (self.index, self.total))? {
                opt.set_setted(true);
                self.matched_uid = Some(opt.get_uid());
            } else {
                matched = false;
            }
        }
        Ok(matched)
    }
}
