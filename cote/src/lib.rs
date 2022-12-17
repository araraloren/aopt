use std::{
    fmt::Debug,
    future::Future,
    ops::{Deref, DerefMut},
};

use aopt::{prelude::*, RawVal};

pub use aopt::Error;

pub struct Cote<P>
where
    P: Policy,
{
    name: String,

    parser: Parser<P>,
}

impl<P> Debug for Cote<P>
where
    P::Ret: Debug,
    P::Set: Debug,
    P: Policy + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cote")
            .field("name", &self.name)
            .field("parser", &self.parser)
            .finish()
    }
}

impl<P> Default for Cote<P>
where
    P::Set: Default,
    P: Policy + APolicyExt<P::Set> + Default,
{
    fn default() -> Self {
        Self {
            name: "Cote".to_owned(),
            parser: Parser::default(),
        }
    }
}

impl<P: Policy> Deref for Cote<P> {
    type Target = Parser<P>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl<P: Policy> DerefMut for Cote<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

impl<P> Cote<P>
where
    P: Policy + APolicyExt<P::Set>,
{
    pub fn new<S: Into<String>>(name: S, policy: P) -> Self {
        Self {
            name: name.into(),

            parser: Parser::new(policy),
        }
    }
}

impl<P> Cote<P>
where
    P: Policy<Error = Error>,
{
    pub fn new_with<S: Into<String>>(
        name: S,
        policy: P,
        optset: P::Set,
        services: Services,
    ) -> Self {
        Self {
            name: name.into(),

            parser: Parser::new_with(policy, optset, services),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = name.into();
        self
    }

    // many apis can access through Deref
    // pub fn policy(&self) -> &P {
    //     &self.policy
    // }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::Cote;
    /// use cote::Error;
    /// use aopt::prelude::*;
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_mut(["-a", "-b", "42"].into_iter(), move |ret, cote| {
    ///         if ret.is_some() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // app still avilable here, SingleApp::run_async_mut pass mutable reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub fn run_mut<'a, 'b, I, R, F>(
        &'a mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'a: 'b,
        I: Into<RawVal>,
        F: FnMut(Option<()>, &'b mut Cote<P>) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let ret = parser.parse(aopt::Arc::new(Args::from(args)))?;

        r(ret, self)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use cote::Cote;
    /// use cote::Error;
    /// use aopt::prelude::*;
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_async_mut(["-a", "-b", "42"].into_iter(), |ret, cote| async move {
    ///         if ret.is_some() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // cote still avilable here, Cote::run_async_mut pass mutable reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run_async_mut<'a, 'b, I, R, FUT, F>(
        &'a mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'a: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(Option<()>, &'b mut Cote<P>) -> FUT,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        match parser.parse(aopt::Arc::new(Args::from(args))) {
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

    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::Cote;
    /// use cote::Error;
    /// use aopt::prelude::*;
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run(["-a", "-b", "42"].into_iter(), move |ret, cote| {
    ///         if ret.is_some() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // cote still avilable here, Cote::run pass reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub fn run<'a, 'b, I, R, F>(
        &'a mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'a: 'b,
        I: Into<RawVal>,
        F: FnMut(Option<()>, &'b Cote<P>) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let ret = parser.parse(aopt::Arc::new(Args::from(args)))?;

        r(ret, self)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use cote::Cote;
    /// use cote::Error;
    /// use aopt::prelude::*;
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_async(["-a", "-b", "42"].into_iter(), |ret, cote| async move {
    ///         if ret.is_some() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // cote still avilable here, Cote::run_async pass reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run_async<'a, 'b, I, R, FUT, F>(
        &'a mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'a: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(Option<()>, &'b Cote<P>) -> FUT,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        match parser.parse(aopt::Arc::new(Args::from(args))) {
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
