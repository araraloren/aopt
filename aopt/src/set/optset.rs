use std::ffi::OsString;
use std::fmt::Debug;

use crate::opt::Any;
use crate::opt::BuiltInCtor;
use crate::opt::Cmd;
use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Information;
use crate::opt::Main;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::opt::Pos;
use crate::raise_error;
use crate::set::Ctor;
use crate::set::Filter;
use crate::set::FilterMatcher;
use crate::set::FilterMut;
use crate::set::Set;
use crate::set::SetCommit;
use crate::set::SetIndex;
use crate::value::Infer;
use crate::value::Placeholder;
use crate::value::RawValParser;
use crate::Error;
use crate::Str;
use crate::Uid;

use super::OptValidator;
use super::SetOpt;
use super::SetValueFindExt;

/// Simple [`Set`] implementation hold [`Opt`] and [`Ctor`].
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Result;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<()> {
/// let mut set = OptSet::<StrParser, ACreator, PrefixOptValidator>::default();
///
/// // add default and bool creator
/// set.register(Creator::fallback());
/// set.register(Creator::from(aopt::opt::BuiltInCtor::Bool));
///
/// // create a bool option
/// set.add_opt_i::<bool>("--flag")?;
/// assert_eq!(set.add_opt("/flag=b!")?.run()?, 1);
///
/// // filter the set option
/// assert_eq!(set.filter("/flag")?.find_all().count(), 1);
/// assert!(set.find("--flag")?.is_some());
/// # Ok(())
/// # }
/// ```
pub struct OptSet<P, C, V>
where
    C: Ctor,
    P: OptParser,
    V: OptValidator,
{
    parser: P,
    validator: V,
    opts: Vec<C::Opt>,
    creators: Vec<C>,
}

impl<P, C, V> OptSet<P, C, V>
where
    C: Ctor,
    P: OptParser,
    V: OptValidator,
{
    pub fn new(parser: P, validator: V) -> Self {
        Self {
            parser,
            validator,
            opts: vec![],
            creators: vec![],
        }
    }
}

impl<P, C, V> Debug for OptSet<P, C, V>
where
    C::Opt: Debug,
    C: Ctor + Debug,
    C::Config: Debug,
    P: OptParser + Debug,
    V: OptValidator + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptSet")
            .field("parser", &self.parser)
            .field("validator", &self.validator)
            .field("opts", &self.opts)
            .field("creators", &self.creators)
            .finish()
    }
}

impl<P, C, V> Default for OptSet<P, C, V>
where
    C: Ctor,
    P: OptParser + Default,
    V: OptValidator + Default,
{
    fn default() -> Self {
        Self {
            parser: P::default(),
            validator: V::default(),
            opts: vec![],
            creators: vec![],
        }
    }
}

impl<P, C, V> OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    C::Config: Config,
    V: OptValidator,
{
    pub fn with_creator(mut self, creator: C) -> Self {
        self.register(creator);
        self
    }
}

impl<P, C, V> OptSet<P, C, V>
where
    C: Ctor,
    P: OptParser,
    C::Config: Config,
    V: OptValidator,
{
    pub fn parser(&self) -> &P {
        &self.parser
    }

    pub fn parser_mut(&mut self) -> &mut P {
        &mut self.parser
    }
}

impl<P, C, V> OptSet<P, C, V>
where
    C: Ctor,
    P: OptParser,
    V: OptValidator,
{
    pub fn with_validator(mut self, validator: V) -> Self {
        self.validator = validator;
        self
    }

    pub fn set_validator(&mut self, validator: V) -> &mut Self {
        self.validator = validator;
        self
    }

    pub fn validator(&self) -> &V {
        &self.validator
    }

    pub fn validator_mut(&mut self) -> &mut V {
        &mut self.validator
    }
}

