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

///
/// A convenient struct using for create application.
///
/// # Example
///
///```ignore
/// use aopt::app::SingleApp;
/// use aopt::err::Result;
/// use aopt::prelude::*;
///
/// #[async_std::main]
/// async fn main() -> Result<()> {
///     let mut app = SingleApp::<SimpleSet, DefaultService, ForwardPolicy>::default();
///
///     app.add_opt("-a=b!")?.commit()?;
///     app.add_opt("-b=i")?.commit()?;
///
///     app.run_async_mut(
///         ["-a", "-b", "42"].into_iter(),
///         |ret, app| async move {
///             if ret {
///                 dbg!(&app);
///                 dbg!(app.find("-a")?);
///                 dbg!(app.find("-b")?);
///             }
///             Ok(())
///         },
///     )
///     .await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct SingleApp<S, SS, P>
where
    S: Set,
    SS: Service<S>,
    P: Policy<S, SS>,
{
    name: String,
    parser: Parser<S, SS, P>,
}

impl<S, SS, P> Default for SingleApp<S, SS, P>
where
    S: Set + Default,
    SS: Service<S> + Default,
    P: Policy<S, SS> + Default,
{
    /// The default name of app is `singleapp`.
    /// And the default parser is `Parser::default()`.
    fn default() -> Self {
        Self {
            name: "singleapp".into(),
            parser: Parser::default(),
        }
    }
}

impl<S, SS, P> SingleApp<S, SS, P>
where
    S: Set + Default,
    SS: Service<S> + Default,
    P: Policy<S, SS>,
{
    /// The default name of app is `singleapp`.
    /// And the parser is `Parser::new_policy(policy)`.
    pub fn new_policy(policy: P) -> Self {
        Self {
            name: "singleapp".into(),
            parser: Parser::new_policy(policy),
        }
    }
}

