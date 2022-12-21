#![doc = include_str!("../README.md")]
pub mod meta;

use std::{
    borrow::Cow,
    fmt::Debug,
    future::Future,
    ops::{Deref, DerefMut},
};

use aopt::{
    prelude::*,
    set::{SetCfg, SetOpt},
    RawVal,
};
use aopt_help::{prelude::Block, store::Store};

pub use aopt::Error;
use prelude::MetaConfig;

pub mod prelude {
    pub use crate::cote_help;
    pub use crate::meta::MetaConfig;
    pub use crate::Cote;
    pub use aopt;
    pub use aopt_help;
}

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
    P::Set: 'static,
    P: Policy<Error = Error>,
    SetOpt<P::Set>: Opt,
    P::Set: Pre + Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    /// Add the option from the [`MetaConfig`].
    ///
    ///```ignore
    /// # use cote::prelude::*;
    /// # use aopt::prelude::*;
    /// # use aopt::Error;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     let mut cote = Cote::<AFwdPolicy>::default();
    ///
    ///     cote.add_meta::<String>(
    ///         serde_json::from_str(
    ///             r#"
    ///     {
    ///         "option": "-c=s",
    ///         "hint": "-c <str>",
    ///         "help": "This is a help for option c",
    ///         "action": "App",
    ///         "assoc": "Str",
    ///         "alias": null,
    ///         "value": [
    ///           "we",
    ///           "it"
    ///         ]
    ///     }
    ///     "#,
    ///         )
    ///         .unwrap(),
    ///     )?;
    ///     cote.add_meta::<i64>(
    ///         serde_json::from_str(
    ///             r#"
    ///     {
    ///         "option": "--point=i",
    ///         "hint": "--point <int>",
    ///         "help": "This is a help for option",
    ///         "action": "App",
    ///         "assoc": "Int",
    ///         "alias": [
    ///             "-p"
    ///         ]
    ///       }
    ///     "#,
    ///         )
    ///         .unwrap(),
    ///     )?;
    ///
    ///     cote.run(["-p", "256"].into_iter(), |ret, cote: &Cote<AFwdPolicy>| {
    ///         if ret.is_some() {
    ///             assert_eq!(
    ///                 &vec!["we".to_owned(), "it".to_owned()],
    ///                 cote.find_vals::<String>("-c")?
    ///             );
    ///             assert_eq!(&256, cote.find_val::<i64>("--point")?);
    ///             println!("cote running okay!!!");
    ///         }
    ///         Ok(())
    ///     })?;
    ///     # Ok(())
    /// # }
    /// ```
    pub fn add_meta<T: Clone + 'static>(
        &mut self,
        mut meta: MetaConfig<T>,
    ) -> Result<ParserCommit<'_, P::Set>, Error> {
        let mut pc = self.add_opt(meta.take_option())?;

        if let Some(hint) = meta.take_hint() {
            pc = pc.set_hint(hint);
        }
        if let Some(help) = meta.take_help() {
            pc = pc.set_help(help);
        }
        if let Some(action) = meta.take_action() {
            pc = pc.set_action(action);
        }
        if let Some(assoc) = meta.take_assoc() {
            pc = pc.set_assoc(assoc);
        }
        if let Some(value) = meta.take_value() {
            pc = pc.set_initiator(ValInitiator::with(value));
        }
        if let Some(alias_) = meta.take_alias() {
            for alias in alias_ {
                pc = pc.add_alias(alias);
            }
        }
        Ok(pc)
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

    pub fn display_help<'a, S: Into<Cow<'a, str>>>(&self, head: S, foot: S) -> Result<(), Error> {
        self.__display_help(head, foot)
            .map_err(|e| Error::raise_error(format!("Can not show help message: {:?}", e)))
    }

    fn __display_help<'a, S: Into<Cow<'a, str>>>(
        &self,
        head: S,
        foot: S,
    ) -> Result<(), aopt_help::Error> {
        let head = head.into();
        let foot = foot.into();
        let mut app_help = aopt_help::AppHelp::new(
            self.name.as_str(),
            &head,
            &foot,
            aopt_help::prelude::Style::default(),
            std::io::stdout(),
        );
        let global = app_help.global_mut();
        let set = self.parser.optset();

        global.add_block(Block::new("command", "<COMMAND>", "", "COMMAND:", ""))?;
        global.add_block(Block::new("option", "", "", "OPTION:", ""))?;
        global.add_block(Block::new("args", "[ARGS]", "", "ARGS:", ""))?;
        for opt in set.iter() {
            if opt.mat_style(Style::Pos) {
                global.add_store(
                    "args",
                    Store::new(
                        Cow::from(opt.name().as_str()),
                        Cow::from(opt.hint().as_str()),
                        Cow::from(opt.help().as_str()),
                        Cow::from(opt.r#type().to_string()),
                        opt.optional(),
                        true,
                    ),
                )?;
            } else if opt.mat_style(Style::Cmd) {
                global.add_store(
                    "command",
                    Store::new(
                        Cow::from(opt.name().as_str()),
                        Cow::from(opt.hint().as_str()),
                        Cow::from(opt.help().as_str()),
                        Cow::from(opt.r#type().to_string()),
                        opt.optional(),
                        true,
                    ),
                )?;
            } else if opt.mat_style(Style::Argument)
                || opt.mat_style(Style::Boolean)
                || opt.mat_style(Style::Combined)
            {
                global.add_store(
                    "option",
                    Store::new(
                        Cow::from(opt.name().as_str()),
                        Cow::from(opt.hint().as_str()),
                        Cow::from(opt.help().as_str()),
                        Cow::from(opt.r#type().to_string()),
                        opt.optional(),
                        false,
                    ),
                )?;
            }
        }

        app_help.display(true)?;

        Ok(())
    }
}

/// Display help message of [`Cote`] generate from `Cargo.toml`.
/// The `head` will be generate from package's description.
/// The `foot` will be generate from package's authors and version.
#[macro_export]
macro_rules! cote_help {
    ($cote:ident) => {{
        let foot = format!(
            "Create by {} v{}",
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_VERSION")
        );
        let head = format!("{}", env!("CARGO_PKG_DESCRIPTION"));

        $cote.display_help(head, foot)
    }};
}