use std::fmt::Debug;

use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Creator;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::set::Commit;
use crate::set::Filter;
use crate::set::FilterMatcher;
use crate::set::FilterMut;
use crate::set::Pre;
use crate::set::Set;
use crate::set::SetIndex;
use crate::Error;
use crate::Str;
use crate::Uid;

/// Simple [`Set`] implementation hold [`Opt`] and [`Creator`].
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Result;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<()> {
///  let mut set = OptSet::<StrParser, Box<dyn Creator<Opt = AOpt, Config = OptConfig, Error = Error>>>::default();
///
///  // add prefix for option
///  set.add_prefix("/");
///  // add bool creator
///  set.register(BoolCreator::boxed());
///  // create a bool option
///  set.add_opt("/foo=b")?.run()?;
///  // filter the set option
///  assert_eq!(set.filter("foo")?.find_all().count(), 1);
///
///  Ok(())
/// # }
/// ```
pub struct OptSet<Parser, Ctor>
where
    Ctor: Creator,
    Ctor::Config: Config,
    Parser: OptParser,
{
    parser: Parser,
    opts: Vec<Ctor::Opt>,
    keys: Vec<Uid>,
    creators: Vec<Ctor>,
}

impl<Parser, Ctor> OptSet<Parser, Ctor>
where
    Ctor: Creator,
    Ctor::Config: Config,
    Parser: OptParser,
{
    pub fn new(parser: Parser) -> Self {
        Self {
            parser,
            opts: vec![],
            keys: vec![],
            creators: vec![],
        }
    }
}

impl<Parser, Ctor> Debug for OptSet<Parser, Ctor>
where
    Ctor::Opt: Debug,
    Ctor: Creator + Debug,
    Ctor::Config: Config + Debug,
    Parser: OptParser + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptSet")
            .field("parser", &self.parser)
            .field("opts", &self.opts)
            .field("keys", &self.keys)
            .field("creators", &self.creators)
            .finish()
    }
}

impl<Parser, Ctor> Default for OptSet<Parser, Ctor>
where
    Ctor: Creator,
    Ctor::Config: Config,
    Parser: OptParser + Default,
{
    fn default() -> Self {
        Self {
            parser: Parser::default(),
            opts: vec![],
            keys: vec![],
            creators: vec![],
        }
    }
}

