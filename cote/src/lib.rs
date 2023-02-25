use aopt::prelude::*;
use aopt::Error;

use std::fmt::Debug;

pub trait ParserIntoExtension<'zlifetime, P>
where
    P::Ser: 'zlifetime,
    P::Set: Set + 'zlifetime,
    P::Error: Into<aopt::Error>,
    P: Policy + APolicyExt<P> + Default,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    fn into_parser() -> Result<Parser<'zlifetime, P>, Error> {
        let mut parser = Parser::<'zlifetime, P>::new(P::default());
        Self::update(&mut parser)?;
        Ok(parser)
    }
    fn update(parser: &mut Parser<'zlifetime, P>) -> Result<(), Error>;
}

pub trait ParserExtractExtension<'zlifetime, S>
where
    S: SetValueFindExt,
{
    fn try_extract(set: &'zlifetime mut S) -> Result<Self, aopt::Error>
    where
        Self: Sized;
}

// use aopt::prelude::*;
// use aopt::{Error, HashMap, RawVal};

// pub struct Cote<P: Policy, E: Policy = NullPolicy<ASet>> {
//     name: String,

//     auto_help: bool,

//     maj_parser: Parser<P>,

//     sub_parser: HashMap<String, Parser<E>>,

//     parser_map: HashMap<String, Vec<String>>,

//     parser_ret: HashMap<String, Option<()>>,
// }

// impl<P, E> Default for Cote<P, E>
// where
//     P::Set: Default,
//     E::Set: Default,
//     P: Policy + APolicyExt<P::Set> + Default,
//     E: Policy + Default,
// {
//     fn default() -> Self {
//         Self {
//             name: "cote".to_owned(),
//             auto_help: Default::default(),
//             maj_parser: Default::default(),
//             sub_parser: Default::default(),
//             parser_map: Default::default(),
//             parser_ret: Default::default(),
//         }
//     }
// }

// impl<P, E> Debug for Cote<P, E>
// where
//     P::Set: Debug,
//     E::Set: Debug,
//     P::Ret: Debug,
//     E::Ret: Debug,
//     P: Policy + APolicyExt<P::Set> + Debug,
//     E: Policy + Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Cote")
//             .field("name", &self.name)
//             .field("auto_help", &self.auto_help)
//             .field("maj_parser", &self.maj_parser)
//             .field("sub_parser", &self.sub_parser)
//             .field("parser_map", &self.parser_map)
//             .field("parser_ret", &self.parser_ret)
//             .finish()
//     }
// }

// impl<P: Policy, E: Policy> Cote<P, E> {
//     pub fn maj_parser(&self) -> &Parser<P> {
//         &self.maj_parser
//     }

//     pub fn maj_parser_mut(&mut self) -> &mut Parser<P> {
//         &mut self.maj_parser
//     }

//     pub fn add_sub_parser<S: Into<String>>(&mut self, name: S, parser: Parser<E>) -> &mut Self {
//         self.sub_parser.insert(name.into(), parser);
//         self
//     }

//     pub fn link_parser<S: Into<String>>(&mut self, name: S, sub_parser: Vec<S>) -> &mut Self {
//         let sub_parser = sub_parser.into_iter().map(|v| v.into()).collect();

//         self.parser_map.insert(name.into(), sub_parser);
//         self
//     }
//     pub fn contain_link(&mut self, name: &str) -> bool {
//         self.parser_map.contains_key(name)
//             .ok_or_else(|| Error::raise_error(format!("Invalid sub parser name: {}", name)))
//     }

//     pub fn sub_parser(&self, name: &str) -> Result<&Parser<E>, Error> {
//         println!("--> get sub parser -> {}", name);
//         self.sub_parser
//             .get(name)
//             .ok_or_else(|| Error::raise_error(format!("Invalid sub parser name: {}", name)))
//     }
// }

// impl<P, E> Cote<P, E>
// where
//     E: Policy<Error = Error>,
//     P: Policy<Error = Error>,
// {
//     pub fn inject<'a, I: Inject<'a, Self, Error = Error>>(&'a mut self) -> Result<I::Ret, Error> {
//         I::inject(self)
//     }

//     pub fn inject_from<'a, I: InjectFrom<'a, Self, Error = Error>>(
//         &'a mut self,
//         mut meta: I,
//     ) -> Result<I::Ret, Error> {
//         meta.inject_from(self)
//     }

//     pub fn extract_val<'a, I: ExtractVal<'a, Self, Error = Error>>(&'a self) -> Result<I, Error> {
//         I::extract_new(self)
//     }

//     pub fn extract_val_for<'a, 'b, I: ExtractValFor<'a, Self, Error = Error>>(
//         &'a self,
//         val: &'b mut I,
//     ) -> Result<&'b mut I, Error> {
//         val.extract_for(&self.name, self)
//     }

//     pub fn extract_val_mut<'a, I: ExtractValMut<'a, Self, Error = Error>>(
//         &'a mut self,
//     ) -> Result<I, Error> {
//         I::extract_new_mut(self)
//     }

//     pub fn extract_val_for_mut<'a, 'b, I: ExtractValForMut<'a, Self, Error = Error>>(
//         &'a mut self,
//         val: &'b mut I,
//     ) -> Result<&'b mut I, Error> {
//         let name = self.name.clone();

//         val.extract_for_mut(&name, self)
//     }
// }

// // impl<P, E> Cote<P, E>
// // where
// //     P: Policy,
// //     E::Set: 'static,
// //     E: Policy<Error = Error>,
// //     SetOpt<E::Set>: Opt,
// //     E::Set: Set + OptValidator + OptParser,
// //     <E::Set as OptParser>::Output: Information,
// //     SetCfg<E::Set>: Config + ConfigValue + Default,
// // {
// //     pub fn inject_sub<'a, I: Inject<'a, Parser<E>, Error = Error>>(
// //         &'a mut self,
// //         name: &str,
// //     ) -> Result<I::Ret, Error> {
// //         I::inject(self._sub_parser_mut(name)?)
// //     }

// //     pub fn inject_sub_from<'a, I: InjectFrom<'a, Parser<E>, Error = Error>>(
// //         &'a mut self,
// //         mut meta: I,
// //         name: &str,
// //     ) -> Result<I::Ret, Error> {
// //         meta.inject_from(self._sub_parser_mut(name)?)
// //     }

// //     pub fn extract_subval<'a, I: ExtractVal<'a, Parser<E>, Error = Error>>(
// //         &'a self,
// //         name: &str,
// //     ) -> Result<I, Error> {
// //         I::extract_new(self._sub_parser(name)?)
// //     }

// //     pub fn extract_subval_for<'a, 'b, I: ExtractValFor<'a, Parser<E>, Error = Error>>(
// //         &'a self,
// //         val: &'b mut I,
// //         name: &str,
// //     ) -> Result<&'b mut I, Error> {
// //         val.extract_for(name, self._sub_parser(name)?)
// //     }

// //     pub fn extract_subval_mut<'a, I: ExtractValMut<'a, Parser<E>, Error = Error>>(
// //         &'a mut self,
// //         name: &str,
// //     ) -> Result<I, Error> {
// //         I::extract_new_mut(self._sub_parser_mut(name)?)
// //     }

// //     pub fn extract_subval_for_mut<'a, 'b, I: ExtractValForMut<'a, Parser<E>, Error = Error>>(
// //         &'a mut self,
// //         val: &'b mut I,
// //         name: &str,
// //     ) -> Result<&'b mut I, Error> {
// //         val.extract_for_mut(name, self._sub_parser_mut(name)?)
// //     }
// // }

// impl<P, E> Cote<P, E>
// where
//     P::Ret: Into<Args>,
//     E::Ret: Into<Args>,
//     P: Policy<Error = Error>,
//     E: Policy<Error = Error>,
// {
//     pub fn run_sub_parser(
//         &mut self,
//         args: aopt::Arc<Args>,
//         name: &str,
//         reset: bool,
//     ) -> Result<Option<String>, Error> {
//         let sub_parser_names = self.parser_map.get(name);
//         let mut ret = Ok(None);

//         if let Some(sub_parser_names) = sub_parser_names {
//             for parser_name in sub_parser_names {
//                 let sub_parser = self.sub_parser.get_mut(parser_name).ok_or_else(|| {
//                     Error::raise_error(format!("Invalid sub parser name: {}", parser_name,))
//                 })?;

//                 println!("--> running parser {}", &parser_name);
//                 if reset {
//                     sub_parser.clear_all()?;
//                 }
//                 sub_parser.init()?;
//                 match sub_parser.parse(args.clone()) {
//                     Ok(Some(_)) => {
//                         println!("--> running parser ok -> previous error {:?}", &ret);
//                         ret = Ok(Some(parser_name.clone()));
//                         break;
//                     }
//                     Ok(None) => {}
//                     Err(e) => {
//                         if e.is_failure() {
//                             // if e is failure, continue parsing
//                             ret = Err(e);
//                         } else {
//                             return Err(e);
//                         }
//                     }
//                 }
//             }
//         }
//         ret
//     }

//     fn _convert2args<T: Policy<Error = Error>>(
//         parser: &mut Parser<T>,
//     ) -> Result<aopt::Arc<Args>, Error>
//     where
//         T::Ret: Into<Args>,
//     {
//         Ok(aopt::Arc::new(
//             parser
//                 .take_retval()
//                 .ok_or_else(|| Error::raise_error("can not take return value of pre parser"))?
//                 .into(),
//         ))
//     }

//     pub fn run_mut_with<'a, 'b, I, R, F>(
//         &'a mut self,
//         iter: impl Iterator<Item = I>,
//         mut r: F,
//     ) -> Result<R, Error>
//     where
//         'a: 'b,
//         I: Into<RawVal>,
//         F: FnMut(Result<Option<String>, Error>, &'b mut Cote<P, E>) -> Result<R, Error>,
//     {
//         let args = iter.map(|v| v.into());
//         let mut ret;

//         // initialize the option value
//         self.maj_parser.init()?;

//         let pre_parser_ret = self.maj_parser.parse(aopt::Arc::new(Args::from(args)));

//         ret = pre_parser_ret.map(|ret| ret.map(|_| self.name.clone()));
//         println!("--> pre parser ==> {:?}", &ret);
//         if matches!(ret, Ok(Some(_))) {
//             // pre parser name
//             let mut next_parser = self.name.clone();
//             let mut args = Self::_convert2args(&mut self.maj_parser)?;

//             loop {
//                 println!("--> ready running parser = {}", &next_parser);
//                 if !self.contain_link(&next_parser) {
//                     break;
//                 }
//                 ret = self.run_sub_parser(args.clone(), &next_parser, true);
//                 if let Ok(Some(sub_parser_name)) = &ret {
//                     next_parser = sub_parser_name.clone();
//                     args = Self::_convert2args(self.sub_parser_mut(&next_parser)?)?;
//                 } else {
//                     break;
//                 }
//             }
//         }
//         r(ret, self)
//     }

//     pub fn run_mut<'a, 'b, R, F>(&'a mut self, r: F) -> Result<R, Error>
//     where
//         'a: 'b,
//         F: FnMut(Result<Option<String>, Error>, &'b mut Cote<P, E>) -> Result<R, Error>,
//     {
//         self.run_mut_with(std::env::args().skip(1), r)
//     }
// }
