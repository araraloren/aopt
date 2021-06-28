
use super::info::CreateInfo;
use super::Set;

#[derive(Debug)]
pub struct Commit<'a> {
    set: &'a dyn Set,

    info: CreateInfo,
}

