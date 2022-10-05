use super::Match;
use super::NOAMatch;
use super::Process;
use crate::opt::Opt;
use crate::opt::OptStyle;
use crate::set::Set;
use crate::Error;
use crate::Uid;

/// OptProcess matching the [`Opt`] against [`NOAMatch`].
#[derive(Debug)]
pub struct NOAProcess<S: Set> {
    matches: Option<NOAMatch<S>>,

    consume_arg: bool,
}

impl<S> NOAProcess<S>
where
    S: Set,
{
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
    fn get_style(&self) -> OptStyle {
        self.matches
            .as_ref()
            .map_or(OptStyle::Null, |v| v.get_style())
    }

    /// Return true if the process successful.
    fn is_matched(&self) -> bool {
        self.matches.as_ref().map_or(false, |v| v.is_matched())
    }

    /// Return true if the process need consume an argument.
    fn is_consume_argument(&self) -> bool {
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
            if let Some(uid) = mat.get_matched_uid() {
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
            let style_check = opt.match_style(OptStyle::Cmd)
                || opt.match_style(OptStyle::Main)
                || opt.match_style(OptStyle::Pos);

            if style_check {
                if let Some(mat) = self.matches.as_mut() {
                    if !mat.is_matched() && mat.process(opt)? {
                        self.consume_arg = self.consume_arg || mat.is_consume_argument();
                        return Ok(Some(0));
                    }
                }
            }
        }
        Ok(None)
    }
}
