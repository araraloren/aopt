use crate::ctx::Context;
use crate::err::Result;
use crate::opt::Opt;
use crate::opt::Style;
use crate::proc::Proc;
use crate::uid::Uid;

#[derive(Debug, Default)]
pub struct NonOptCtxProc {
    uid: Uid,

    context: Option<Box<dyn Context>>,

    consoume_argument: bool,
}

impl From<Uid> for NonOptCtxProc {
    fn from(uid: Uid) -> Self {
        Self {
            uid,
            ..Self::default()
        }
    }
}

impl Proc for NonOptCtxProc {
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

    fn process(&mut self, opt: &mut dyn Opt) -> Result<Option<usize>> {
        if opt.match_style(Style::Cmd)
            || opt.match_style(Style::Main)
            || opt.match_style(Style::Pos)
        {
            if let Some(ctx) = self.context.as_mut() {
                if !ctx.is_matched() {
                    if ctx.match_opt(opt) {
                        if ctx.process_opt(opt)? {
                            self.consoume_argument =
                                self.consoume_argument || ctx.is_comsume_argument();
                            return Ok(ctx.get_matched_index());
                        }
                    }
                } else {
                    return Ok(ctx.get_matched_index());
                }
            }
        }
        Ok(None)
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