impl<Parser, Ctor> OptSet<Parser, Ctor>
where
    Ctor: Creator,
    Parser: OptParser,
    Ctor::Config: Config,
{
    pub fn with_creator(mut self, creator: Ctor) -> Self {
        self.register(creator);
        self
    }

    pub fn register(&mut self, creator: Ctor) -> &mut Self {
        self.creators.push(creator);
        self
    }

    pub fn creator<S: Into<Str>>(&mut self, type_name: S) -> Option<&mut Ctor> {
        let type_name = type_name.into();

        self.creators.iter_mut().find(|v| v.r#type() == type_name)
    }
}

impl<Parser, Ctor> OptSet<Parser, Ctor>
where
    Ctor: Creator,
    Parser: OptParser,
    Ctor::Config: Config,
{
    pub fn parser(&self) -> &Parser {
        &self.parser
    }

    pub fn parser_mut(&mut self) -> &mut Parser {
        &mut self.parser
    }
}

impl<Parser, Ctor> OptSet<Parser, Ctor>
where
    Ctor: Creator,
    Ctor::Config: Config,
    Parser: OptParser + Pre,
{
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.add_prefix(prefix);
        self
    }

    pub fn contain_type<S: Into<Str>>(&self, type_name: S) -> bool {
        let type_name = type_name.into();

        self.creators.iter().any(|v| v.r#type() == type_name)
    }
}

impl<Parser, Ctor> OptSet<Parser, Ctor>
where
    Ctor::Opt: Opt,
    Ctor: Creator,
    Parser: OptParser + Pre,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    pub fn iter(&self) -> std::slice::Iter<'_, Ctor::Opt> {
        self.opts.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Ctor::Opt> {
        self.opts.iter_mut()
    }

    /// Add an option into current [`OptSet`].
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`Commit`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn add_opt<S: Into<Str>>(&mut self, opt_str: S) -> Result<Commit<'_, Parser, Ctor>, Error> {
        Ok(Commit::new(
            self,
            <Ctor::Config as Config>::new(self.parser(), opt_str.into())?,
        ))
    }

    /// Filter the option by configuration.
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`Filter`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn filter<S: Into<Str>>(&self, opt_str: S) -> Result<Filter<'_, Parser, Ctor>, Error> {
        Ok(Filter::new(
            self,
            <Ctor::Config as Config>::new(self.parser(), opt_str.into())?,
        ))
    }

    /// Filter the option, return the reference of first matched [`Opt`].
    pub fn find<S: Into<Str>>(&self, opt_str: S) -> Result<Option<&Ctor::Opt>, Error> {
        let info = <Ctor::Config as Config>::new(self.parser(), opt_str.into())?;
        Ok(self.iter().find(|opt| info.mat_opt(*opt)))
    }

    /// Filter the option, return an iterator of reference of [`Opt`]s.
    pub fn find_all<S: Into<Str>>(
        &self,
        opt_str: S,
    ) -> Result<impl Iterator<Item = &Ctor::Opt>, Error> {
        let info = <Ctor::Config as Config>::new(self.parser(), opt_str.into())?;
        Ok(self.iter().filter(move |opt| info.mat_opt(*opt)))
    }

    /// Filter the option by configuration.
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`FilterMut`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn filter_mut<S: Into<Str>>(
        &mut self,
        opt_str: S,
    ) -> Result<FilterMut<'_, Parser, Ctor>, Error> {
        Ok(FilterMut::new(
            self,
            <Ctor::Config as Config>::new(self.parser(), opt_str.into())?,
        ))
    }

    /// Filter the option, return the mutable reference of first matched [`Opt`].
    pub fn find_mut<S: Into<Str>>(&mut self, opt_str: S) -> Result<Option<&mut Ctor::Opt>, Error> {
        let info = <Ctor::Config as Config>::new(self.parser(), opt_str.into())?;
        Ok(self.iter_mut().find(|opt| info.mat_opt(*opt)))
    }

    /// Filter the option, return an iterator of mutable reference of [`Opt`]s.
    pub fn find_all_mut<S: Into<Str>>(
        &mut self,
        opt_str: S,
    ) -> Result<impl Iterator<Item = &mut Ctor::Opt>, Error> {
        let info = <Ctor::Config as Config>::new(self.parser(), opt_str.into())?;
        Ok(self.iter_mut().filter(move |opt| info.mat_opt(*opt)))
    }
}

impl<Parser, Ctor, I: SetIndex<OptSet<Parser, Ctor>>> std::ops::Index<I> for OptSet<Parser, Ctor>
where
    Ctor::Opt: Opt,
    Ctor: Creator,
    Parser: OptParser,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    type Output = Ctor::Opt;

    fn index(&self, index: I) -> &Self::Output {
        index.ref_from(self).unwrap()
    }
}

impl<Parser, Ctor, I: SetIndex<OptSet<Parser, Ctor>>> std::ops::IndexMut<I> for OptSet<Parser, Ctor>
where
    Ctor::Opt: Opt,
    Ctor: Creator,
    Parser: OptParser,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.mut_from(self).unwrap()
    }
}

impl<'b, Parser, Ctor> SetIndex<OptSet<Parser, Ctor>> for &'b str
where
    Ctor::Opt: Opt,
    Ctor: Creator,
    Parser: OptParser + Pre,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    fn ref_from<'a>(&self, set: &'a OptSet<Parser, Ctor>) -> Result<&'a Ctor::Opt, Error> {
        set.find(*self)?
            .ok_or_else(|| Error::raise_error(format!("Can not find option {}", *self)))
    }

    fn mut_from<'a>(&self, set: &'a mut OptSet<Parser, Ctor>) -> Result<&'a mut Ctor::Opt, Error> {
        set.find_mut(*self)?
            .ok_or_else(|| Error::raise_error(format!("Can not find option {}", *self)))
    }
}

