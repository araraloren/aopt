#![doc = include_str!("../README.md")]
pub mod meta;
pub mod valid;
pub mod _toturial;

use std::borrow::Cow;
use std::fmt::Debug;
use std::future::Future;
use std::ops::Deref;
use std::ops::DerefMut;

use aopt::prelude::*;
use aopt::value::Placeholder;
use aopt::Error;
use aopt::RawVal;
use aopt_help::prelude::Block;
use aopt_help::prelude::Store;

use meta::IntoConfig;

pub use aopt;
pub use aopt_help;
pub use cote_derive;

pub mod prelude {
    pub use crate::aopt;
    pub use crate::aopt::prelude::*;
    pub use crate::cote_derive::Cote;
    pub use crate::display_help;
    pub use crate::display_set_help;
    pub use crate::meta::IntoConfig;
    pub use crate::meta::OptionMeta;
    pub use crate::simple_display_set_help;
    pub use crate::valid;
    pub use crate::CoteApp;
    pub use crate::ExtractFromSetDerive;
    pub use crate::IntoParserDerive;
}

pub trait IntoParserDerive<'zlifetime, P>
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

pub trait ExtractFromSetDerive<'zlifetime, S>
where
    S: SetValueFindExt,
{
    fn try_extract(set: &'zlifetime mut S) -> Result<Self, aopt::Error>
    where
        Self: Sized;
}

pub struct CoteApp<'a, P>
where
    P: Policy,
{
    name: String,

    parser: Parser<'a, P>,
}

impl<'a, P> Debug for CoteApp<'a, P>
where
    P::Ret: Debug,
    P::Set: Debug,
    P::Ser: Debug,
    P: Policy + Debug,
    P::Inv<'a>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoteApp")
            .field("name", &self.name)
            .field("parser", &self.parser)
            .finish()
    }
}

impl<'a, P> Default for CoteApp<'a, P>
where
    P::Set: Default,
    P::Ser: Default,
    P::Inv<'a>: Default,
    P: Policy + APolicyExt<P> + Default,
{
    fn default() -> Self {
        Self {
            name: "CoteApp".to_owned(),
            parser: Parser::default(),
        }
    }
}

