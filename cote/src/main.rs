use aopt::{
    prelude::*,
    set::{SetCfg, SetOpt},
    Error,
};
use cote::{Cote, InferConfig};
use cote::{ExtractVal, Inject};

// acfg(no_alias)
//
// aopt(skip)
//
// aopt(name, type, hint, val, vals, force, index, alias, action, assoc, handler)
//
// opt(name, type, hint, val, vals, force, alias, action, assoc, handler)
//
// pos(name, type, hint, val, vals, force, index, action, assoc, handler)
//
// cmd(name, type, hint, val, vals, force, action, assoc, handler)

// #[derive(Cote, Debug, Clone, Default)]
// #[cote(no_alias)]
// pub struct Copied {
//     // What do you want
//     #[cote(name = "-f", value = 48)]
//     from: String,

//     #[cote(name = "-t", value = "uou", alias = "--to")]
//     to: String,

//     force: bool,

//     count: Vec<String>,
// }

fn policy() -> &'static str {
    "pre"
}

#[derive(Debug, Clone, Default, cote_derive::Cote)]
#[cote(policy = "pre")]
pub struct Widget<'a> {
    copied: std::option::Option<&'a Option<Vec<bool>>>,
}

#[derive(Debug)]
pub struct Operator<'a> {
    debug: bool,

    copied: Option<Copied<'a>>,

    list: Option<List<'a>>,
}

#[derive(Debug)]
pub struct Copied<'a> {
    from: &'a String,

    force: bool,

    reverse: bool,

    to: &'a Vec<String>,
}

#[derive(Debug)]
pub struct List<'a> {
    path: &'a String,

    reverse: bool,

    file: bool,
}

impl<'a, P, E> Inject<'a, Cote<P, E>> for Operator<'a>
where
    SetOpt<P::Set>: Opt,
    SetOpt<E::Set>: Opt,
    <P::Set as OptParser>::Output: Information,
    <E::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    SetCfg<E::Set>: Config + ConfigValue + Default,
    P: Policy<Error = Error> + APolicyExt<P::Set> + Default,
    E: Policy<Error = Error> + APolicyExt<E::Set> + Default,
    P::Set: Set + OptValidator + OptParser + Default + 'static,
    E::Set: Set + OptValidator + OptParser + Default + 'static,
{
    type Ret = ();

    type Error = Error;

    fn inject(cote: &'a mut Cote<P, E>) -> Result<Self::Ret, Self::Error> {
        let major_parser = cote.maj_parser_mut();

        major_parser.add_opt("--debug=b")?;

        let mut ls_parser = Parser::<E>::default();
        let mut cp_parser = Parser::<E>::default();

        ls_parser.add_opt("ls=c")?.set_value(false);
        cp_parser.add_opt("cp=c")?.set_value(false);
        cote.add_sub_parser("cp", cp_parser)
            .add_sub_parser("ls", ls_parser);
        cote.link_parser("cote", vec!["ls", "cp"]);

        Copied::inject(cote)?;
        List::inject(cote)?;
        Ok(())
    }
}

impl<'a, P, E> Inject<'a, Cote<P, E>> for Copied<'a>
where
    SetOpt<P::Set>: Opt,
    SetOpt<E::Set>: Opt,
    <P::Set as OptParser>::Output: Information,
    <E::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    SetCfg<E::Set>: Config + ConfigValue + Default,
    P: Policy<Error = Error> + APolicyExt<P::Set> + Default,
    E: Policy<Error = Error> + APolicyExt<E::Set> + Default,
    P::Set: Set + OptValidator + OptParser + Default + 'static,
    E::Set: Set + OptValidator + OptParser + Default + 'static,
{
    type Ret = ();

    type Error = Error;

    fn inject(cote: &'a mut Cote<P, E>) -> Result<Self::Ret, Self::Error> {
        let parser = cote.sub_parser_mut("cp")?;

        parser.add_opt("--from")?.set_type("s").set_force(true);
        parser
            .add_opt("to")?
            .set_type("p")
            .set_idx(Index::parse("2..")?)
            .set_assoc(Assoc::Str)
            .set_initiator(ValInitiator::empty::<String>());
        parser.add_opt("--force")?.set_type("b");
        parser.add_opt("--reverse")?.set_type("b");
        Ok(())
    }
}

