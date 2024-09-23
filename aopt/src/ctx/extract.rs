use crate::Error;

/// Implement the trait if your want use your type in the [`Invoker`](crate::ctx::Invoker).
/// Return an [`Error::sp_extract`] if any error occured.
pub trait Extract<Set, Ser, Ctx>
where
    Self: Sized,
{
    type Error: Into<Error>;

    fn extract(set: &Set, ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error>;
}

impl<Set, Ser, Ctx> Extract<Set, Ser, Ctx> for () {
    type Error = Error;

    fn extract(_set: &Set, _ser: &Ser, _ctx: &Ctx) -> Result<Self, Self::Error> {
        Ok(())
    }
}

/// Supress the error result.
/// Return the `Ok(Some(T))` if successful, otherwise return `Ok(None)`.
impl<T, Err, Set, Ser, Ctx> Extract<Set, Ser, Ctx> for Option<T>
where
    Err: Into<Error>,
    T: Extract<Set, Ser, Ctx, Error = Err>,
{
    type Error = Err;

    fn extract(set: &Set, ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
        match T::extract(set, ser, ctx) {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }
}

macro_rules! impl_extracter_for {
    ($($arg:ident)*) => {
        impl<Set, Ser, Ctx, $($arg,)*> Extract<Set, Ser, Ctx> for ($($arg,)*)
        where
            $(
                $arg: Extract<Set, Ser, Ctx, Error = Error>,
            )*
        {
            type Error = Error;

            fn extract(set: &Set, ser: &Ser, ctx: &Ctx) -> Result<Self, Self::Error> {
                Ok(($($arg::extract(set, ser, ctx)?,)*))
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