impl<P, C, V> OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    V: OptValidator,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    /// Add an option by configuration into current [`OptSet`].
    pub fn add_opt_cfg(
        &mut self,
        config: impl Into<C::Config>,
    ) -> Result<SetCommit<'_, Self, Placeholder>, Error> {
        Ok(SetCommit::new_placeholder(self, config.into()))
    }

    /// Add an option by configuration into current [`OptSet`].
    pub fn add_opt_cfg_i<U>(
        &mut self,
        config: impl Into<C::Config>,
    ) -> Result<SetCommit<'_, Self, U>, Error>
    where
        U: Infer + 'static,
        U::Val: RawValParser,
    {
        let mut info = config.into();

        U::infer_fill_info(&mut info, true);
        Ok(SetCommit::new(self, info))
    }

    /// Add an option into current [`OptSet`].
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`SetCommit`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn add_opt(
        &mut self,
        opt_str: impl Into<Str>,
    ) -> Result<SetCommit<'_, Self, Placeholder>, Error> {
        Ok(SetCommit::new_placeholder(
            self,
            <C::Config as Config>::new(self.parser(), opt_str.into())?,
        ))
    }

    fn infer_fill_type<Cfg>(cfg: &mut Cfg) -> &mut Cfg
    where
        Cfg: ConfigValue,
    {
        if let Some(ctor) = cfg.ctor() {
            let built_in_ctor = BuiltInCtor::from_name(ctor);

            match built_in_ctor {
                BuiltInCtor::Int => cfg.set_type::<i64>(),
                BuiltInCtor::Str => cfg.set_type::<String>(),
                BuiltInCtor::Flt => cfg.set_type::<f64>(),
                BuiltInCtor::Uint => cfg.set_type::<u64>(),
                BuiltInCtor::Bool => cfg.set_type::<bool>(),
                BuiltInCtor::Cmd => cfg.set_type::<Cmd>(),
                BuiltInCtor::Pos => cfg.set_type::<Pos<bool>>(),
                BuiltInCtor::Main => cfg.set_type::<Main>(),
                BuiltInCtor::Any => cfg.set_type::<Any>(),
                BuiltInCtor::Raw => cfg.set_type::<OsString>(),
                BuiltInCtor::Fallback => {
                    unreachable!("Fallback creator can't infer any type")
                }
            }
        } else {
            cfg
        }
    }

    /// Add an option into current [`OptSet`].
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`SetCommit`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn add_opt_i<U>(&mut self, opt_str: impl Into<Str>) -> Result<SetCommit<'_, Self, U>, Error>
    where
        U: Infer + 'static,
        U::Val: RawValParser,
    {
        let mut info = <C::Config as Config>::new(self.parser(), opt_str.into())?;

        U::infer_fill_info(&mut info, true);
        Ok(SetCommit::new(self, info))
    }

    /// Filter the option by configuration.
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`Filter`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn filter<S: Into<Str>>(&self, opt_str: S) -> Result<Filter<'_, Self>, Error> {
        let mut info = <C::Config as Config>::new(self.parser(), opt_str.into())?;

        Self::infer_fill_type(&mut info);
        Ok(Filter::new(self, info))
    }

    /// Filter the option, return the reference of first matched [`Opt`].
    pub fn find<S: Into<Str>>(&self, opt_str: S) -> Result<Option<&C::Opt>, Error> {
        let mut info = <C::Config as Config>::new(self.parser(), opt_str.into())?;

        Self::infer_fill_type(&mut info);
        Ok(self.iter().find(|opt| info.mat_opt(*opt)))
    }

    /// Filter the option, return an iterator of reference of [`Opt`]s.
    pub fn find_all<S: Into<Str>>(
        &self,
        opt_str: S,
    ) -> Result<impl Iterator<Item = &C::Opt>, Error> {
        let mut info = <C::Config as Config>::new(self.parser(), opt_str.into())?;

        Self::infer_fill_type(&mut info);
        Ok(self.iter().filter(move |opt| info.mat_opt(*opt)))
    }

    /// Filter the option by configuration.
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`FilterMut`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn filter_mut<S: Into<Str>>(&mut self, opt_str: S) -> Result<FilterMut<'_, Self>, Error> {
        let mut info = <C::Config as Config>::new(self.parser(), opt_str.into())?;

        Self::infer_fill_type(&mut info);
        Ok(FilterMut::new(self, info))
    }

    /// Filter the option, return the mutable reference of first matched [`Opt`].
    pub fn find_mut<S: Into<Str>>(&mut self, opt_str: S) -> Result<Option<&mut C::Opt>, Error> {
        let mut info = <C::Config as Config>::new(self.parser(), opt_str.into())?;

        Self::infer_fill_type(&mut info);
        Ok(self.iter_mut().find(|opt| info.mat_opt(*opt)))
    }

    /// Filter the option, return an iterator of mutable reference of [`Opt`]s.
    pub fn find_all_mut<S: Into<Str>>(
        &mut self,
        opt_str: S,
    ) -> Result<impl Iterator<Item = &mut C::Opt>, Error> {
        let mut info = <C::Config as Config>::new(self.parser(), opt_str.into())?;

        Self::infer_fill_type(&mut info);
        Ok(self.iter_mut().filter(move |opt| info.mat_opt(*opt)))
    }
}

