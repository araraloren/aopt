use super::Ctx;
use crate::ser::Services;
use crate::Error;
use crate::Uid;

/// Implement the trait if your want use your type in the
/// [`Callback`](super::Callback) of [`InvokeService`](crate::ser::InvokeService).
pub trait ExtractCtx<Set>
where
    Self: Sized,
{
    type Error: Into<Error>;

    fn extract(uid: Uid, set: &Set, ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error>;
}

impl<Set> ExtractCtx<Set> for ()
where
    Set: crate::set::Set,
{
    type Error = Error;

    fn extract(_uid: Uid, _set: &Set, _ser: &Services, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(())
    }
}

/// Supress the error result.
/// Return the `Ok(Some(T))` if successful, otherwise return `Ok(None)`.
impl<T, Err, Set> ExtractCtx<Set> for Option<T>
where
    Err: Into<Error>,
    Set: crate::set::Set,
    T: ExtractCtx<Set, Error = Err>,
{
    type Error = Err;

    fn extract(uid: Uid, set: &Set, ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
        match T::extract(uid, set, ser, ctx) {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }
}

macro_rules! impl_extracter_for {
    ($($arg:ident)*) => {
        impl<Set, $($arg,)*> ExtractCtx<Set> for ($($arg,)*)
        where
            $(
                $arg: ExtractCtx<Set, Error = Error>,
            )*
        {
            type Error = Error;

            fn extract(uid: Uid, set: &Set, ser: &Services, ctx: &Ctx) -> Result<Self, Self::Error> {
                Ok(($($arg::extract(uid, set, ser, ctx)?,)*))
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

impl_extracter_for!(A B C D E F G H I J K L M);

impl_extracter_for!(A B C D E F G H I J K L M N);

impl_extracter_for!(A B C D E F G H I J K L M N O);

impl_extracter_for!(A B C D E F G H I J K L M N O P);

impl_extracter_for!(A B C D E F G H I J K L M N O P Q);
