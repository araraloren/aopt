use crate::ctx::Invoker;
use crate::opt::AOpt;
use crate::opt::Creator;
use crate::opt::OptConfig;
use crate::opt::StrParser;
use crate::parser::DefaultSetChecker;
use crate::parser::DelayPolicy;
use crate::parser::FwdPolicy;
use crate::parser::Parser;
use crate::parser::Policy;
use crate::parser::PrePolicy;
use crate::prelude::OptParser;
use crate::prelude::OptValidator;
use crate::prelude::SetChecker;
use crate::ser::AppServices;
use crate::set::OptSet;
use crate::set::PrefixOptValidator;
use crate::Error;

pub mod ctx;
pub mod ser;

pub trait APolicyExt<P: Policy> {
    fn default_ser(&self) -> P::Ser;

    fn default_set(&self) -> P::Set;

    fn default_inv<'a>(&self) -> P::Inv<'a>;
}

pub type ACreator = Creator<AOpt, OptConfig, Error>;

pub type ASet = OptSet<StrParser, ACreator, PrefixOptValidator>;

pub type ASer = AppServices;

pub type AInvoker<'a> = Invoker<'a, ASet, ASer>;

pub type AFwdPolicy = FwdPolicy<ASet, ASer, DefaultSetChecker<ASet>>;

pub type APrePolicy = PrePolicy<ASet, ASer, DefaultSetChecker<ASet>>;

pub type ADelayPolicy = DelayPolicy<ASet, ASer, DefaultSetChecker<ASet>>;

pub type AFwdParser<'a> = Parser<'a, AFwdPolicy>;

pub type APreParser<'a> = Parser<'a, APrePolicy>;

pub type ADelayParser<'a> = Parser<'a, ADelayPolicy>;

impl<Set, Ser, Chk> APolicyExt<FwdPolicy<Set, Ser, Chk>> for FwdPolicy<Set, Ser, Chk>
where
    Ser: Default,
    Chk: SetChecker<Set>,
    Set: crate::set::Set + OptParser + OptValidator + Default,
{
    fn default_set(&self) -> Set {
        Set::default()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <FwdPolicy<Set, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<Set, Ser>::default()
    }
}

impl<Set, Ser, Chk> APolicyExt<PrePolicy<Set, Ser, Chk>> for PrePolicy<Set, Ser, Chk>
where
    Ser: Default,
    Chk: SetChecker<Set>,
    Set: crate::set::Set + OptParser + OptValidator + Default,
{
    fn default_set(&self) -> Set {
        Set::default()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <PrePolicy<Set, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<Set, Ser>::default()
    }
}

impl<Set, Ser, Chk> APolicyExt<DelayPolicy<Set, Ser, Chk>> for DelayPolicy<Set, Ser, Chk>
where
    Ser: Default,
    Chk: SetChecker<Set>,
    Set: crate::set::Set + OptParser + OptValidator + Default,
{
    fn default_set(&self) -> Set {
        Set::default()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <DelayPolicy<Set, Ser, Chk> as Policy>::Inv<'a> {
        Invoker::<Set, Ser>::default()
    }
}