impl<P, C, V> SetValueFindExt for OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    V: OptValidator,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    fn find_uid<S: Into<Str>>(&self, opt: S) -> Result<Uid, Error> {
        let opt: Str = opt.into();

        self.find(opt.clone())?
            .map(|v| v.uid())
            .ok_or_else(|| raise_error!("Can not find option `{}` in option set", opt))
    }

    fn find_opt<S: Into<Str>>(&self, opt: S) -> Result<&SetOpt<Self>, Error> {
        let opt: Str = opt.into();

        self.find(opt.clone())?
            .ok_or_else(|| raise_error!("Can not find option(ref) `{}` in option set", opt))
    }

    fn find_opt_mut<S: Into<Str>>(&mut self, opt: S) -> Result<&mut SetOpt<Self>, Error> {
        let opt: Str = opt.into();

        self.find_mut(opt.clone())?
            .ok_or_else(|| raise_error!("Can not find option(mut) `{}` in option set", opt))
    }
}

impl<P, C, V, I: SetIndex<OptSet<P, C, V>>> std::ops::Index<I> for OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    V: OptValidator,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    type Output = C::Opt;

    fn index(&self, index: I) -> &Self::Output {
        index.ref_from(self).unwrap()
    }
}

impl<P, C, V, I: SetIndex<OptSet<P, C, V>>> std::ops::IndexMut<I> for OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    V: OptValidator,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.mut_from(self).unwrap()
    }
}

impl<'b, P, C, V> SetIndex<OptSet<P, C, V>> for &'b str
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    V: OptValidator,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    fn ref_from<'a>(&self, set: &'a OptSet<P, C, V>) -> Result<&'a C::Opt, Error> {
        set.find(*self)?
            .ok_or_else(|| raise_error!("Can not find option {}", *self))
    }

    fn mut_from<'a>(&self, set: &'a mut OptSet<P, C, V>) -> Result<&'a mut C::Opt, Error> {
        set.find_mut(*self)?
            .ok_or_else(|| raise_error!("Can not find option {}", *self))
    }
}

impl<P, C, V> Set for OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    V: OptValidator,
{
    type Ctor = C;

    fn register(&mut self, ctor: Self::Ctor) -> Option<Self::Ctor> {
        self.creators.push(ctor);
        None
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

impl<P, C, V> OptParser for OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    V: OptValidator,
    P::Output: Information,
    C::Config: Config + ConfigValue + Default,
{
    type Output = P::Output;

    type Error = P::Error;

    fn parse(&self, pattern: Str) -> Result<Self::Output, Self::Error> {
        self.parser().parse(pattern)
    }
}

impl<P, C, V> OptValidator for OptSet<P, C, V>
where
    C: Ctor,
    P: OptParser,
    V: OptValidator,
{
    type Error = Error;

    fn check(&mut self, name: &str) -> Result<bool, Self::Error> {
        OptValidator::check(&mut self.validator, name).map_err(Into::into)
    }

    fn split<'a>(&self, name: &'a str) -> Result<(&'a str, &'a str), Self::Error> {
        OptValidator::split(&self.validator, name).map_err(Into::into)
    }
}

#[cfg(test)]
mod test {

    use std::any::TypeId;
    use std::ffi::OsString;

    use crate::opt::Cmd;
    use crate::opt::Pos;
    use crate::prelude::*;
    use crate::Error;

