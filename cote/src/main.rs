use std::ops::Deref;
use std::ops::DerefMut;

use aopt::Error;
use cote::prelude::aopt;
use cote::prelude::derive::*;

fn main() -> Result<(), aopt::Error> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    Ok(())
}

pub type DeleyCreator = Creator<DelayOpt, OptConfig, aopt::Error>;

#[derive(Debug)]
pub struct DelayOpt {
    opt: AOpt,

    delay: bool,
}

impl DelayOpt {
    pub fn with_delay(mut self, delay: bool) -> Self {
        self.delay = delay;
        self
    }

    pub fn set_delay(&mut self, delay: bool) -> &mut Self {
        self.delay = delay;
        self
    }

    pub fn is_delay(&self) -> bool {
        self.delay
    }
}

impl Deref for DelayOpt {
    type Target = AOpt;

    fn deref(&self) -> &Self::Target {
        &self.opt
    }
}

impl DerefMut for DelayOpt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.opt
    }
}

impl Opt for DelayOpt {
    fn reset(&mut self) {
        Opt::reset(&mut self.opt)
    }

    fn uid(&self) -> Uid {
        Opt::uid(&self.opt)
    }

    fn name(&self) -> &aopt::Str {
        Opt::name(&self.opt)
    }

    fn r#type(&self) -> &std::any::TypeId {
        Opt::r#type(&self.opt)
    }

    fn hint(&self) -> &aopt::Str {
        Opt::hint(&self.opt)
    }

    fn help(&self) -> &aopt::Str {
        Opt::hint(&self.opt)
    }

    fn valid(&self) -> bool {
        Opt::valid(&self.opt)
    }

    fn matched(&self) -> bool {
        Opt::matched(&self.opt)
    }

    fn force(&self) -> bool {
        Opt::force(&self.opt)
    }

    fn action(&self) -> &Action {
        Opt::action(&self.opt)
    }

    fn index(&self) -> Option<&Index> {
        Opt::index(&self.opt)
    }

    fn alias(&self) -> Option<&Vec<aopt::Str>> {
        Opt::alias(&self.opt)
    }

    fn accessor(&self) -> &ValAccessor {
        Opt::accessor(&self.opt)
    }

    fn accessor_mut(&mut self) -> &mut ValAccessor {
        Opt::accessor_mut(&mut self.opt)
    }

    fn ignore_alias(&self) -> bool {
        Opt::ignore_alias(&self.opt)
    }

    fn ignore_name(&self) -> bool {
        Opt::ignore_name(&self.opt)
    }

    fn ignore_index(&self) -> bool {
        Opt::ignore_index(&self.opt)
    }

    fn set_uid(&mut self, uid: Uid) {
        Opt::set_uid(&mut self.opt, uid)
    }

    fn set_matched(&mut self, matched: bool) {
        Opt::set_matched(&mut self.opt, matched)
    }

    fn mat_style(&self, style: Style) -> bool {
        Opt::mat_style(&self.opt, style)
    }

    fn mat_force(&self, force: bool) -> bool {
        Opt::mat_force(&self.opt, force)
    }

    fn mat_name(&self, name: Option<&aopt::Str>) -> bool {
        Opt::mat_name(&self.opt, name)
    }

    fn mat_alias(&self, name: &aopt::Str) -> bool {
        Opt::mat_alias(&self.opt, name)
    }

    fn mat_index(&self, index: Option<(usize, usize)>) -> bool {
        Opt::mat_index(&self.opt, index)
    }

    fn init(&mut self) -> Result<(), aopt::Error> {
        Opt::init(&mut self.opt)
    }
}
