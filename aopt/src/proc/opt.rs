use super::Matcher;

use crate::ctx::Context;
use crate::err::Result;
use crate::opt::Style;
use crate::set::Set;
use crate::uid::Uid;

/// The [`Matcher`] using for [`Opt`](crate::opt::Opt).
#[derive(Debug, Default)]
pub struct OptMatcher {
    uid: Uid,

    context: Vec<Box<dyn Context>>,

    consoume_argument: bool,
}

impl From<Uid> for OptMatcher {
    fn from(uid: Uid) -> Self {
        Self {
            uid,
            ..Self::default()
        }
    }
}

impl Matcher for OptMatcher {
    fn get_uid(&self) -> Uid {
        self.uid
    }

    fn add_ctx(&mut self, ctx: Box<dyn Context>) {
        if let Some(last) = self.context.last() {
            // make sure the style are the same
            assert_eq!(last.get_style(), ctx.get_style());
        }
        self.context.push(ctx);
    }

    fn get_ctx(&self, index: usize) -> Option<&Box<dyn Context>> {
        self.context.get(index)
    }

    fn get_ctx_mut(&mut self, index: usize) -> Option<&mut Box<dyn Context>> {
        self.context.get_mut(index)
    }

    fn get_style(&self) -> Style {
        self.context.last().map_or(Style::Null, |v| v.get_style())
    }

    fn process<S: Set>(&mut self, uid: Uid, set: &mut S) -> Result<Option<&mut Box<dyn Context>>> {
        if let Some(opt) = set.get_opt_mut(uid) {
            info!(?uid, "process opt");
            for ctx in self.context.iter_mut() {
                if !ctx.is_matched() {
                    if ctx.process(opt.as_mut())? {
                        self.consoume_argument =
                            self.consoume_argument || ctx.is_comsume_argument();
                        return Ok(Some(ctx));
                    }
                }
            }
        }
        Ok(None)
    }

    fn undo<S: Set>(&mut self, set: &mut S) {
        self.context.iter_mut().for_each(|v| {
            if let Some(uid) = v.get_matched_uid() {
                if let Some(opt) = set.get_opt_mut(uid) {
                    v.undo(opt.as_mut());
                }
            }
        });
    }

    fn is_matched(&self) -> bool {
        self.context
            .iter()
            .fold(true, |acc, x| acc && x.is_matched())
    }

    fn is_comsume_argument(&self) -> bool {
        self.consoume_argument
    }

    fn quit(&self) -> bool {
        self.is_matched()
    }

    fn reset(&mut self) {}

    fn len(&self) -> usize {
        self.context.len()
    }
}
