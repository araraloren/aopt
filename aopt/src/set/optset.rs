use std::borrow::Cow;
use std::fmt::Debug;

use ahash::HashMapExt;

use crate::opt::Cid;
use crate::opt::ConfigBuild;
use crate::opt::ConfigValue;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::raise_error;
use crate::set::Ctor;
use crate::set::Filter;
use crate::set::FilterMatcher;
use crate::set::FilterMut;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetCommit;
use crate::set::SetIndex;
use crate::value::Infer;
use crate::value::RawValParser;
use crate::Error;
use crate::HashMap;
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
/// set.register(Creator::from(aopt::opt::Cid::Bool));
///
/// // create a bool option
/// set.add_opt("--flag".infer::<bool>())?;
/// assert_eq!(set.add_opt("/flag=b!")?.run()?, 1);
///
/// // filter the set option
/// assert_eq!(set.filter("/flag")?.find_all().count(), 1);
/// assert!(set.find("--flag").is_ok());
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
    creators: HashMap<Cid, C>,
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
            creators: HashMap::new(),
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
    C: Ctor + From<Cid>,
    P: OptParser + Default,
    V: OptValidator + Default,
{
    fn default() -> Self {
        Self {
            parser: P::default(),
            validator: V::default(),
            opts: vec![],
            creators: crate::ctors!(C),
        }
    }
}

impl<P, C, V> OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
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

    /// Take all the options
    pub fn take_options(&mut self) -> Option<Vec<C::Opt>> {
        let mut ret = None;

        if !self.opts.is_empty() {
            ret = Some(std::mem::take(&mut self.opts));
        }
        ret
    }
}

