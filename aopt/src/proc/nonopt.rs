use super::Matcher;

use crate::ctx::Context;
use crate::err::Result;
use crate::opt::Style;
use crate::set::Set;
use crate::uid::Uid;

#[derive(Debug, Default)]
pub struct NonOptMatcher {
    uid: Uid,

    context: Option<Box<dyn Context>>,

    consoume_argument: bool,
}

impl From<Uid> for NonOptMatcher {
    fn from(uid: Uid) -> Self {
        Self {
            uid,
            ..Self::default()
        }
    }
}

impl Matcher for NonOptMatcher {
    fn uid(&self) -> Uid {
        self.uid
    }

    fn add_ctx(&mut self, ctx: Box<dyn Context>) {
        self.context = Some(ctx);
    }

    fn get_ctx(&self, index: usize) -> Option<&Box<dyn Context>> {
        if index == 0 {
            self.context.as_ref()
        } else {
            None
        }
    }

    fn get_ctx_mut(&mut self, index: usize) -> Option<&mut Box<dyn Context>> {
        if index == 0 {
            self.context.as_mut()
        } else {
            None
        }
    }

    fn process(&mut self, uid: Uid, set: &mut dyn Set) -> Result<Option<&mut Box<dyn Context>>> {
        let opt = set[uid].as_mut();

        info!(?uid, "process nonopt");
        if opt.match_style(Style::Cmd)
            || opt.match_style(Style::Main)
            || opt.match_style(Style::Pos)
        {
            if let Some(ctx) = self.context.as_mut() {
                if !ctx.is_matched() {
                    if ctx.process(opt)? {
                        self.consoume_argument =
                            self.consoume_argument || ctx.is_comsume_argument();
                        return Ok(Some(ctx));
                    }
                } else {
                    return Ok(Some(ctx));
                }
            }
        }
        Ok(None)
    }

    fn undo(&mut self) {
        if let Some(ctx) = self.context.as_mut() {
            ctx.undo();
        }
    }

    fn is_matched(&self) -> bool {
        self.context
            .as_ref()
            .map(|v| v.is_matched())
            .unwrap_or(false)
    }

    fn is_comsume_argument(&self) -> bool {
        self.consoume_argument
    }

    fn quit(&self) -> bool {
        false
    }

    fn reset(&mut self) {
        if let Some(ctx) = self.context.as_mut() {
            ctx.set_matched_index(None);
        }
    }

    fn len(&self) -> usize {
        1
    }
}
