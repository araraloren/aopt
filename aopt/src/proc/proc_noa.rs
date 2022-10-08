use std::fmt::Debug;

use super::Match;
use super::NOAMatch;
use super::Process;
use crate::opt::Opt;
use crate::opt::OptStyle;
use crate::set::Set;
use crate::Error;
use crate::Uid;

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
    S::Opt: Opt,
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
    fn sty(&self) -> OptStyle {
        self.matches.as_ref().map_or(OptStyle::Null, |v| v.sty())
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
            let style_check = opt.mat_sty(OptStyle::Cmd)
                || opt.mat_sty(OptStyle::Main)
                || opt.mat_sty(OptStyle::Pos);

            if style_check {
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
