use std::ops::Deref;
use std::ops::DerefMut;

use super::Service;
use crate::astr;
use crate::Str;

#[derive(Debug, Clone, Default)]
pub struct NOAService(Vec<Str>);

impl NOAService {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for NOAService {
    type Target = Vec<Str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NOAService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Service for NOAService {
    fn service_name() -> Str {
        astr("NOAService")
    }
}

impl From<NOAService> for crate::arg::Args {
    fn from(mut v: NOAService) -> Self {
        let value = v.deref_mut();
        Self::from(std::mem::take(value))
    }
}
