use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::ops::{Deref, DerefMut};

use crate::arg::ArgStream;
use crate::err::Result;
use crate::opt::OptCallback;
use crate::parser::Parser;
use crate::parser::Policy;
use crate::parser::Service;
use crate::set::Set;
use crate::uid::Uid;

#[derive(Debug)]
pub struct SingleApp<S, SS, P>
where
    S: Set,
    SS: Service,
    P: Policy<S, SS>,
{
    name: String,
    parser: Parser<S, SS, P>,
}

impl<S, SS, P> Default for SingleApp<S, SS, P>
where
    S: Set + Default,
    SS: Service + Default,
    P: Policy<S, SS> + Default,
{
    fn default() -> Self {
        Self {
            name: "singleapp".into(),
            parser: Parser::<S, SS, P>::default(),
        }
    }
}

impl<S, SS, P> SingleApp<S, SS, P>
where
    S: Set,
    SS: Service,
    P: Policy<S, SS>,
{
    pub fn new(name: String, parser: Parser<S, SS, P>) -> Self {
        Self { name, parser }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_parser(mut self, parser: Parser<S, SS, P>) -> Self {
        self.parser = parser;
        self
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_parser(&self) -> &Parser<S, SS, P> {
        &self.parser
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_parser(&mut self, parser: Parser<S, SS, P>) {
        self.parser = parser;
    }

    pub fn add_callback(&mut self, uid: Uid, callback: OptCallback) {
        self.parser.add_callback(uid, callback);
    }

    pub fn get_callback(&self) -> &HashMap<Uid, RefCell<OptCallback>> {
        self.parser.get_service().get_callback()
    }

    pub fn get_callback_mut(&mut self) -> &mut HashMap<Uid, RefCell<OptCallback>> {
        self.parser.get_service_mut().get_callback_mut()
    }

    pub fn run_mut<RET, F: FnMut(bool, &mut SingleApp<S, SS, P>) -> Result<RET>>(
        &mut self,
        iter: impl Iterator<Item = String>,
        mut r: F,
    ) -> Result<RET> {
        let parser = &mut self.parser;
        let ret = parser.parse(&mut ArgStream::from(iter))?;

        r(ret, self)
    }

    pub async fn run_async_mut<
        RET,
        FUT: Future<Output = Result<RET>>,
        F: FnMut(bool, &mut SingleApp<S, SS, P>) -> FUT,
    >(
        &mut self,
        iter: impl Iterator<Item = String>,
        mut r: F,
    ) -> Result<RET> {
        let parser = &mut self.parser;
        let async_ret;

        match parser.parse(&mut ArgStream::from(iter)) {
            Ok(ret) => {
                let ret = r(ret, self).await;

                async_ret = ret;
            }
            Err(e) => {
                async_ret = Err(e);
            }
        }
        async_ret
    }
}

impl<S, SS, P> SingleApp<S, SS, P>
where
    S: Set + Default,
    SS: Service + Default,
    P: Policy<S, SS> + Default,
{
    pub fn run<RET, F: FnMut(bool, SingleApp<S, SS, P>) -> Result<RET>>(
        &mut self,
        iter: impl Iterator<Item = String>,
        mut r: F,
    ) -> Result<RET> {
        let parser = &mut self.parser;
        let ret = parser.parse(&mut ArgStream::from(iter))?;

        r(ret, std::mem::take(self))
    }

    pub async fn run_async<
        RET,
        FUT: Future<Output = Result<RET>>,
        F: FnMut(bool, SingleApp<S, SS, P>) -> FUT,
    >(
        &mut self,
        iter: impl Iterator<Item = String>,
        mut r: F,
    ) -> Result<RET> {
        let parser = &mut self.parser;
        let async_ret;

        match parser.parse(&mut ArgStream::from(iter)) {
            Ok(ret) => {
                let ret = r(ret, std::mem::take(self)).await;

                async_ret = ret;
            }
            Err(e) => {
                async_ret = Err(e);
            }
        }
        async_ret
    }
}

// Implement Deref/DerefMut for SingleApp.
impl<S, SS, P> Deref for SingleApp<S, SS, P>
where
    S: Set,
    SS: Service,
    P: Policy<S, SS>,
{
    type Target = Parser<S, SS, P>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

// Implement Deref/DerefMut for SingleApp.
impl<S, SS, P> DerefMut for SingleApp<S, SS, P>
where
    S: Set,
    SS: Service,
    P: Policy<S, SS>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}
