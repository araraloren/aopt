use super::Context;
use crate::prelude::Services;
use crate::set::Set;
use crate::Error;
use crate::Uid;

/// Implement the trait if your want use your type in the
/// [`Callback`](super::Callback) of [`InvokeService`](crate::ser::InvokeService).
pub trait ExtractFromCtx<S>
where
    Self: Sized,
    S: Set,
{
    type Error: Into<Error>;

    fn extract_from(
        uid: Uid,
        set: &S,
        ser: &mut Services,
        ctx: Context,
    ) -> Result<Self, Self::Error>;
}

impl<S> ExtractFromCtx<S> for ()
where
    S: Set,
{
    type Error = Error;

    fn extract_from(
        _uid: Uid,
        _set: &S,
        _ser: &mut Services,
        _ctx: Context,
    ) -> Result<Self, Self::Error> {
        Ok(())
    }
}

macro_rules! impl_extracter_for {
    ($($arg:ident)*) => {
        impl<S, $($arg,)*> ExtractFromCtx<S> for ($($arg,)*)
        where
            S: Set,
            $(
                $arg: ExtractFromCtx<S, Error = Error> + 'static,
            )*
        {
            type Error = Error;

            fn extract_from(uid: Uid, set: &S, ser: &mut Services, ctx: Context) -> Result<Self, Self::Error> {
                Ok(($($arg::extract_from(uid, set, ser, ctx.clone())?,)*))
            }
        }
    };
}

impl_extracter_for!(A);

impl_extracter_for!(A B);

impl_extracter_for!(A B C);

impl_extracter_for!(A B C D);

impl_extracter_for!(A B C D E);

impl_extracter_for!(A B C D E F);

impl_extracter_for!(A B C D E F G);

impl_extracter_for!(A B C D E F G H);

impl_extracter_for!(A B C D E F G H I);

impl_extracter_for!(A B C D E F G H I J);

impl_extracter_for!(A B C D E F G H I J K);

impl_extracter_for!(A B C D E F G H I J K L);