    #[test]
    fn test_add_option() {
        assert!(test_add_option_impl().is_ok());
    }

    fn test_add_option_impl() -> Result<(), Error> {
        let mut set = aset_with_default_creators();

        assert!(set.add_opt("set;s=c")?.run().is_ok());
        assert!(set.add_opt_i::<Cmd>("g;get")?.run().is_ok());

        assert!(set.add_opt("vala=p@2")?.run().is_ok());
        assert!(set.add_opt("valb=p@2..5")?.run().is_ok());
        assert!(set.add_opt("valc=p@1..4")?.run().is_ok());
        assert!(set.add_opt("vald=p!@2")?.run().is_ok());
        assert!(set.add_opt("vale=p!@2..4")?.run().is_ok());
        assert!(set.add_opt("valf=p@-2")?.run().is_ok());
        assert!(set.add_opt("valg=p!@-1")?.run().is_ok());
        assert!(set
            .add_opt("valh@[1,2,6]")?
            .set_infer::<Option<Pos<f64>>>()
            .run()
            .is_ok());
        assert!(set.add_opt_i::<Option<Pos>>("vali!@*")?.run().is_ok());
        assert!(set.add_opt_i::<Option<Pos<bool>>>("valj!@1")?.run().is_ok());

        assert!(set.add_opt("main=m")?.run().is_ok());

        assert!(set.add_opt("-b;--bool=b")?.run().is_ok());
        assert!(set.add_opt_i::<bool>("-ba;--boola")?.run().is_ok());
        assert!(set.add_opt("--boolb=b!")?.run().is_ok());
        assert!(set.add_opt_i::<bool>("-bc;--boolc")?.run().is_ok());
        assert!(set.add_opt("--boold!")?.set_infer::<bool>().run().is_ok());
        assert!(set.add_opt("-/be;--/boole=b")?.run().is_ok());
        assert!(set.add_opt_i::<bool>("-/bf;--/boolf")?.run().is_ok());
        assert!(set.add_opt_i::<bool>("-/bg;--/boolg!")?.run().is_ok());
        assert!(set.add_opt_i::<bool>("/boolh")?.run().is_ok());
        assert!(set.add_opt_i::<bool>("/booli!")?.run().is_ok());

        assert!(set.add_opt("--float=f")?.run().is_ok());
        assert!(set.add_opt_i::<Option<f64>>("-fa;--floata")?.run().is_ok());
        assert!(set.add_opt("-fb;--floatb=f")?.run().is_ok());
        assert!(set
            .add_opt_i::<Option<f64>>("-fc;--floatc=f")?
            .run()
            .is_ok());
        assert!(set.add_opt("--floatd=f!")?.run().is_ok());
        assert!(set.add_opt_i::<Option<f64>>("-fe;--floate!")?.run().is_ok());

        assert!(set.add_opt("--int=i")?.run().is_ok());
        assert!(set.add_opt_i::<Option<i64>>("-i")?.run().is_ok());
        assert!(set.add_opt("-ia;--inta=i")?.run().is_ok());
        assert!(set.add_opt("-ib;--intb=i!")?.run().is_ok());
        assert!(set.add_opt_i::<Option<i64>>("--intc!")?.run().is_ok());
        assert!(set.add_opt("-id;--intd=i!")?.run().is_ok());

        assert!(set.add_opt("--uint=u")?.add_alias("-u").run().is_ok());
        assert!(set.add_opt("-ua;--uinta=u")?.run().is_ok());
        assert!(set
            .add_opt("--ub;--uintb")?
            .set_infer::<Option<u64>>()
            .run()
            .is_ok());
        assert!(set
            .add_opt_i::<Option<u64>>("--uintc=u")?
            .set_force(true)
            .run()
            .is_ok());
        assert!(set
            .add_opt("--uintd")?
            .set_infer::<Option<u64>>()
            .set_force(true)
            .run()
            .is_ok());
        assert!(set.add_opt("--uinte")?.set_force(true).run().is_err());

        assert!(set.add_opt("-s=s")?.run().is_ok());
        assert!(set.add_opt_i::<Option<String>>("--str")?.run().is_ok());
        assert!(set
            .add_opt_i::<Option<String>>("--stra")?
            .add_alias("/stra")
            .run()
            .is_ok());
        assert!(set.add_opt_i::<Option<String>>("--strb!")?.run().is_ok());
        assert!(set.add_opt("--strc=s")?.add_alias("/strc").run().is_ok());
        assert!(set.add_opt("/stre;--strd")?.set_ctor("s").run().is_ok());
        assert!(set
            .add_opt("!")?
            .set_name("--strf")
            .set_ctor("s")
            .run()
            .is_ok());

        assert!(set.add_opt("--raw=r")?.run().is_ok());
        assert!(set
            .add_opt_i::<Option<OsString>>("-raw;--raw-value")?
            .run()
            .is_ok());

        assert_eq!(set.len(), 49);
        assert!(set.find("s=c")?.is_some());
        assert_eq!(set.find_all("=c")?.count(), 2);

        assert_eq!(set.find_all("=p")?.count(), 9);
        assert!(set.filter("")?.set_type::<Pos<bool>>().find().is_some());
        assert!(set.filter("")?.set_type::<Pos<f64>>().find().is_some());
        assert_eq!(set.find_all_mut("@2")?.count(), 2);
        assert_eq!(set.filter_mut("=p")?.set_force(true).find_all().count(), 5);
        assert!(set.filter("=p")?.set_name("vala").find().is_some());
        assert!(set.filter("=p")?.set_name("valw").find().is_none());

        assert!(set.find("main")?.is_some());

        assert_eq!(set.find_all("=b")?.count(), 10);
        assert_eq!(set.find_all_mut("=b")?.count(), 10);
        assert!(set.find("-b")?.is_some());
        assert_eq!(set.find_all("=b!")?.count(), 4);
        assert!(set.find("--boola")?.is_some());
        assert!(set.find("/booli")?.is_some());
        assert_eq!(set.filter_mut("--boolc")?.find_all().count(), 1);

        assert_eq!(set.find_all("=i")?.count(), 6);
        assert!(set.find("-ia")?.is_some());
        assert!(set.find("-ib")?.is_some());
        assert!(set.filter("--intd")?.set_type::<i64>().find().is_some());
        assert!(set.filter("--intw")?.set_type::<i64>().find().is_none());
        assert_eq!(set.find_all_mut("=i!")?.count(), 3);

        assert_eq!(set.find_all("=f")?.count(), 6);
        assert!(set.find("-fa")?.is_some());
        assert!(set.find("-fb")?.is_some());
        assert!(set.find("-fc")?.is_some());
        assert!(set.find("--floatd")?.is_some());
        assert_eq!(
            set.filter("")?
                .set_type::<f64>()
                .set_force(true)
                .find_all()
                .count(),
            2
        );

        assert_eq!(set.find_all("=u")?.count(), 5);
        assert!(set.find("--ub")?.is_some());
        assert!(set.find("--uintc=u")?.is_some());
        assert!(set.find("--uintd")?.is_some());
        assert!(set.find("=u!")?.is_some());
        assert_eq!(set.find_all_mut("=u!")?.count(), 2);
        assert_eq!(set.filter("!")?.set_type::<u64>().find_all().count(), 2);

        assert_eq!(set.find_all("=s")?.count(), 7);
        assert!(set.find("--strc")?.is_some());
        assert_eq!(set.find_all("/stra")?.count(), 1);
        assert_eq!(set.find_all_mut("=s!")?.count(), 2);
        assert_eq!(set.filter("--strf")?.find_all().count(), 1);

        assert_eq!(set["s"].uid(), 0);
        assert_eq!(set[2].name(), "vala");
        assert_eq!(set["vali"].index(), Some(&Index::anywhere()));
        assert_eq!(set["/booli"].force(), true);
        assert_eq!(set["--floata"].name(), "-fa");
        assert_eq!(set["-ib=i"].r#type(), &TypeId::of::<i64>());
        assert_eq!(set.opt(43)?.name(), "--strb");

        // you can add option with different prefix,
        // but you can't set it if validator not support it
        assert!(set.add_opt_i::<bool>("+flag")?.run().is_ok());
        assert_eq!(set["+flag"].uid(), 49);

        Ok(())
    }
}