impl<'a, P: Policy> Deref for CoteApp<'a, P> {
    type Target = Parser<'a, P>;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl<'a, P: Policy> DerefMut for CoteApp<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P: Policy + APolicyExt<P>,
{
    pub fn new<S: Into<String>>(name: S, policy: P) -> Self {
        Self {
            name: name.into(),

            parser: Parser::new(policy),
        }
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P: Policy,
{
    pub fn inner_parser(&mut self) -> &mut Parser<'a, P> {
        &mut self.parser
    }

    pub fn inner_parser_mut(&mut self) -> &mut Parser<'a, P> {
        &mut self.parser
    }

    pub fn set_parser(&mut self, parser: Parser<'a, P>) -> &mut Self {
        self.parser = parser;
        self
    }
}

impl<'a, 'b, P> CoteApp<'a, P>
where
    'a: 'b,
    P: Policy,
    P::Set: SetValueFindExt,
{
    pub fn extract_type<T>(&'b mut self) -> Result<T, Error>
    where
        T: ExtractFromSetDerive<'b, P::Set>,
    {
        let set = self.parser.optset_mut();

        T::try_extract(set)
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P::Ser: 'a,
    P: Policy,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser + 'a,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    P::Inv<'a>: HandlerCollection<'a, P::Set, P::Ser>,
{
    /// Add option by option configuration generated from [`OptionMeta`](crate::meta::OptionMeta).
    ///
    /// # Example load option from json configuration.
    /// ```rust
    /// # use cote::prelude::*;
    /// # use aopt::prelude::*;
    /// # use aopt::Error;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     let mut cote = CoteApp::<AFwdPolicy>::default();
    ///
    ///     let config: OptionMeta<String> = serde_json::from_str(
    ///         r#"
    ///                      {
    ///                          "id": "-c",
    ///                          "option": "-c=s",
    ///                          "hint": "-c <str>",
    ///                          "help": "This is a help for option c",
    ///                          "action": "App",
    ///                          "alias": null,
    ///                          "value": [
    ///                          "we",
    ///                          "it"
    ///                          ]
    ///                      }
    ///                  "#,
    ///     )
    ///     .unwrap();
    ///
    ///     cote.add_opt_meta(config)?;
    ///
    ///     let config: OptionMeta<i64> = serde_json::from_str(
    ///         r#"
    ///                          {
    ///                              "id": "-p",
    ///                              "option": "--point=i",
    ///                              "hint": "--point <int>",
    ///                              "help": "This is a help for option",
    ///                              "action": "App",
    ///                              "alias": [
    ///                                  "-p"
    ///                              ]
    ///                          }
    ///                      "#,
    ///     )
    ///     .unwrap();
    ///
    ///     cote.add_opt_meta(config)?;
    ///
    ///     cote.run_with(
    ///         ["-p", "256"].into_iter(),
    ///         |ret, cote: &CoteApp<AFwdPolicy>| {
    ///             if ret.status() {
    ///                 assert_eq!(
    ///                     &vec!["we".to_owned(), "it".to_owned()],
    ///                     cote.find_vals::<String>("-c")?
    ///                 );
    ///                 assert_eq!(&256, cote.find_val::<i64>("--point")?);
    ///                 println!("cote running okay!!!");
    ///             }
    ///             Ok(())
    ///         },
    ///     )?;
    ///
    /// #    Ok(())
    /// # }
    ///```
    ///
    pub fn add_opt_meta(
        &mut self,
        mut meta: impl IntoConfig<Ret = SetCfg<P::Set>>,
    ) -> Result<ParserCommit<'a, '_, P::Inv<'a>, P::Set, P::Ser, Placeholder>, Error> {
        let set = self.parser.optset();
        let config = meta.into_config(set)?;

        self.parser.add_opt_cfg(config)
    }

    /// This function will insert help option `--help;-h;-?: Display help message`.
    pub fn add_help_option(&mut self) -> Result<&mut Self, Error> {
        self.add_opt_i::<bool>("--help;-h;-?: Display help message")?;
        Ok(self)
    }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::CoteApp;
    /// use aopt::Error;
    /// use aopt::prelude::*;
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut cote = CoteApp::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_mut_with(["-a", "-b", "42"].into_iter(), move |ret, cote| {
    ///         if ret.status() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // cote still avilable here, CoteApp::run_mut_with pass mutable reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub fn run_mut_with<'c, 'b, I, R, F>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        F: FnMut(P::Ret, &'b mut CoteApp<'a, P>) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;

        // initialize the option value
        parser.init()?;

        let ret = parser
            .parse(aopt::ARef::new(Args::from(args)))
            .map_err(Into::into)?;

        r(ret, self)
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub fn run_mut<'c, 'b, R, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        F: FnMut(P::Ret, &'b mut CoteApp<'a, P>) -> Result<R, Error>,
    {
        let args = Args::from_env().into_inner();
        self.run_mut_with(args.into_iter(), r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::CoteApp;
    /// use aopt::Error;
    /// use aopt::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    ///     let mut cote = CoteApp::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_async_mut_with(["-a", "-b", "42"].into_iter(), |ret, cote| async move {
    ///         if ret.status() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // cote still avilable here, CoteApp::run_async_mut_with pass mutable reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run_async_mut_with<'c, 'b, I, R, FUT, F>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b mut CoteApp<'a, P>) -> FUT,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        // initialize the option value
        parser.init()?;
        match parser.parse(aopt::ARef::new(Args::from(args))) {
            Ok(ret) => {
                let ret = r(ret, self).await;

                async_ret = ret;
            }
            Err(e) => {
                async_ret = Err(e.into());
            }
        }
        async_ret
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub async fn run_async_mut<'c, 'b, R, FUT, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b mut CoteApp<'a, P>) -> FUT,
    {
        let args = Args::from_env().into_inner();
        self.run_async_mut_with(args.into_iter(), r).await
    }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::CoteApp;
    /// use aopt::Error;
    /// use aopt::prelude::*;
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut cote = CoteApp::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_with(["-a", "-b", "42"].into_iter(), move |ret, cote| {
    ///         if ret.status() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })?;
    ///
    ///     // cote still avilable here, CoteApp::run_with pass reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub fn run_with<'c, 'b, I, R, F>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        F: FnMut(P::Ret, &'b CoteApp<'a, P>) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;

        // initialize the option value
        parser.init()?;

        let ret = parser
            .parse(aopt::ARef::new(Args::from(args)))
            .map_err(Into::into)?;

        r(ret, self)
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub fn run<'c, 'b, R, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        F: FnMut(P::Ret, &'b CoteApp<'a, P>) -> Result<R, Error>,
    {
        let args = Args::from_env().into_inner();
        self.run_with(args.into_iter(), r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cote::CoteApp;
    /// use aopt::Error;
    /// use aopt::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    ///     let mut cote = CoteApp::<AFwdPolicy>::default();
    ///
    ///     cote.add_opt("-a=b!")?;
    ///     cote.add_opt("-b=i")?;
    ///
    ///     cote.run_async_with(["-a", "-b", "42"].into_iter(), |ret, cote| async move {
    ///         if ret.status() {
    ///             assert_eq!(cote.find_val::<bool>("-a")?, &true);
    ///             assert_eq!(cote.find_val::<i64>("-b")?, &42);
    ///             println!("{} running over!", cote.name());
    ///         }
    ///         Ok(())
    ///     })
    ///     .await?;
    ///
    ///     // cote still avilable here, CoteApp::run_async_with pass reference to closure.
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn run_async_with<'c, 'b, I, R, FUT, F>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b CoteApp<'a, P>) -> FUT,
    {
        let args = iter.map(|v| v.into());
        let parser = &mut self.parser;
        let async_ret;

        // initialize the option value
        parser.init()?;
        match parser.parse(aopt::ARef::new(Args::from(args))) {
            Ok(ret) => {
                let ret = r(ret, self).await;

                async_ret = ret;
            }
            Err(e) => {
                async_ret = Err(e.into());
            }
        }
        async_ret
    }

    /// Running with default arguments [`args()`](std::env::args).
    pub async fn run_async<'c, 'b, R, FUT, F>(&'c mut self, r: F) -> Result<R, Error>
    where
        'c: 'b,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b CoteApp<'a, P>) -> FUT,
    {
        let args = Args::from_env().into_inner();
        self.run_async_with(args.into_iter(), r).await
    }
}

impl<'a, P> CoteApp<'a, P>
where
    P: Policy,
    P::Set: SetValueFindExt,
{
    pub fn new_with<S: Into<String>>(
        name: S,
        policy: P,
        optset: P::Set,
        invoker: P::Inv<'a>,
        appser: P::Ser,
    ) -> Self {
        Self {
            name: name.into(),

            parser: Parser::new_with(policy, optset, invoker, appser),
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

    pub fn display_help(
        &self,
        author: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<bool, Error> {
        self.display_help_if("--help", author, version, description)
    }

    pub fn display_help_if(
        &self,
        option: &str,
        author: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<bool, Error> {
        self.display_help_if_width(option, author, version, description, 40, 10)
    }

    pub fn display_help_if_width(
        &self,
        option: &str,
        author: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        option_width: usize,
        usage_width: usize,
    ) -> Result<bool, Error> {
        let set = self.parser.optset();

        if let Ok(help_option) = set.find_val::<bool>(option) {
            if *help_option {
                let (author, version, description) =
                    (author.into(), version.into(), description.into());
                let name = self.name.to_string();

                crate::display_set_help!(
                    &name,
                    set,
                    author,
                    version,
                    description,
                    option_width,
                    usage_width
                )?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub fn simple_display_set_help<'a, T: Set, S: Into<Cow<'a, str>>>(
    set: &T,
    name: S,
    head: S,
    foot: S,
    max_width: usize,
    usage_width: usize,
) -> Result<(), aopt_help::Error> {
    let mut app_help = aopt_help::AppHelp::new(
        name,
        head,
        foot,
        aopt_help::prelude::Style::default(),
        std::io::stdout(),
        max_width,
        usage_width,
    );
    let global = app_help.global_mut();

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
                    Cow::default(),
                    !opt.force(),
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
                    Cow::default(),
                    !opt.force(),
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
                    Cow::default(),
                    !opt.force(),
                    false,
                ),
            )?;
        }
    }

    app_help.display(true)?;

    Ok(())
}

/// Display help message of [`CoteApp`] generate from `Cargo.toml`.
/// The `head` will be generate from package's description.
/// The `foot` will be generate from package's authors and version.
/// Default option text width is 20, and default usage width is 10.
#[macro_export]
macro_rules! display_help {
    ($cote:ident) => {{
        $cote.display_help(
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_DESCRIPTION"),
        )
    }};
}

/// Display help message of [`CoteApp`] generate from `Cargo.toml`.
/// The `head` will be generate from package's description.
/// The `foot` will be generate from package's authors and version.
#[macro_export]
macro_rules! display_set_help {
    ($name:expr, $set:ident, $author:expr, $version:expr, $description:expr, $width:expr, $usage_width:expr) => {{
        let foot = format!("Create by {} v{}", $author, $version,);
        let head = format!("{}", $description);

        fn __check_set<S: aopt::prelude::Set>(a: &S) -> &S {
            a
        }

        fn __check_name<T: Into<String>>(a: T) -> String {
            a.into()
        }

        $crate::simple_display_set_help(
            __check_set($set),
            __check_name($name),
            head,
            foot,
            $width,
            $usage_width,
        )
        .map_err(|e| aopt::Error::raise_error(format!("Can not show help message: {:?}", e)))
    }};
}

#[cfg(test)]
mod test {
    #[test]
    fn test_example_simple() {
        use crate::prelude::*;
        // macro generate the code depend on crate name
        use crate as cote;
        use aopt::opt::Pos;

        #[derive(Debug, Cote)]
        pub struct Example {
            /// a flag argument
            foo: bool,

            /// a position argument
            #[arg(index = "1")]
            bar: Pos<usize>,
        }

        let example = Example::parse(Args::from_array(["app", "--foo", "42"]));

        assert!(example.is_ok());

        let example = example.unwrap();

        assert_eq!(example.foo, true);
        assert_eq!(example.bar.0, 42);

        let parser = Example::into_parser().unwrap();

        assert_eq!(parser["--foo"].help(), &aopt::astr("a flag argument"));
        assert_eq!(parser["bar"].help(), &aopt::astr("a position argument"));
    }

    #[test]
    fn test_multiple_pos_arguments() {
        use crate::prelude::*;
        // macro generate the code depend on crate name
        use crate as cote;
        use aopt::opt::Pos;
        use std::path::PathBuf;

        #[derive(Debug, Cote)]
        #[cote(help)]
        pub struct CopyTool {
            #[arg(alias = "-f")]
            force: bool,

            /// Enable the recursive mode
            #[arg(alias = "-r")]
            recursive: bool,

            #[arg(index = "1", help = "The copy destination")]
            destination: Pos<String>,

            /// Specify path to copy
            #[arg(index = "2..")]
            sources: Pos<Vec<PathBuf>>,
        }

        let example = CopyTool::parse(Args::from_array([
            "app", "--force",
        ]));

        assert!(example.is_err());

        let example = CopyTool::parse(Args::from_array([
            "app", "--force", ".", "../foo", "../bar/", "other",
        ]))
        .unwrap();

        assert_eq!(example.force, true);
        assert_eq!(example.recursive, false);
        assert_eq!(example.destination.0, String::from("."));
        assert_eq!(
            example.sources.0,
            ["../foo", "../bar/", "other"]
                .into_iter()
                .map(|v| PathBuf::from(v))
                .collect::<Vec<PathBuf>>()
        );
    }

    #[test]
    fn test_fallback() {
        use crate::prelude::*;
        // macro generate the code depend on crate name
        use crate as cote;
        use aopt::opt::Pos;
        use aopt::GetoptRes;

        #[derive(Debug, Cote)]
        #[cote(policy = delay, help, on = find_main::<P>, name = "find")]
        pub struct Find {
            /// Do not follow symbolic link
            #[arg(name = "-H", nodelay)]
            hard: bool,

            /// Fllow symbolic link
            #[arg(name = "-L", nodelay)]
            symbol: bool,

            #[arg(name = "-P", nodelay, value = true)]
            never: bool,

            #[arg(name = "-name", help = "Search the file base on file name")]
            name: Option<String>,

            /// List the file large than the size
            #[arg(name = "-size")]
            size: Option<usize>,

            #[arg(index = "1", help = "Search starting point", fallback = search::<P>, then = VecStore)]
            destination: Pos<Vec<String>>,
        }

        fn search<P: Policy>(
            _: &mut P::Set,
            _: &mut P::Ser,
        ) -> Result<Option<Vec<String>>, aopt::Error> {
            Ok(Some(
                ["file1", "file2", "dir1", "dir2"]
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect(),
            ))
        }

        fn find_main<P>(set: &mut P::Set, _: &mut P::Ser) -> Result<Option<()>, aopt::Error>
        where
            P: Policy,
            P::Set: SetValueFindExt,
        {
            let tool = Find::try_extract(set)?;

            assert_eq!(tool.hard, true);
            assert_eq!(tool.symbol, false);
            assert_eq!(tool.never, true);
            assert_eq!(tool.name, Some("foo".to_owned()));
            assert_eq!(tool.size, Some(42));
            assert_eq!(
                tool.destination.0,
                ["file1", "file2", "dir1", "dir2"]
                    .into_iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
            );

            Ok(Some(()))
        }

        let args = Args::from_array([
            "app",
            ".",
            "-H",
            "-name=foo",
            "-size",
            "42",
        ]);

        let GetoptRes { ret, parser: _ } = Find::parse_args(args).unwrap();

        ret.ok().unwrap();
    }

    #[test]
    fn sub_test() {
        assert!(sub_test_impl().is_ok());
    }

    fn sub_test_impl() -> Result<(), aopt::Error> {
        use crate::prelude::*;
        // macro generate the code depend on crate name
        use crate as cote;
        use aopt::opt::Pos;
        use aopt::GetoptRes;
        use cote::valid;
        use std::path::PathBuf;

        #[derive(Debug, Cote, PartialEq, Default)]
        #[cote(policy = pre, help, name = "app")]
        pub struct App {
            /// Set the count value
            #[arg(values = [1usize, 2, 3])]
            count: Option<Vec<usize>>,

            #[sub(alias = "ls", help = "list subcommand list file of given path")]
            list: Option<List>,

            #[sub(help = "find something under directory")]
            find: Option<Find>,
        }

        #[derive(Debug, Cote, PartialEq)]
        #[cote(help)]
        pub struct List {
            #[arg(help = "list all the file")]
            all: bool,

            #[arg(valid = valid::array([1, 42, 68]))]
            depth: usize,

            #[arg(index = "1")]
            path: Pos<PathBuf>,
        }

        #[derive(Debug, Cote, PartialEq)]
        #[cote(help)]
        pub struct Find {
            recursive: bool,

            #[arg(index = "1")]
            path: Pos<PathBuf>,
        }

        let args = Args::from_array([
            "app",
            "ls",
            "--all",
            "--depth=42",
            ".",
        ]);

        let app = App::parse(args)?;

        assert_eq!(
            app,
            App {
                count: Some(vec![1, 2, 3]),
                list: Some(List {
                    all: true,
                    depth: 42,
                    path: Pos(PathBuf::from("."))
                }),
                find: None,
            }
        );

        let args = Args::from_array([
            "app",
            "list",
            "--all",
            "--depth=6",
            ".",
        ]);

        let app = App::parse(args)?;

        assert_eq!(
            app,
            App {
                count: Some(vec![1, 2, 3]),
                list: None,
                find: None,
            }
        );

        let args = Args::from_array([
            "app",
            "--count=8",
            "find",
            "something",
        ]);

        let app = App::parse(args)?;

        assert_eq!(
            app,
            App {
                count: Some(vec![1, 2, 3, 8]),
                list: None,
                find: Some(Find {
                    recursive: false,
                    path: Pos(PathBuf::from("something")),
                }),
            }
        );

        let args = Args::from_array(["app", "--count", "42"]);

        let app = App::parse(args)?;

        assert_eq!(
            app,
            App {
                count: Some(vec![1, 2, 3, 42]),
                list: None,
                find: None,
            }
        );

        let args = Args::from_array(["app", "--count=42", "list"]);

        let app = App::parse(args)?;

        assert_eq!(
            app,
            App {
                count: Some(vec![1, 2, 3, 42]),
                list: None,
                find: None,
            }
        );

        Ok(())
    }
}
