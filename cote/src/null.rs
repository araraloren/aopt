// use std::marker::PhantomData;

// use aopt::parser::ReturnVal;
// use aopt::prelude::*;

// #[derive(Debug, Clone, Default)]
// pub struct NullPolicy<S>(PhantomData<S>);

// impl<S: Set> Policy for NullPolicy<S> {
//     type Ret = ReturnVal;

//     type Set = S;

//     type Error = aopt::Error;

//     fn parse(
//         &mut self,
//         _: &mut Self::Set,
//         _: &mut Services,
//         _: aopt::Arc<Args>,
//     ) -> Result<Option<Self::Ret>, Self::Error> {
//         Ok(None)
//     }
// }