impl<P, C, V> OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    V: OptValidator,
    P::Output: Information,
    C::Config: ConfigValue + Default,
{
    /// Add an option into current [`OptSet`].
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`SetCommit`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn add_opt<B>(&mut self, cb: B) -> Result<SetCommit<'_, Self, B::Val>, Error>
    where
        B::Val: Infer + 'static,
        B: ConfigBuild<C::Config>,
        <B::Val as Infer>::Val: RawValParser,
    {
        Ok(SetCommit::new(self, cb.build(self.parser())?))
    }

    /// Filter the option by configuration.
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`Filter`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn filter(&self, cb: impl ConfigBuild<C::Config>) -> Result<Filter<'_, Self>, Error> {
        let mut info = cb.build(self.parser())?;

        info.infer_builtin_ty();
        Ok(Filter::new(self, info))
    }

    /// Filter the option, return the reference of first matched [`Opt`].
    pub fn find(&self, cb: impl ConfigBuild<C::Config>) -> Result<&C::Opt, Error> {
        let mut info = cb.build(self.parser())?;

        info.infer_builtin_ty();
        self.iter().find(|opt| info.mat_opt(*opt)).ok_or_else(|| {
            raise_error!(
                "can not find option with: {:?}={:?}",
                info.name(),
                info.ctor()
            )
        })
    }

    /// Filter the option, return an iterator of reference of [`Opt`]s.
    pub fn find_all(
        &self,
        cb: impl ConfigBuild<C::Config>,
    ) -> Result<impl Iterator<Item = &C::Opt>, Error> {
        let mut info = cb.build(self.parser())?;

        info.infer_builtin_ty();
        Ok(self.iter().filter(move |opt| info.mat_opt(*opt)))
    }

    /// Filter the option by configuration.
    ///
    /// It parsing the given option string `S` using inner [`OptParser`], return an [`FilterMut`].
    /// For option string, reference [`StrParser`](crate::opt::StrParser).
    pub fn filter_mut(
        &mut self,
        cb: impl ConfigBuild<C::Config>,
    ) -> Result<FilterMut<'_, Self>, Error> {
        let mut info = cb.build(self.parser())?;

        info.infer_builtin_ty();
        Ok(FilterMut::new(self, info))
    }

    /// Filter the option, return the mutable reference of first matched [`Opt`].
    pub fn find_mut(&mut self, cb: impl ConfigBuild<C::Config>) -> Result<&mut C::Opt, Error> {
        let mut info = cb.build(self.parser())?;

        info.infer_builtin_ty();
        self.iter_mut()
            .find(|opt| info.mat_opt(*opt))
            .ok_or_else(|| {
                raise_error!(
                    "can not find option with: {:?}={:?}",
                    info.name(),
                    info.ctor()
                )
            })
    }

    /// Filter the option, return an iterator of mutable reference of [`Opt`]s.
    pub fn find_all_mut(
        &mut self,
        cb: impl ConfigBuild<C::Config>,
    ) -> Result<impl Iterator<Item = &mut C::Opt>, Error> {
        let mut info = cb.build(self.parser())?;

        info.infer_builtin_ty();
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
    C::Config: ConfigValue + Default,
{
    fn find_uid(&self, cb: impl ConfigBuild<SetCfg<Self>>) -> Result<Uid, Error> {
        self.find(cb).map(|v| v.uid())
    }

    fn find_opt(&self, cb: impl ConfigBuild<SetCfg<Self>>) -> Result<&SetOpt<Self>, Error> {
        self.find(cb)
    }

    fn find_opt_mut(
        &mut self,
        cb: impl ConfigBuild<SetCfg<Self>>,
    ) -> Result<&mut SetOpt<Self>, Error> {
        self.find_mut(cb)
    }
}

impl<P, C, V, I: SetIndex<OptSet<P, C, V>>> std::ops::Index<I> for OptSet<P, C, V>
where
    C::Opt: Opt,
    C: Ctor,
    P: OptParser,
    V: OptValidator,
    P::Output: Information,
    C::Config: ConfigValue + Default,
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
    C::Config: ConfigValue + Default,
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
    C::Config: ConfigValue + Default,
{
    fn ref_from<'a>(&self, set: &'a OptSet<P, C, V>) -> Result<&'a C::Opt, Error> {
        set.find(*self)
            .map_err(|e| raise_error!("can not find option {}: {e:?}", *self))
    }

    fn mut_from<'a>(&self, set: &'a mut OptSet<P, C, V>) -> Result<&'a mut C::Opt, Error> {
        set.find_mut(*self)
            .map_err(|e| raise_error!("can not find option {}: {e:?}", *self))
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
        self.creators.insert(ctor.cid().clone(), ctor);
        None
    }

    fn get_ctor(&self, name: &str) -> Option<&Self::Ctor> {
        self.creators.get(&Cid::from(name))
    }

    fn get_ctor_mut(&mut self, name: &str) -> Option<&mut Self::Ctor> {
        self.creators.get_mut(&Cid::from(name))
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
    C::Config: ConfigValue + Default,
{
    type Output = P::Output;

    type Error = P::Error;

    fn parse_opt(&self, pattern: &str) -> Result<Self::Output, Self::Error> {
        self.parser().parse_opt(pattern)
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

    fn split<'a>(&self, name: &Cow<'a, str>) -> Result<(Cow<'a, str>, Cow<'a, str>), Self::Error> {
        OptValidator::split(&self.validator, name).map_err(Into::into)
    }
}

#[cfg(test)]
mod test {

    use std::any::TypeId;
    use std::ffi::OsString;

    use crate::opt::Cmd;
    use crate::opt::ConfigBuildInfer;
    use crate::opt::Pos;
    use crate::prelude::*;
    use crate::Error;

    #[test]
    fn test_add_option() {
        assert!(test_add_option_impl().is_ok());
    }

    fn test_add_option_impl() -> Result<(), Error> {
        let mut set = ASet::default();

        #[cfg(not(target_os = "windows"))]
        {
            set.validator_mut().add_prefix("/");
        }

        assert!(set.add_opt("set;s=c")?.run().is_ok());
        assert!(set.add_opt("g;get".infer::<Cmd>())?.run().is_ok());

        assert!(set.add_opt("vala=p@2")?.run().is_ok());
        assert!(set.add_opt("valb=p@2..5")?.run().is_ok());
        assert!(set.add_opt("valc=p@1..4")?.run().is_ok());
        assert!(set.add_opt("vald=p!@2")?.run().is_ok());
        assert!(set.add_opt("vale=p!@2..4")?.run().is_ok());
        assert!(set.add_opt("valf=p@-2")?.run().is_ok());
        assert!(set.add_opt("valg=p!@-1")?.run().is_ok());
        assert!(set
            .add_opt("valh@[1,2,6]")?
            .set_infer::<Pos<f64>>()
            .run()
            .is_ok());
        assert!(set.add_opt("vali!@*".infer::<Pos>())?.run().is_ok());
        assert!(set.add_opt("valj!@1".infer::<Pos<bool>>())?.run().is_ok());

        assert!(set.add_opt("main=m")?.run().is_ok());

        assert!(set.add_opt("-b;--bool=b")?.run().is_ok());
        assert!(set.add_opt("-ba;--boola".infer::<bool>())?.run().is_ok());
        assert!(set.add_opt("--boolb=b!")?.run().is_ok());
        assert!(set.add_opt("-bc;--boolc".infer::<bool>())?.run().is_ok());
        assert!(set.add_opt("--boold!")?.set_infer::<bool>().run().is_ok());
        assert!(set.add_opt("-/be;--/boole=b")?.run().is_ok());
        assert!(set.add_opt("-/bf;--/boolf".infer::<bool>())?.run().is_ok());
        assert!(set.add_opt("-/bg;--/boolg!".infer::<bool>())?.run().is_ok());
        assert!(set.add_opt("/boolh".infer::<bool>())?.run().is_ok());
        assert!(set.add_opt("/booli!".infer::<bool>())?.run().is_ok());

        assert!(set.add_opt("--float=f")?.run().is_ok());
        assert!(set.add_opt("-fa;--floata".infer::<f64>())?.run().is_ok());
        assert!(set.add_opt("-fb;--floatb=f")?.run().is_ok());
        assert!(set.add_opt("-fc;--floatc=f".infer::<f64>())?.run().is_ok());
        assert!(set.add_opt("--floatd=f!")?.run().is_ok());
        assert!(set.add_opt("-fe;--floate!".infer::<f64>())?.run().is_ok());

        assert!(set.add_opt("--int=i")?.run().is_ok());
        assert!(set.add_opt("-i".infer::<i64>())?.run().is_ok());
        assert!(set.add_opt("-ia;--inta=i")?.run().is_ok());
        assert!(set.add_opt("-ib;--intb=i!")?.run().is_ok());
        assert!(set.add_opt("--intc!".infer::<i64>())?.run().is_ok());
        assert!(set.add_opt("-id;--intd=i!")?.run().is_ok());

        assert!(set.add_opt("--uint=u")?.add_alias("-u").run().is_ok());
        assert!(set.add_opt("-ua;--uinta=u")?.run().is_ok());
        assert!(set
            .add_opt("--ub;--uintb")?
            .set_infer::<u64>()
            .run()
            .is_ok());
        assert!(set
            .add_opt("--uintc=u".infer::<u64>())?
            .set_force(true)
            .run()
            .is_ok());
        assert!(set
            .add_opt("--uintd")?
            .set_infer::<u64>()
            .set_force(true)
            .run()
            .is_ok());
        assert!(set.add_opt("--uinte")?.set_force(true).run().is_err());

        assert!(set.add_opt("-s=s")?.run().is_ok());
        assert!(set.add_opt("--str".infer::<String>())?.run().is_ok());
        assert!(set
            .add_opt("--stra".infer::<String>())?
            .add_alias("/stra")
            .run()
            .is_ok());
        assert!(set.add_opt("--strb!".infer::<String>())?.run().is_ok());
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
            .add_opt("-raw;--raw-value".infer::<OsString>())?
            .run()
            .is_ok());

        assert_eq!(set.len(), 49);
        assert!(set.find("s=c").is_ok());
        assert_eq!(set.find_all("=c")?.count(), 2);

        assert_eq!(set.find_all("=p")?.count(), 9);
        assert!(set.filter("")?.set_type::<Pos<bool>>().find().is_some());
        assert!(set.filter("")?.set_type::<Pos<f64>>().find().is_some());
        assert_eq!(set.find_all_mut("@2")?.count(), 2);
        assert_eq!(set.filter_mut("=p")?.set_force(true).find_all().count(), 5);
        assert!(set.filter("=p")?.set_name("vala").find().is_some());
        assert!(set.filter("=p")?.set_name("valw").find().is_none());

        assert!(set.find("main").is_ok());

        assert_eq!(set.find_all("=b")?.count(), 10);
        assert_eq!(set.find_all_mut("=b")?.count(), 10);
        assert!(set.find("-b").is_ok());
        assert_eq!(set.find_all("=b!")?.count(), 4);
        assert!(set.find("--boola").is_ok());
        assert!(set.find("/booli").is_ok());
        assert_eq!(set.filter_mut("--boolc")?.find_all().count(), 1);

        assert_eq!(set.find_all("=i")?.count(), 6);
        assert!(set.find("-ia").is_ok());
        assert!(set.find("-ib").is_ok());
        assert!(set.filter("--intd")?.set_type::<i64>().find().is_some());
        assert!(set.filter("--intw")?.set_type::<i64>().find().is_none());
        assert_eq!(set.find_all_mut("=i!")?.count(), 3);

        assert_eq!(set.find_all("=f")?.count(), 6);
        assert!(set.find("-fa").is_ok());
        assert!(set.find("-fb").is_ok());
        assert!(set.find("-fc").is_ok());
        assert!(set.find("--floatd").is_ok());
        assert_eq!(
            set.filter("")?
                .set_type::<f64>()
                .set_force(true)
                .find_all()
                .count(),
            2
        );

        assert_eq!(set.find_all("=u")?.count(), 5);
        assert!(set.find("--ub").is_ok());
        assert!(set.find("--uintc=u").is_ok());
        assert!(set.find("--uintd").is_ok());
        assert!(set.find("=u!").is_ok());
        assert_eq!(set.find_all_mut("=u!")?.count(), 2);
        assert_eq!(set.filter("!")?.set_type::<u64>().find_all().count(), 2);

        assert_eq!(set.find_all("=s")?.count(), 7);
        assert!(set.find("--strc").is_ok());
        assert_eq!(set.find_all("/stra")?.count(), 1);
        assert_eq!(set.find_all_mut("=s!")?.count(), 2);
        assert_eq!(set.filter("--strf")?.find_all().count(), 1);

        assert_eq!(set["s"].uid(), 0);
        assert_eq!(set[2].name(), "vala");
        assert_eq!(set["vali"].index(), Some(&Index::anywhere()));
        assert!(set["/booli"].force());
        assert_eq!(set["--floata"].name(), "-fa");
        assert_eq!(set["-ib=i"].r#type(), &TypeId::of::<i64>());
        assert_eq!(set.opt(43)?.name(), "--strb");

        // you can add option with different prefix,
        // but you can't set it if validator not support it
        assert!(set.add_opt("+flag".infer::<bool>())?.run().is_ok());
        assert_eq!(set["+flag"].uid(), 49);

        Ok(())
    }
}