impl<Parser, Ctor> Set for OptSet<Parser, Ctor>
where
    Ctor::Opt: Opt,
    Ctor: Creator,
    Ctor::Config: Config,
    Parser: OptParser,
{
    type Opt = Ctor::Opt;

    fn len(&self) -> usize {
        self.opts.len()
    }

    fn reset(&mut self) {
        for opt in self.opts.iter_mut() {
            opt.reset();
        }
    }

    fn keys(&self) -> &[Uid] {
        &self.keys
    }

    fn insert(&mut self, mut opt: Self::Opt) -> Uid {
        let uid = self.len() as Uid;

        opt.set_uid(uid);
        self.opts.push(opt);
        self.keys.push(uid);
        uid
    }

    fn get(&self, id: Uid) -> Option<&Self::Opt> {
        self.opts.get(id as usize)
    }

    fn get_mut(&mut self, id: Uid) -> Option<&mut Self::Opt> {
        self.opts.get_mut(id as usize)
    }
}

impl<Parser, Ctor> OptParser for OptSet<Parser, Ctor>
where
    Ctor::Opt: Opt,
    Ctor: Creator,
    Parser: OptParser + Pre,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    type Output = Parser::Output;

    type Error = Parser::Error;

    fn parse(&self, pattern: Str) -> Result<Self::Output, Self::Error> {
        self.parser().parse(pattern)
    }
}

impl<Parser, Ctor> Pre for OptSet<Parser, Ctor>
where
    Ctor: Creator,
    Ctor::Config: Config,
    Parser: OptParser + Pre,
{
    fn add_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self {
        self.parser_mut().add_prefix(prefix);
        self
    }

    fn prefix(&self) -> &[Str] {
        self.parser().prefix()
    }
}

#[cfg(test)]
mod test {

    use crate::prelude::*;
    use crate::Error;

    #[test]
    fn test_add_option() {
        assert!(test_add_option_impl().is_ok());
    }

