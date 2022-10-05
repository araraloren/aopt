use super::Match;
use super::OptMatch;
use super::Process;
use crate::opt::Opt;
use crate::opt::OptStyle;
use crate::set::Set;
use crate::Error;
use crate::Uid;

/// OptProcess matching the [`Opt`] against [`OptMatch`].
#[derive(Debug)]
pub struct OptProcess<S: Set> {
    matches: Vec<OptMatch<S>>,

    consume_arg: bool,
}

impl<S> OptProcess<S>
where
    S: Set,
{
    pub fn new(matches: Vec<OptMatch<S>>) -> Self {
        Self {
            matches,
            consume_arg: false,
        }
    }
}

impl<S: Set> Process<OptMatch<S>> for OptProcess<S>
where
    S::Opt: Opt,
{
    type Set = S;

    type Error = Error;

    fn reset(&mut self) {
        self.matches.iter_mut().for_each(|v| v.reset());
    }

    /// Return true if the process successful.
    fn quit(&self) -> bool {
        self.is_matched()
    }

    /// Return the count of [`OptMatch`].
    fn count(&self) -> usize {
        self.matches.len()
    }

    /// Return the [`OptStyle`] of OptProcess.
    fn get_style(&self) -> OptStyle {
        self.matches
            .last()
            .map_or(OptStyle::Null, |v| v.get_style())
    }

    /// Return true if the process successful.
    fn is_matched(&self) -> bool {
        self.matches.iter().all(|v| v.is_matched())
    }

    /// Return true if the process need consume an argument.
    fn is_consume_argument(&self) -> bool {
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
            if let Some(uid) = mat.get_matched_uid() {
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
            let style_check = opt.match_style(OptStyle::Argument)
                || opt.match_style(OptStyle::Boolean)
                || opt.match_style(OptStyle::Combined);

            if style_check {
                for (index, mat) in self.matches.iter_mut().enumerate() {
                    if !mat.is_matched() && mat.process(opt)? {
                        self.consume_arg = self.consume_arg || mat.is_consume_argument();
                        return Ok(Some(index));
                    }
                }
            }
        }
        Ok(None)
    }
}
