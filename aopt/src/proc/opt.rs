use super::Matcher;

use crate::ctx::Context;
use crate::err::Result;
use crate::set::Set;
use crate::uid::Uid;

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
    fn uid(&self) -> Uid {
        self.uid
    }

    fn add_ctx(&mut self, ctx: Box<dyn Context>) {
        self.context.push(ctx);
    }

    fn get_ctx(&self, index: usize) -> Option<&Box<dyn Context>> {
        self.context.get(index)
    }

    fn get_ctx_mut(&mut self, index: usize) -> Option<&mut Box<dyn Context>> {
        self.context.get_mut(index)
    }

    fn process(&mut self, uid: Uid, set: &mut dyn Set) -> Result<Option<&mut Box<dyn Context>>> {
        let opt = set[uid].as_mut();

        info!(?uid, "process opt");
        for ctx in self.context.iter_mut() {
            if !ctx.is_matched() {
                if ctx.process(opt)? {
                    self.consoume_argument = self.consoume_argument || ctx.is_comsume_argument();
                    return Ok(Some(ctx));
                }
                else {
                    ctx.undo(opt);
                    break;
                }
            }
        }
        Ok(None)
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
