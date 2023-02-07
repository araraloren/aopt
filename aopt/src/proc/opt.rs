use std::fmt::Debug;
use std::marker::PhantomData;

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

pub struct OptMatch<S> {
    name: Str,

    style: Style,

    argument: Option<Arc<RawVal>>,

    matched_uid: Option<Uid>,

    consume_arg: bool,

    index: usize,

    total: usize,

    marker: PhantomData<S>,
}

impl<S> Debug for OptMatch<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptMatch")
            .field("name", &self.name)
            .field("style", &self.style)
            .field("argument", &self.argument)
            .field("matched_uid", &self.matched_uid)
            .field("consume_arg", &self.consume_arg)
            .field("total", &self.total)
            .finish()
    }
}

impl<S> Default for OptMatch<S> {
    fn default() -> Self {
        Self {
            name: Str::default(),
            style: Style::default(),
            argument: None,
            matched_uid: None,
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

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_idx(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn with_total(mut self, total: usize) -> Self {
        self.total = total;
        self
    }

    pub fn with_consume(mut self, consume_arg: bool) -> Self {
        self.consume_arg = consume_arg;
        self
    }

    pub fn with_arg(mut self, argument: Option<Arc<RawVal>>) -> Self {
        self.argument = argument;
        self
    }
}

impl<S> OptMatch<S> {
    pub fn name(&self) -> Option<&Str> {
        Some(&self.name)
    }

    pub fn idx(&self) -> usize {
        self.index
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn clone_arg(&self) -> Option<Arc<RawVal>> {
        self.argument.clone()
    }
}

impl<S: Set> Match for OptMatch<S>
where
    <S::Ctor as Ctor>::Opt: Opt,
{
    type Set = S;

    type Error = Error;

    fn reset(&mut self) {
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
        self.argument.as_ref().map(|v| v.as_ref())
    }

    fn is_consume(&self) -> bool {
        self.consume_arg
    }

    fn undo(&mut self, opt: &mut <<S as Set>::Ctor as Ctor>::Opt) -> Result<(), Self::Error> {
        opt.set_matched(false);
        self.reset();
        Ok(())
    }

    /// Match the [`Opt`]'s name, prefix and style.
    /// Then call the [`check_val`](Opt::check_val) check the argument.
    /// If matched, set the matched of [`Opt`] and return true.
    fn process(
        &mut self,
        opt: &mut <<Self::Set as Set>::Ctor as Ctor>::Opt,
    ) -> Result<bool, Self::Error> {
        let mut matched = opt.mat_style(self.style);

        if matched {
            if !opt.ignore_name() {
                matched = opt.mat_name(self.name());
            }
            if !opt.ignore_alias() {
                if opt.alias().is_some() {
                    matched = matched || opt.mat_alias(&self.name)
                }
            }
            if !opt.ignore_index() {
                matched = matched && {
                    if opt.index().is_some() {
                        opt.mat_index(Some((self.index, self.total)))
                    } else {
                        false
                    }
                };
            }
        }
        if matched {
            if self.is_consume() && self.argument.is_none() {
                return Err(Error::sp_missing_argument(opt.hint()));
            }
            opt.set_matched(true);
            self.matched_uid = Some(opt.uid());
        }
        crate::trace_log!(
            "Matching {{{:?}}} with Opt{{{}}}: {}",
            self,
            opt.hint(),
            matched
        );
        Ok(matched)
    }
}

/// OptProcess matching the [`Opt`] against [`OptMatch`].
pub struct OptProcess<S> {
    matches: Vec<OptMatch<S>>,

    consume_arg: bool,

    any_match: bool,
}

impl<S> Debug for OptProcess<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptProcess")
            .field("matches", &self.matches)
            .field("consume_arg", &self.consume_arg)
            .field("any_match", &self.any_match)
            .finish()
    }
}

impl<S> OptProcess<S> {
    pub fn new(matches: Vec<OptMatch<S>>) -> Self {
        Self {
            matches,
            consume_arg: false,
            any_match: false,
        }
    }

    pub fn set_any_match(&mut self, any_match: bool) -> &mut Self {
        self.any_match = any_match;
        self
    }
}

impl<S: Set> Process<OptMatch<S>> for OptProcess<S>
where
    <S::Ctor as Ctor>::Opt: Opt,
{
    type Set = S;

    type Error = Error;

    fn reset(&mut self) {
        self.matches.iter_mut().for_each(|v| v.reset());
    }

    /// Return true if the process successful.
    fn quit(&self) -> bool {
        self.status()
    }

    /// Return the count of [`OptMatch`].
    fn count(&self) -> usize {
        self.matches.len()
    }

    /// Return the [`Style`] of OptProcess.
    fn style(&self) -> Style {
        self.matches.last().map_or(Style::Null, |v| v.style())
    }

    /// Return true if the process successful.
    fn status(&self) -> bool {
        if self.any_match {
            self.matches.iter().any(|v| v.status())
        } else {
            self.matches.iter().all(|v| v.status())
        }
    }

    /// Return true if the process need consume an argument.
    fn is_consume(&self) -> bool {
        self.consume_arg
    }

    fn add_match(&mut self, mat: OptMatch<S>) -> &mut Self {
        self.matches.push(mat);
        self
    }

    fn get_match(&self, index: usize) -> Option<&OptMatch<S>> {
        self.matches.get(index)
    }

    fn get_match_mut(&mut self, index: usize) -> Option<&mut OptMatch<S>> {
        self.matches.get_mut(index)
    }

    /// Undo the process modification.
    fn undo(&mut self, set: &mut Self::Set) -> Result<(), Self::Error> {
        for mat in self.matches.iter_mut() {
            if let Some(uid) = mat.uid() {
                if let Some(opt) = set.get_mut(uid) {
                    mat.undo(opt)?;
                }
            }
        }
        Ok(())
    }

    /// Match the given [`Opt`] against inner [`OptMatch`], return the index if successful.
    fn process(&mut self, uid: Uid, set: &mut Self::Set) -> Result<Option<usize>, Self::Error> {
        if let Some(opt) = set.get_mut(uid) {
            let style_check = opt.mat_style(Style::Argument)
                || opt.mat_style(Style::Boolean)
                || opt.mat_style(Style::Combined);

            if style_check {
                crate::trace_log!(
                    "Start process OPT{{{}}} eg. {} action: {}",
                    opt.uid(),
                    opt.hint(),
                    opt.action()
                );
                for (index, mat) in self.matches.iter_mut().enumerate() {
                    if !mat.status() && mat.process(opt)? {
                        self.consume_arg = self.consume_arg || mat.is_consume();
                        return Ok(Some(index));
                    }
                }
            }
        }
        Ok(None)
    }
}
