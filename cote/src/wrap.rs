#[macro_export]
macro_rules! wrap_mut_for {
    ($type:ident, $inner:ident) => {
        impl aopt::prelude::Infer for $type {
            type Val = $inner;
        }

        impl<'a> aopt::prelude::InferValueMut<'a> for $type {
            fn infer_fetch<S: aopt::prelude::SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, aopt::Error>
            where
                Self: Sized,
            {
                Ok($type(set.take_val::<$inner>(name)?))
            }
        }
    };
    ($type:ident) => {
        impl aopt::prelude::Infer for $type {
            type Val = $type;
        }

        impl<'a> aopt::prelude::InferValueMut<'a> for $type {
            fn infer_fetch<S: aopt::prelude::SetValueFindExt>(name: &str, set: &'a mut S) -> Result<Self, aopt::Error>
            where
                Self: Sized,
            {
                Ok(set.take_val::<$type>(name)?)
            }
        }
    };
}
