use std::collections::HashMap;
use std::fmt::Debug;

use super::GenStyle;
use super::{HashMapIter, SliceIter};
use super::{Parser, ReturnValue};
use crate::err::{Error, Result};
use crate::opt::{OptCallback, OptValue};
use crate::proc::Info;
use crate::set::Set;
use crate::uid::{Generator, Uid};

#[derive(Debug, Default)]
pub struct ForwardParser<S, G>
where
    S: Set + Default,
    G: Generator + Debug + Default,
{
    uid_gen: G,

    subscriber_info: Vec<Box<dyn Info>>,

    callback: HashMap<Uid, OptCallback>,

    ret: ReturnValue<S>,

    gen_style_order: Vec<GenStyle>,
}

impl<S, G> ForwardParser<S, G>
where
    S: Set + Default,
    G: Generator + Debug + Default,
{
    pub fn new(uid_gen: G) -> Self {
        Self {
            uid_gen,
            subscriber_info: vec![],
            callback: HashMap::new(),
            ret: ReturnValue::default(),
            gen_style_order: vec![],
        }
    }

    pub fn add_gen_style(&mut self, style: GenStyle) -> &mut Self {
        self.gen_style_order.push(style);
        self
    }
}

#[async_trait::async_trait(?Send)]
impl<S, G> Parser<S> for ForwardParser<S, G>
where
    S: Set + Default,
    G: Generator + Debug + Default,
{
    fn parse(
        &mut self,
        set: S,
        iter: impl Iterator<Item = String>,
    ) -> Result<Option<ReturnValue<S>>> {
        todo!()
    }

    fn add_callback(&mut self, uid: Uid, callback: OptCallback) {
        todo!()
    }

    fn invoke_callback(&self, uid: Uid, set: &mut S, noa_index: usize) -> Result<Option<OptValue>> {
        todo!()
    }

    fn subscriber_iter(&self) -> SliceIter<'_, Box<dyn Info>> {
        self.subscriber_info.iter()
    }

    fn reg_subscriber(&mut self, info: Box<dyn Info>) {
        self.subscriber_info.push(info);
    }

    fn clr_subscriber(&mut self) {
        self.subscriber_info.clear();
    }

    fn reset(&mut self) {
        todo!()
    }

    fn callback_iter(&self) -> HashMapIter<'_, Uid, OptCallback> {
        self.callback.iter()
    }
}
