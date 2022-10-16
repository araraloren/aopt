use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Debug;
use std::marker::PhantomData;

use super::Match;
use crate::opt::Index;
use crate::opt::Name;
use crate::opt::Opt;
use crate::opt::OptStyle;
use crate::prelude::Args;
use crate::prelude::Prefix;
use crate::set::Set;
use crate::Arc;
use crate::Error;
use crate::Str;
use crate::Uid;

pub struct NOAMatch<S> {
    args: Arc<Args>,

    style: OptStyle,

    noa_index: usize,

    noa_total: usize,

    matched_uid: Option<Uid>,

    matched_index: Option<usize>,

    marker: PhantomData<S>,
}

impl<S> Debug for NOAMatch<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NOAMatch")
            .field("args", &self.args)
            .field("style", &self.style)
            .field("noa_index", &self.noa_index)
            .field("noa_total", &self.noa_total)
            .field("matched_uid", &self.matched_uid)
            .field("matched_index", &self.matched_index)
            .field("marker", &self.marker)
            .finish()
    }
}

impl<S> Default for NOAMatch<S> {
    fn default() -> Self {
        Self {
            args: Arc::new(Args::default()),
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
    pub fn with_args(mut self, args: Arc<Args>) -> Self {
        self.args = args;
        self
    }

    pub fn with_sty(mut self, style: OptStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_idx(mut self, index: usize) -> Self {
        self.noa_index = index;
        self
    }

    pub fn with_len(mut self, total: usize) -> Self {
        self.noa_total = total;
        self
    }

    pub fn name(&self) -> &OsString {
        // noa index == index + 1
        &self.args[self.noa_index - 1]
    }

    pub fn pre(&self) -> Option<&Str> {
        None
    }

    pub fn dsb(&self) -> bool {
        false
    }

    pub fn idx(&self) -> usize {
        self.noa_index
    }

    pub fn len(&self) -> usize {
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
        None
    }

    fn consume(&self) -> bool {
        false
    }

    fn undo(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<(), Self::Error> {
        opt.set_setted(false);
        self.reset();
        Ok(())
    }

    /// Match the [`Opt`]'s name, prefix and style, index.
    /// Then call the [`val`](Opt::val) check the argument.
    /// If matched, set the setted of [`Opt`] and return true.
    fn process(&mut self, opt: &mut <Self::Set as Set>::Opt) -> Result<bool, Self::Error> {
        let mut matched = opt.mat_sty(self.style);

        if matched {
            // if the name is valid utf8, pass it to match name
            if let Some(utf8) = self.name().to_str() {
                matched = matched && opt.mat_name(&utf8.into());
            }
            // or pass a fake string to it
            else {
                matched = matched && opt.mat_name(&Str::default());
            }
            matched = matched
                && opt.mat_pre(self.pre())
                && opt.mat_idx(Some((self.noa_index as usize, self.noa_total as usize)));
        }
        if matched {
            // set the value of current option
            if opt.check(
                Some(Arc::new(self.name().clone())),
                false,
                (self.noa_index, self.noa_total),
            )? {
                opt.set_setted(true);
                self.matched_index = Some(self.noa_index);
                self.matched_uid = Some(opt.uid());
            } else {
                matched = false;
            }
        }
        Ok(matched)
    }
}
