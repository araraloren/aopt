use std::ffi::OsString;
use std::fmt::Debug;
use std::marker::PhantomData;

use super::Match;
use crate::opt::Alias;
use crate::opt::Help;
use crate::opt::Name;
use crate::opt::Opt;
use crate::opt::OptStyle;
use crate::opt::Prefix;
use crate::set::Set;
use crate::Arc;
use crate::Error;
use crate::Str;
use crate::Uid;

pub struct OptMatch<S> {
    prefix: Str,

    name: Str,

    style: OptStyle,

    argument: Option<Arc<OsString>>,

    matched_uid: Option<Uid>,

    disbale: bool,

    consume_arg: bool,

    index: usize,

    total: usize,

    marker: PhantomData<S>,
}

impl<S> Debug for OptMatch<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptMatch")
            .field("prefix", &self.prefix)
            .field("name", &self.name)
            .field("style", &self.style)
            .field("argument", &self.argument)
            .field("matched_uid", &self.matched_uid)
            .field("disbale", &self.disbale)
            .field("consume_arg", &self.consume_arg)
            .field("index", &self.index)
            .field("total", &self.total)
            .finish()
    }
}

impl<S> Default for OptMatch<S> {
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

    pub fn with_pre(mut self, prefix: Str) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_sty(mut self, style: OptStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_arg(mut self, argument: Option<Arc<OsString>>) -> Self {
        self.argument = argument;
        self
    }

    pub fn with_dsb(mut self, disbale: bool) -> Self {
        self.disbale = disbale;
        self
    }

    pub fn with_idx(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn with_len(mut self, total: usize) -> Self {
        self.total = total;
        self
    }

    pub fn with_consume(mut self, consume_arg: bool) -> Self {
        self.consume_arg = consume_arg;
        self
    }

    pub fn name(&self) -> &Str {
        &self.name
    }

    pub fn pre(&self) -> Option<&Str> {
        Some(&self.prefix)
    }

    pub fn dsb(&self) -> bool {
        self.disbale
    }

    pub fn idx(&self) -> usize {
        self.index
    }

    pub fn len(&self) -> usize {
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

    fn is_mat(&self) -> bool {
        self.matched_uid.is_some()
    }

    fn mat_uid(&self) -> Option<Uid> {
        self.matched_uid
    }

    fn set_uid(&mut self, uid: Uid) {
        self.matched_uid = Some(uid);
    }

    fn sty(&self) -> OptStyle {
        self.style
    }

    fn arg(&self) -> Option<&Arc<OsString>> {
        self.argument.as_ref()
    }

    fn consume(&self) -> bool {
        self.consume_arg
    }

    fn undo(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<(), Self::Error> {
        opt.set_setted(false);
        self.reset();
        Ok(())
    }

    /// Match the [`Opt`]'s name, prefix and style.
    /// Then call the [`val`](Opt::val) check the argument.
    /// If matched, set the setted of [`Opt`] and return true.
    fn process(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<bool, Self::Error> {
        let mut matched = opt.mat_sty(self.style);

        if matched {
            matched = matched
                && ((opt.mat_name(self.name()) && opt.mat_pre(self.pre()))
                    || opt.mat_alias(&self.prefix, self.name()));
        }
        if matched {
            if self.consume() && self.argument.is_none() {
                return Err(Error::sp_missing_argument(opt.hint()));
            }
            // set the value of current option
            if opt.check(self.arg().cloned(), self.disbale, (self.index, self.total))? {
                opt.set_setted(true);
                self.matched_uid = Some(opt.uid());
            } else {
                matched = false;
            }
        }
        Ok(matched)
    }
}
