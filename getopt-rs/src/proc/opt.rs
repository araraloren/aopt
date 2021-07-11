use crate::ctx::Context;
use crate::err::Result;
use crate::opt::Opt;
use crate::proc::Proc;
use crate::uid::Uid;

#[derive(Debug, Default)]
pub struct OptCtxProc {
    uid: Uid,

    context: Vec<Box<dyn Context>>,

    consoume_argument: bool,
}

impl From<Uid> for OptCtxProc {
    fn from(uid: Uid) -> Self {
        Self {
            uid,
            ..Self::default()
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Proc for OptCtxProc {
    fn uid(&self) -> Uid {
        self.uid
    }

    fn add_ctx(&mut self, ctx: Box<dyn Context>) {
        self.context.push(ctx);
    }

    fn get_ctx(&self, index: usize) -> Option<&Box<dyn Context>> {
        self.context.get(index)
    }

    #[cfg(not(feature = "async"))]
    fn process(&mut self, opt: &mut dyn Opt) -> Result<Option<usize>> {
        for ctx in self.context.iter_mut() {
            if !ctx.is_matched() {
                if ctx.match_opt(opt) {
                    if ctx.process_opt(opt)? {
                        self.consoume_argument =
                            self.consoume_argument || ctx.is_comsume_argument();
                        return Ok(ctx.get_matched_index());
                    }
                }
            }
        }
        Ok(None)
    }

    #[cfg(feature = "async")]
    async fn process(&mut self, opt: &mut dyn Opt) -> Result<Option<usize>> {
        for ctx in self.context.iter_mut() {
            if !ctx.is_matched() {
                if ctx.match_opt(opt) {
                    if ctx.process_opt(opt)? {
                        self.consoume_argument =
                            self.consoume_argument || ctx.is_comsume_argument();
                        return Ok(ctx.get_matched_index());
                    }
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