impl<S, SS, P> SingleApp<S, SS, P>
where
    S: Set,
    SS: Service<S>,
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

    /// Insert callback to hash map.
    pub fn add_callback(&mut self, uid: Uid, callback: OptCallback<S>) {
        self.parser.add_callback(uid, callback);
    }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aopt::app::SingleApp;
    /// use aopt::err::Result;
    /// use aopt::prelude::*;
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<()> {
    ///     let mut app = SingleApp::<SimpleSet, DefaultService, ForwardPolicy>::default();
    ///
    ///     app.add_opt("-a=b!")?.commit()?;
    ///     app.add_opt("-b=i")?.commit()?;
    ///
    ///     app.run_mut(["-a", "-b", "42"].into_iter(), move |ret, app| {
    ///         if ret {
    ///             assert_eq!(app.get_value("-a")?, Some(&OptValue::from(true)));
    ///             assert_eq!(app.get_value("-b")?, Some(&OptValue::from(42i32)));
    ///             println!("{} running over!", app.get_name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // app still avilable here, SingleApp::run_async_mut pass mutable reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub fn run_mut<'a, 'b, I, ITER, R, F>(&'a mut self, iter: ITER, mut r: F) -> Result<R>
    where
        'a: 'b,
        I: Into<String>,
        ITER: Iterator<Item = I>,
        F: FnMut(bool, &'b mut SingleApp<S, SS, P>) -> Result<R>,
    {
        let args: Vec<String> = iter.map(|v| v.into()).collect();
        let parser = &mut self.parser;
        let ret = parser.parse(&mut ArgStream::from(args.into_iter()))?;

        r(ret, self)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aopt::app::SingleApp;
    /// use aopt::err::Result;
    /// use aopt::prelude::*;
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<()> {
    ///     let mut app = SingleApp::<SimpleSet, DefaultService, ForwardPolicy>::default();
    ///
    ///     app.add_opt("-a=b!")?.commit()?;
    ///     app.add_opt("-b=i")?.commit()?;
    ///
    ///     app.run_async_mut(["-a", "-b", "42"].into_iter(), |ret, app| async move {
    ///         if ret {
    ///             assert_eq!(app.get_value("-a")?, Some(&OptValue::from(true)));
    ///             assert_eq!(app.get_value("-b")?, Some(&OptValue::from(42i32)));
    ///             println!("{} running over!", app.get_name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // app still avilable here, SingleApp::run_async_mut pass mutable reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run_async_mut<'a, 'b, I, ITER, R, FUT, F>(
        &'a mut self,
        iter: ITER,
        mut r: F,
    ) -> Result<R>
    where
        'a: 'b,
        I: Into<String>,
        ITER: Iterator<Item = I>,
        FUT: Future<Output = Result<R>>,
        F: FnMut(bool, &'b mut SingleApp<S, SS, P>) -> FUT,
    {
        let args: Vec<String> = iter.map(|v| v.into()).collect();
        let parser = &mut self.parser;
        let async_ret;

        match parser.parse(&mut ArgStream::from(args.into_iter())) {
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
    SS: Service<S> + Default,
    P: Policy<S, SS> + Default,
{
    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aopt::app::SingleApp;
    /// use aopt::err::Result;
    /// use aopt::prelude::*;
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<()> {
    ///     let mut app = SingleApp::<SimpleSet, DefaultService, ForwardPolicy>::default();
    ///
    ///     app.add_opt("-a=b!")?.commit()?;
    ///     app.add_opt("-b=i")?.commit()?;
    ///
    ///     app.run(["-a", "-b", "42"].into_iter(), move |ret, app| {
    ///         if ret {
    ///             assert_eq!(app.get_value("-a")?, Some(&OptValue::from(true)));
    ///             assert_eq!(app.get_value("-b")?, Some(&OptValue::from(42i32)));
    ///             println!("{} running over!", app.get_name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // app not avilable here, SingleApp::run take the ownership of app
    ///
    ///     Ok(())
    /// }
    ///```
    pub fn run<I, ITER, R, F>(&mut self, iter: ITER, mut r: F) -> Result<R>
    where
        I: Into<String>,
        ITER: Iterator<Item = I>,
        F: FnMut(bool, SingleApp<S, SS, P>) -> Result<R>,
    {
        let args: Vec<String> = iter.map(|v| v.into()).collect();
        let parser = &mut self.parser;
        let ret = parser.parse(&mut ArgStream::new(args.into_iter()))?;

        r(ret, std::mem::take(self))
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aopt::app::SingleApp;
    /// use aopt::err::Result;
    /// use aopt::prelude::*;
    ///
    /// #[async_std::main]
    /// async fn main() -> Result<()> {
    ///     let mut app = SingleApp::<SimpleSet, DefaultService, ForwardPolicy>::default();
    ///
    ///     app.add_opt("-a=b!")?.commit()?;
    ///     app.add_opt("-b=i")?.commit()?;
    ///
    ///     app.run_async(["-a", "-b", "42"].into_iter(), |ret, app| async move {
    ///         if ret {
    ///             assert_eq!(app.get_value("-a")?, Some(&OptValue::from(true)));
    ///             assert_eq!(app.get_value("-b")?, Some(&OptValue::from(42i32)));
    ///             println!("{} running over!", app.get_name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // app not avilable here, SingleApp::run_async take the ownership of app
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run_async<I, ITER, R, FUT, F>(&mut self, iter: ITER, mut r: F) -> Result<R>
    where
        I: Into<String>,
        ITER: Iterator<Item = I>,
        FUT: Future<Output = Result<R>>,
        F: FnMut(bool, SingleApp<S, SS, P>) -> FUT,
    {
        let args: Vec<String> = iter.map(|v| v.into()).collect();
        let parser = &mut self.parser;
        let async_ret;

        match parser.parse(&mut ArgStream::from(args.into_iter())) {
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
    SS: Service<S>,
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
    SS: Service<S>,
    P: Policy<S, SS>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}
