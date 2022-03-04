use super::Set;
use crate::opt::Opt;
use crate::uid::Uid;

pub trait SetIndex<T: Set> {
    fn ref_from<'s>(&self, set: &'s T) -> Option<&'s Box<dyn Opt>>;

    fn mut_from<'s>(&self, set: &'s mut T) -> Option<&'s mut Box<dyn Opt>>;
}

macro_rules! impl_num_index_for {
    ($num:ty) => {
        impl<T: Set> SetIndex<T> for $num {
            fn ref_from<'s>(&self, set: &'s T) -> Option<&'s Box<dyn Opt>> {
                set.get_opt(*self as Uid)
            }

            fn mut_from<'s>(&self, set: &'s mut T) -> Option<&'s mut Box<dyn Opt>> {
                set.get_opt_mut(*self as Uid)
            }
        }
    };
}

impl_num_index_for!(i8);
impl_num_index_for!(i16);
impl_num_index_for!(i32);
impl_num_index_for!(i64);
impl_num_index_for!(i128);
impl_num_index_for!(u8);
impl_num_index_for!(u16);
impl_num_index_for!(u32);
impl_num_index_for!(u64);
impl_num_index_for!(u128);
impl_num_index_for!(usize);
impl_num_index_for!(isize);

impl<'a, T: Set> SetIndex<T> for &'a str {
    fn ref_from<'s>(&self, set: &'s T) -> Option<&'s Box<dyn Opt>> {
        set.find(self)
            .expect(&format!("Can not find current option: {}", self))
    }

    fn mut_from<'s>(&self, set: &'s mut T) -> Option<&'s mut Box<dyn Opt>> {
        set.find_mut(self)
            .expect(&format!("Can not find current option: {}", self))
    }
}

impl<T: Set> SetIndex<T> for String {
    fn ref_from<'s>(&self, set: &'s T) -> Option<&'s Box<dyn Opt>> {
        set.find(self)
            .expect(&format!("Can not find current option: {}", self))
    }

    fn mut_from<'s>(&self, set: &'s mut T) -> Option<&'s mut Box<dyn Opt>> {
        set.find_mut(self)
            .expect(&format!("Can not find current option: {}", self))
    }
}