impl<'a, P, E> Inject<'a, Cote<P, E>> for List<'a>
where
    SetOpt<P::Set>: Opt,
    SetOpt<E::Set>: Opt,
    <P::Set as OptParser>::Output: Information,
    <E::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    SetCfg<E::Set>: Config + ConfigValue + Default,
    P: Policy<Error = Error> + APolicyExt<P::Set> + Default,
    E: Policy<Error = Error> + APolicyExt<E::Set> + Default,
    P::Set: Set + OptValidator + OptParser + Default + 'static,
    E::Set: Set + OptValidator + OptParser + Default + 'static,
{
    type Ret = ();

    type Error = Error;

    fn inject(cote: &'a mut Cote<P, E>) -> Result<Self::Ret, Self::Error> {
        let parser = cote.sub_parser_mut("ls")?;

        parser.add_opt("--path")?.set_type("s").set_force(true);
        parser.add_opt("--file")?.set_type("b");
        parser.add_opt("--reverse")?.set_type("b");
        Ok(())
    }
}

impl<'a, P, E> ExtractVal<'a, Cote<P, E>> for Operator<'a>
where
    P: Policy<Error = Error>,
    E: Policy<Error = Error>,
    P::Set: Set + OptParser,
    E::Set: Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    <E::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    SetCfg<E::Set>: Config + ConfigValue + Default,
{
    type Error = Error;

    fn extract_new(cote: &'a Cote<P, E>) -> Result<Self, Self::Error> {
        let debug = *cote.maj_parser().find_val("--debug")?;
        let copied = Copied::extract_new(cote).ok();
        let list = List::extract_new(cote).ok();

        Ok(Self {
            debug,
            copied,
            list,
        })
    }
}

impl<'a, P, E> ExtractVal<'a, Cote<P, E>> for Copied<'a>
where
    P: Policy<Error = Error>,
    E: Policy<Error = Error>,
    P::Set: Set + OptParser,
    E::Set: Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    <E::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    SetCfg<E::Set>: Config + ConfigValue + Default,
{
    type Error = Error;

    fn extract_new(cote: &'a Cote<P, E>) -> Result<Self, Self::Error> {
        let parser = cote.sub_parser("cp")?;
        let from = parser.find_val("--from")?;
        let to = parser.find_vals("to")?;
        let force = *parser.find_val("--force")?;
        let reverse = *parser.find_val("--reverse")?;

        Ok(Self {
            from,
            to,
            force,
            reverse,
        })
    }
}

impl<'a, P, E> ExtractVal<'a, Cote<P, E>> for List<'a>
where
    P: Policy<Error = Error>,
    E: Policy<Error = Error>,
    P::Set: Set + OptParser,
    E::Set: Set + OptParser,
    <P::Set as OptParser>::Output: Information,
    <E::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
    SetCfg<E::Set>: Config + ConfigValue + Default,
{
    type Error = Error;

    fn extract_new(cote: &'a Cote<P, E>) -> Result<Self, Self::Error> {
        let parser = cote.sub_parser("ls")?;
        let path = parser.find_val("--path")?;
        let file = *parser.find_val("--file")?;
        let reverse = *parser.find_val("--reverse")?;

        Ok(Self {
            path,
            file,
            reverse,
        })
    }
}

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut cote = Cote::<APrePolicy, AFwdPolicy>::default();

    cote.inject::<Operator>()?;

    cote.run_mut(|ret, cote| {
        if ret?.is_some() {
            let op = cote.extract_val::<Operator>()?;

            dbg!(op);
        }
        Ok(())
    })?;

    Ok(())
}