    fn test_add_option_impl() -> Result<(), Error> {
        let mut set = aset_with_default_creators();

        assert!(set.add_opt("cmda=c")?.run().is_ok());
        assert!(set.add_opt("cmdb=c")?.run().is_ok());

        assert!(set.add_opt("posa=p@2")?.run().is_ok());
        assert!(set.add_opt("posb=p@3")?.run().is_ok());
        assert!(set.add_opt("posc=p@4")?.run().is_ok());
        assert!(set.add_opt("posd=p!@4")?.run().is_ok());

        assert!(set.add_opt("main=m")?.run().is_ok());

        assert!(set.add_opt("--boola=b")?.run().is_ok());
        assert!(set.add_opt("--boolb=b")?.run().is_ok());
        assert!(set.add_opt("-boolc=b")?.run().is_ok());
        assert!(set.add_opt("-boold=b")?.run().is_ok());
        assert!(set.add_opt("--boole=b!")?.run().is_ok());
        assert!(set.add_opt("--boolf=b/")?.run().is_ok());
        assert!(set.add_opt("--boolg=b!/")?.run().is_ok());
        assert!(set.add_opt("-boolh=b!")?.run().is_ok());

        assert!(set.add_opt("--floatb=f")?.run().is_ok());
        assert!(set.add_opt("--floatc=f!")?.run().is_ok());
        assert!(set.add_opt("-floata=f")?.run().is_ok());
        assert!(set.add_opt("-floatd=f!")?.run().is_ok());
        assert!(set.add_opt("-floate=f")?.add_alias("-e").run().is_ok());

        assert!(set.add_opt("--inta=i")?.run().is_ok());
        assert!(set.add_opt("--intb=i!")?.run().is_ok());
        assert!(set.add_opt("-intc=i")?.run().is_ok());
        assert!(set.add_opt("-intd=i!")?.run().is_ok());

        assert!(set.find("cmda")?.is_some());
        assert_eq!(set.find_all("=c")?.count(), 2);

        assert!(set.find("posb")?.is_some());
        assert!(set.find_mut("posc")?.is_some());
        assert_eq!(set.find_all("=p")?.count(), 4);
        assert_eq!(set.find_all_mut("=p")?.count(), 4);
        assert_eq!(set.find_all("=p@4")?.count(), 2);
        assert!(set.filter("posd")?.set_opt(false).find().is_some());
        assert!(set.filter("=p")?.set_name("pose").find().is_none());

        assert!(set.find("main")?.is_some());

        assert!(set.find("--boola")?.is_some());
        assert!(set.find("--boolb")?.is_some());
        assert!(set.find_mut("--boole=b!")?.is_some());
        assert!(set.find_mut("--boolf=b/")?.is_some());
        assert_eq!(set.find_all("=b")?.count(), 8);
        assert_eq!(set.find_all("=b!")?.count(), 3);
        assert_eq!(set.filter("=b")?.set_opt(false).find_all().count(), 3);
        assert!(set.filter("--boolg=b!")?.find().is_some());
        assert!(set.filter("-boolg=b!")?.find().is_none());

        assert!(set.find("=f")?.is_some());
        assert!(set.find("--floatc=f!")?.is_some());
        assert!(set.find("-e")?.is_some());
        assert!(set.find_mut("-floata=f")?.is_some());
        assert!(set.find_mut("-floatd=f!")?.is_some());
        assert_eq!(set.find_all("=f")?.count(), 5);
        assert_eq!(set.find_all("=f!")?.count(), 2);
        assert_eq!(set.filter_mut("=f")?.set_deact(true).find_all().count(), 0);

        assert!(set.find("--=i")?.is_some());
        assert!(set.find("--intb")?.is_some());
        assert!(set.find_mut("-inta=i")?.is_none());
        assert!(set.find_mut("--intb=i!")?.is_some());
        assert_eq!(set.find_all_mut("=i")?.count(), 4);
        assert_eq!(set.find_all_mut("=i!")?.count(), 2);
        assert_eq!(set.filter_mut("=i")?.set_opt(true).find_all().count(), 2);
        assert_eq!(set.filter_mut("=i")?.set_opt(false).find_all().count(), 2);

        set.add_prefix("+");
        set.add_prefix("/");

        assert!(set.add_opt("--stra=s")?.add_alias("/stre").run().is_ok());
        assert!(set.add_opt("--strb=s!")?.add_alias("/strf").run().is_ok());
        assert!(set.add_opt("-strc=s")?.run().is_ok());
        assert!(set.add_opt("-strd=s!")?.run().is_ok());
        assert!(set.add_opt("+strg=s")?.run().is_ok());
        assert!(set.add_opt("+strh=s!")?.run().is_ok());

        assert!(set.add_opt("--uinta=u")?.run().is_ok());
        assert!(set.add_opt("--uintb=u!")?.run().is_ok());
        assert!(set.add_opt("-uintc=u")?.add_alias("+uintg").run().is_ok());
        assert!(set.add_opt("-uintd=u!")?.add_alias("+uinth").run().is_ok());
        assert!(set.add_opt("/uinte=u")?.run().is_ok());
        assert!(set.add_opt("/uintf=u!")?.run().is_ok());

        assert!(set.find("--=s")?.is_some());
        assert!(set.find("--=s!")?.is_some());
        assert!(set.find("/stre")?.is_some());
        assert!(set.find("/strf")?.is_some());
        assert!(set.find_mut("+strg=s")?.is_some());
        assert!(set.find_mut("+strh=s!")?.is_some());
        assert_eq!(set.find_all_mut("/=s")?.count(), 2);
        assert_eq!(set.find_all_mut("+=s!")?.count(), 1);

        assert!(set.find("-uintc")?.is_some());
        assert!(set.find("--uintb")?.is_some());
        assert!(set.find("+uintg")?.is_some());
        assert!(set.find("+uinth")?.is_some());
        assert_eq!(set.find_all("--=u")?.count(), 2);
        assert_eq!(set.find_all("--=u!")?.count(), 1);

        assert_eq!(set.filter("")?.set_pre("+").find_all().count(), 4);
        assert_eq!(set.filter("")?.set_pre("/").find_all().count(), 4);

        assert!(set
            .add_opt("")?
            .set_name("foo")
            .set_prefix("/")
            .set_optional(false)
            .set_type("b")
            .set_deactivate(true)
            .run()
            .is_ok());
        assert!(set.find("/foo")?.is_some());

        Ok(())
    }
}
