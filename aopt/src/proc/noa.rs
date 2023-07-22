use std::fmt::Debug;
use std::marker::PhantomData;

use crate::args::Args;
use crate::opt::Opt;
use crate::opt::Style;
use crate::proc::Match;
use crate::proc::Process;
use crate::set::Ctor;
use crate::set::Set;
use crate::trace_log;
use crate::ARef;
use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

pub struct NOAMatch<S> {
    name: Option<Str>,

    args: ARef<Args>,

    arg: Option<ARef<RawVal>>,

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
            .finish()
    }
}

impl<S> Default for NOAMatch<S> {
    fn default() -> Self {
        Self {
            name: None,
            args: ARef::new(Args::default()),
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

    pub fn with_args(mut self, args: ARef<Args>) -> Self {
        self.args = args;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_arg(mut self, arg: Option<ARef<RawVal>>) -> Self {
        self.arg = arg;
        self
    }

    pub fn reset_arg(mut self) -> Self {
        self.arg = self.args.get(self.idx()).map(|v| v.clone().into());
        self
    }
}

impl<S> NOAMatch<S> {
    pub fn idx(&self) -> usize {
        self.noa_index
    }

    pub fn total(&self) -> usize {
        self.noa_total
    }

    pub fn name(&self) -> Option<&Str> {
        self.name.as_ref()
    }

    pub fn clone_arg(&self) -> Option<ARef<RawVal>> {
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

    fn status(&self) -> bool {
        self.matched_uid.is_some()
    }

    fn uid(&self) -> Option<Uid> {
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

    fn is_consume(&self) -> bool {
        false
    }

    fn undo(
        &mut self,
        opt: &mut <<Self::Set as Set>::Ctor as Ctor>::Opt,
    ) -> Result<(), Self::Error> {
        opt.set_matched(false);
        self.reset();
        Ok(())
    }

    /// Match the [`Opt`]'s name, prefix and style, index.
    /// If matched, set the matched of [`Opt`] and return true.
    fn process(
        &mut self,
        opt: &mut <<Self::Set as Set>::Ctor as Ctor>::Opt,
    ) -> Result<bool, Self::Error> {
        let mut matched = opt.mat_style(self.style);

        if matched {
            if !opt.ignore_name() {
                matched = matched && opt.mat_name(self.name());
            }
            if !opt.ignore_alias() && opt.alias().is_some() {
                if let Some(name) = &self.name {
                    matched = matched || opt.mat_alias(name);
                }
            }
            if !opt.ignore_index() {
                matched = matched && {
                    if opt.index().is_some() {
                        opt.mat_index(Some((self.noa_index, self.noa_total)))
                    } else {
                        false
                    }
                };
            }
        }
        if matched {
            opt.set_matched(true);
            self.matched_index = Some(self.noa_index);
            self.matched_uid = Some(opt.uid());
        }
        trace_log!(
            "Matching {{{:?}}} with NOA{{{}}}: {:?}",
            self,
            opt.hint(),
            self.matched_uid
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
    fn style(&self) -> Style {
        self.matches.as_ref().map_or(Style::Null, |v| v.style())
    }

    /// Return true if the process successful.
    fn status(&self) -> bool {
        self.matches.as_ref().map_or(false, |v| v.status())
    }

    /// Return true if the process need consume an argument.
    fn is_consume(&self) -> bool {
        self.consume_arg
    }

    fn add_match(&mut self, mat: NOAMatch<S>) -> &mut Self {
        self.matches = Some(mat);
        self
    }

    fn get_match(&self, index: usize) -> Option<&NOAMatch<S>> {
        if index == 0 {
            self.matches.as_ref()
        } else {
            None
        }
    }

    fn get_match_mut(&mut self, index: usize) -> Option<&mut NOAMatch<S>> {
        if index == 0 {
            self.matches.as_mut()
        } else {
            None
        }
    }

    /// Undo the process modification.
    fn undo(&mut self, set: &mut Self::Set) -> Result<(), Self::Error> {
        if let Some(mat) = self.matches.as_mut() {
            if let Some(uid) = mat.uid() {
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
                crate::trace_log!(
                    "Start process NOA{{{}}} eg. {} -{:?}-",
                    opt.uid(),
                    opt.hint(),
                    opt.index()
                );
                if let Some(mat) = self.matches.as_mut() {
                    if !mat.status() && mat.process(opt)? {
                        self.consume_arg = self.consume_arg || mat.is_consume();
                        return Ok(Some(0));
                    }
                }
            }
        }
        Ok(None)
    }
}
