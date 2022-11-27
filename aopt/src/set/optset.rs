use std::fmt::Debug;

use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Ctor;
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

use super::SetOpt;

/// Simple [`Set`] implementation hold [`Opt`] and [`Ctor`].
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Result;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<()> {
///  let mut set = OptSet::<StrParser, Box<dyn Ctor<Opt = AOpt, Config = OptConfig, Error = Error>>>::default();
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
pub struct OptSet<P, C>
where
    C: Ctor,
    P: OptParser,
{
    parser: P,
    opts: Vec<C::Opt>,
    creators: Vec<C>,
}

impl<P, C> OptSet<P, C>
where
    C: Ctor,
    P: OptParser,
{
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            opts: vec![],
            creators: vec![],
        }
    }
}

impl<P, C> Debug for OptSet<P, C>
where
    C::Opt: Debug,
    C: Ctor + Debug,
    C::Config: Debug,
    P: OptParser + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptSet")
            .field("parser", &self.parser)
            .field("opts", &self.opts)
            .field("creators", &self.creators)
            .finish()
    }
}

impl<P, C> Default for OptSet<P, C>
where
    C: Ctor,
    P: OptParser + Default,
{
    fn default() -> Self {
        Self {
            parser: P::default(),
            opts: vec![],
            creators: vec![],
        }
    }
}

impl<P, C> OptSet<P, C>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    C::Config: Config,
{
    pub fn with_creator(mut self, creator: C) -> Self {
        self.register(creator);
        self
    }
}

impl<P, C> OptSet<P, C>
where
    C: Ctor,
    P: OptParser,
    C::Config: Config,
{
    pub fn parser(&self) -> &P {
        &self.parser
    }

    pub fn parser_mut(&mut self) -> &mut P {
        &mut self.parser
    }
}

impl<P, C> OptSet<P, C>
where
    C: Ctor,
    C::Config: Config,
    P: OptParser + Pre,
{
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.add_prefix(prefix);
        self
    }
}

impl<P, C> OptSet<P, C>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser + Pre,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    pub fn iter(&self) -> std::slice::Iter<'_, C::Opt> {
        self.opts.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, C::Opt> {
        self.opts.iter_mut()
    }

    /// Add an option into current [`OptSet`].
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`Commit`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn add_opt<S: Into<Str>>(&mut self, opt_str: S) -> Result<Commit<'_, Self>, Error> {
        Ok(Commit::new(
            self,
            <C::Config as Config>::new(self.parser(), opt_str.into())?,
        ))
    }

    /// Filter the option by configuration.
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`Filter`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn filter<S: Into<Str>>(&self, opt_str: S) -> Result<Filter<'_, Self>, Error> {
        Ok(Filter::new(
            self,
            <C::Config as Config>::new(self.parser(), opt_str.into())?,
        ))
    }

    /// Filter the option, return the reference of first matched [`Opt`].
    pub fn find<S: Into<Str>>(&self, opt_str: S) -> Result<Option<&C::Opt>, Error> {
        let info = <C::Config as Config>::new(self.parser(), opt_str.into())?;
        Ok(self.iter().find(|opt| info.mat_opt(*opt)))
    }

    /// Filter the option, return an iterator of reference of [`Opt`]s.
    pub fn find_all<S: Into<Str>>(
        &self,
        opt_str: S,
    ) -> Result<impl Iterator<Item = &C::Opt>, Error> {
        let info = <C::Config as Config>::new(self.parser(), opt_str.into())?;
        Ok(self.iter().filter(move |opt| info.mat_opt(*opt)))
    }

    /// Filter the option by configuration.
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`FilterMut`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn filter_mut<S: Into<Str>>(&mut self, opt_str: S) -> Result<FilterMut<'_, Self>, Error> {
        Ok(FilterMut::new(
            self,
            <C::Config as Config>::new(self.parser(), opt_str.into())?,
        ))
    }

    /// Filter the option, return the mutable reference of first matched [`Opt`].
    pub fn find_mut<S: Into<Str>>(&mut self, opt_str: S) -> Result<Option<&mut C::Opt>, Error> {
        let info = <C::Config as Config>::new(self.parser(), opt_str.into())?;
        Ok(self.iter_mut().find(|opt| info.mat_opt(*opt)))
    }

    /// Filter the option, return an iterator of mutable reference of [`Opt`]s.
    pub fn find_all_mut<S: Into<Str>>(
        &mut self,
        opt_str: S,
    ) -> Result<impl Iterator<Item = &mut C::Opt>, Error> {
        let info = <C::Config as Config>::new(self.parser(), opt_str.into())?;
        Ok(self.iter_mut().filter(move |opt| info.mat_opt(*opt)))
    }
}

