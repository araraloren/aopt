use crate::opt::Creator;
use crate::set::Set;
use crate::set::SetExt;
use crate::Error;
use crate::Uid;

pub trait SetIndex<S: Set> {
    fn ref_from<'a>(&self, set: &'a S) -> Result<&'a <S::Ctor as Creator>::Opt, Error>;

    fn mut_from<'a>(&self, set: &'a mut S) -> Result<&'a mut <S::Ctor as Creator>::Opt, Error>;
}

macro_rules! impl_num_index_for {
    ($num:ty) => {
        impl<S: Set> SetIndex<S> for $num {
            fn ref_from<'a>(&self, set: &'a S) -> Result<&'a <S::Ctor as Creator>::Opt, Error> {
                set.opt(*self as Uid)
            }

            fn mut_from<'a>(
                &self,
                set: &'a mut S,
            ) -> Result<&'a mut <S::Ctor as Creator>::Opt, Error> {
                set.opt_mut(*self as Uid)
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
