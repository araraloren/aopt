use crate::Error;

pub trait Handler<Set, Ser, Args> {
    type Output;
    type Error: Into<Error>;

    fn invoke(
        &mut self,
        set: &mut Set,
        ser: &mut Ser,
        args: Args,
    ) -> Result<Self::Output, Self::Error>;
}

macro_rules! impl_handler_for {
    ($($arg:ident)*) => {
        impl<Set, Ser, Func, Out, Err, $($arg,)*> Handler<Set, Ser, ($($arg,)*)> for Func
        where
            Err: Into<Error>,
            Func: FnMut(&mut Set, &mut Ser, $($arg),*) -> Result<Out, Err>,
        {
            type Output = Out;
            type Error = Err;

            #[inline]
            #[allow(non_snake_case)]
            fn invoke(&mut self, set: &mut Set, ser: &mut Ser, ($($arg,)*): ($($arg,)*)) -> Result<Self::Output, Self::Error> {
                (self)(set, ser, $($arg,)*)
            }
        }
    };
}

impl_handler_for!();

impl_handler_for!(A);

impl_handler_for!(A B);

impl_handler_for!(A B C);

impl_handler_for!(A B C D);

impl_handler_for!(A B C D E);

impl_handler_for!(A B C D E F);

impl_handler_for!(A B C D E F G);

impl_handler_for!(A B C D E F G H);

impl_handler_for!(A B C D E F G H I);

impl_handler_for!(A B C D E F G H I J);

impl_handler_for!(A B C D E F G H I J K);

impl_handler_for!(A B C D E F G H I J K L);

impl_handler_for!(A B C D E F G H I J K L M);

impl_handler_for!(A B C D E F G H I J K L M N);

impl_handler_for!(A B C D E F G H I J K L M N O);

impl_handler_for!(A B C D E F G H I J K L M N O P);

impl_handler_for!(A B C D E F G H I J K L M N O P Q);
