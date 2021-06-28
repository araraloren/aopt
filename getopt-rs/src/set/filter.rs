
use super::info::FilterInfo;
use super::Set;

#[derive(Debug)]
pub struct Filter<'a> {
    set: &'a dyn Set,

    info: FilterInfo,
}

impl<'a> Filter<'a> {
    pub fn new(set: &'a dyn Set, info: FilterInfo) -> Self {
        Self {
            set, info,
        }
    }
}

#[derive(Debug)]
pub struct FilterMut<'a> {
    set: &'a mut dyn Set,

    info: FilterInfo,
}

impl<'a> FilterMut<'a> {
    pub fn new(set: &'a mut dyn Set, info: FilterInfo) -> Self {
        Self {
            set, info,
        }
    }
}