impl<P, C, I: SetIndex<OptSet<P, C>>> std::ops::Index<I> for OptSet<P, C>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    type Output = C::Opt;

    fn index(&self, index: I) -> &Self::Output {
        index.ref_from(self).unwrap()
    }
}

impl<P, C, I: SetIndex<OptSet<P, C>>> std::ops::IndexMut<I> for OptSet<P, C>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.mut_from(self).unwrap()
    }
}

impl<'b, P, C> SetIndex<OptSet<P, C>> for &'b str
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser + Pre,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    fn ref_from<'a>(&self, set: &'a OptSet<P, C>) -> Result<&'a C::Opt, Error> {
        set.find(*self)?
            .ok_or_else(|| Error::raise_error(format!("Can not find option {}", *self)))
    }

    fn mut_from<'a>(&self, set: &'a mut OptSet<P, C>) -> Result<&'a mut C::Opt, Error> {
        set.find_mut(*self)?
            .ok_or_else(|| Error::raise_error(format!("Can not find option {}", *self)))
    }
}

impl<P, C> Set for OptSet<P, C>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
{
    type Ctor = C;

    fn register(&mut self, ctor: Self::Ctor) -> Option<Self::Ctor> {
        let at = self
            .creators
            .iter()
            .enumerate()
            .find(|(_, v)| v.r#type() == ctor.r#type())
            .map(|v| v.0);

        if let Some(at) = at {
            Some(std::mem::replace(&mut self.creators[at], ctor))
        } else {
            self.creators.push(ctor);
            None
        }
    }

    fn ctor_iter(&self) -> std::slice::Iter<'_, Self::Ctor> {
        self.creators.iter()
    }

    fn ctor_iter_mut(&mut self) -> std::slice::IterMut<'_, Self::Ctor> {
        self.creators.iter_mut()
    }

    fn reset(&mut self) {
        for opt in self.opts.iter_mut() {
            opt.reset();
        }
    }

    fn len(&self) -> usize {
        self.opts.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> std::slice::Iter<'_, SetOpt<Self>> {
        self.opts.iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<'_, SetOpt<Self>> {
        self.opts.iter_mut()
    }

    fn insert(&mut self, mut opt: SetOpt<Self>) -> Uid {
        let uid = self.len() as Uid;

        opt.set_uid(uid);
        self.opts.push(opt);
        uid
    }

    fn get(&self, id: Uid) -> Option<&SetOpt<Self>> {
        self.opts.get(id as usize)
    }

    fn get_mut(&mut self, id: Uid) -> Option<&mut SetOpt<Self>> {
        self.opts.get_mut(id as usize)
    }
}

impl<P, C> OptParser for OptSet<P, C>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser + Pre,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    type Output = P::Output;

    type Error = P::Error;

    fn parse(&self, pattern: Str) -> Result<Self::Output, Self::Error> {
        self.parser().parse(pattern)
    }
}

impl<P, C> Pre for OptSet<P, C>
where
    C: Ctor,
    C::Config: Config,
    P: OptParser + Pre,
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
