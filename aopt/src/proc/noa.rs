use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::trace;

use crate::args::Args;
use crate::opt::Opt;
use crate::opt::Style;
use crate::proc::Match;
use crate::proc::Process;
use crate::set::Ctor;
use crate::set::Set;
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

pub struct NOAMatch<S> {
    name: Option<Str>,

    args: Arc<Args>,

    arg: Option<Arc<RawVal>>,

    style: Style,

    noa_index: usize,

    noa_total: usize,

    matched_uid: Option<Uid>,

    matched_index: Option<usize>,

    marker: PhantomData<S>,
}

impl<S> Debug for NOAMatch<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NOAMatch")
            .field("name", &self.name)
            .field("args", &self.args)
            .field("arg", &self.arg)
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
            name: None,
            args: Arc::new(Args::default()),
            arg: None,
            style: Style::default(),
            noa_index: 0,
            noa_total: 0,
            matched_uid: None,
            matched_index: None,
            marker: Default::default(),
        }
        .reset_arg()
    }
}

impl<S> NOAMatch<S> {
    pub fn with_idx(mut self, index: usize) -> Self {
        self.noa_index = index;
        self
    }

    pub fn with_total(mut self, total: usize) -> Self {
        self.noa_total = total;
        self
    }

    pub fn with_name(mut self, name: Option<Str>) -> Self {
        self.name = name;
        self
    }

    pub fn with_args(mut self, args: Arc<Args>) -> Self {
        self.args = args;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn reset_arg(mut self) -> Self {
        self.arg = self
            .args
            .get(self.idx().saturating_sub(1))
            .map(|v| v.clone().into());
        self
    }
}

impl<S> NOAMatch<S> {
    pub fn disable(&self) -> bool {
        false
    }

    pub fn idx(&self) -> usize {
        self.noa_index
    }

    pub fn total(&self) -> usize {
        self.noa_total
    }

    pub fn prefix(&self) -> Option<&Str> {
        None
    }

    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    pub fn clone_arg(&self) -> Option<Arc<RawVal>> {
        self.arg.clone()
    }
}

impl<S: Set> Match for NOAMatch<S>
where
    <S::Ctor as Ctor>::Opt: Opt,
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

    fn style(&self) -> Style {
        self.style
    }

    fn arg(&self) -> Option<&RawVal> {
        self.arg.as_ref().map(|v| v.as_ref())
    }

    fn consume(&self) -> bool {
        false
    }

    fn undo(
        &mut self,
        opt: &mut <<Self::Set as Set>::Ctor as Ctor>::Opt,
    ) -> Result<(), Self::Error> {
        opt.set_setted(false);
        self.reset();
        Ok(())
    }

    /// Match the [`Opt`]'s name, prefix and style, index.
    /// Then call the [`check_val`](Opt::check_val) check the argument.
    /// If matched, set the setted of [`Opt`] and return true.
    fn process(
        &mut self,
        opt: &mut <<Self::Set as Set>::Ctor as Ctor>::Opt,
    ) -> Result<bool, Self::Error> {
        let mut matched = opt.mat_style(self.style);

        if matched {
            matched = matched && opt.mat_name(self.name());
            matched = matched
                && opt.mat_prefix(self.prefix())
                && opt.mat_idx(Some((self.noa_index, self.noa_total)));
            // NOA not support alias, skip alias matching
        }
        if matched {
            // set the value of current option
            if opt.check_val(self.arg(), false, (self.noa_index, self.noa_total))? {
                opt.set_setted(true);
                self.matched_index = Some(self.noa_index);
                self.matched_uid = Some(opt.uid());
            } else {
                matched = false;
            }
        }
        trace!(
            "Matching {{name: {:?}, index: {:?} style: {}, arg: {:?}}} with NOA{{{}}}: {:?}",
            self.name(),
            self.idx(),
            self.style(),
            self.arg(),
            opt.hint(),
            self.matched_uid,
        );
        Ok(matched)
    }
}

/// OptProcess matching the [`Opt`] against [`NOAMatch`].
pub struct NOAProcess<S> {
    matches: Option<NOAMatch<S>>,

    consume_arg: bool,
}

impl<S> Debug for NOAProcess<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NOAProcess")
            .field("matches", &self.matches)
            .field("consume_arg", &self.consume_arg)
            .finish()
    }
}

impl<S> NOAProcess<S> {
    pub fn new(matches: Option<NOAMatch<S>>) -> Self {
        Self {
            matches,
            consume_arg: false,
        }
    }
}

impl<S: Set> Process<NOAMatch<S>> for NOAProcess<S>
where
    <S::Ctor as Ctor>::Opt: Opt,
{
    type Set = S;

    type Error = Error;

    fn reset(&mut self) {
        self.matches.iter_mut().for_each(|v| v.reset())
    }

    /// NOA matching will process all the [`Opt`].
    fn quit(&self) -> bool {
        false
    }

    /// Always return 1.
    fn count(&self) -> usize {
        1
    }

    /// Return the style of inner [`NOAMatch`].
    fn sty(&self) -> Style {
        self.matches.as_ref().map_or(Style::Null, |v| v.style())
    }

    /// Return true if the process successful.
    fn is_mat(&self) -> bool {
        self.matches.as_ref().map_or(false, |v| v.is_mat())
    }

    /// Return true if the process need consume an argument.
    fn consume(&self) -> bool {
        self.consume_arg
    }

    fn add_mat(&mut self, mat: NOAMatch<S>) -> &mut Self {
        self.matches = Some(mat);
        self
    }

    fn mat(&self, index: usize) -> Option<&NOAMatch<S>> {
        if index == 0 {
            self.matches.as_ref()
        } else {
            None
        }
    }

    fn mat_mut(&mut self, index: usize) -> Option<&mut NOAMatch<S>> {
        if index == 0 {
            self.matches.as_mut()
        } else {
            None
        }
    }

    /// Undo the process modification.
    fn undo(&mut self, set: &mut Self::Set) -> Result<(), Self::Error> {
        if let Some(mat) = self.matches.as_mut() {
            if let Some(uid) = mat.mat_uid() {
                if let Some(opt) = set.get_mut(uid) {
                    mat.undo(opt)?;
                }
            }
        }
        Ok(())
    }

    /// Match the given [`Opt`] against inner [`NOAMatch`], return the index (always 0) if successful.
    fn process(&mut self, uid: Uid, set: &mut Self::Set) -> Result<Option<usize>, Self::Error> {
        if let Some(opt) = set.get_mut(uid) {
            let style_check = opt.mat_style(Style::Cmd)
                || opt.mat_style(Style::Main)
                || opt.mat_style(Style::Pos);

            if style_check {
                trace!(
                    "Start process NOA{{{}}} eg. {}@{:?}",
                    opt.uid(),
                    opt.hint(),
                    opt.idx()
                );
                if let Some(mat) = self.matches.as_mut() {
                    if !mat.is_mat() && mat.process(opt)? {
                        self.consume_arg = self.consume_arg || mat.consume();
                        return Ok(Some(0));
                    }
                }
            }
        }
        Ok(None)
    }
}
