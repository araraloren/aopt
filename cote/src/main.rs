
use cote::Cote;

use aopt::{
    prelude::*,
    set::{SetCfg, SetOpt},
    Error,
};
use cote::{ExtractVal, ExtractValFor, Inject, InjectFrom};

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

#[derive(Debug)]
pub struct Operator<'a> {
    debug: bool,

    copied: Option<Copied<'a>>,

    list: Option<List<'a>>,
}

#[derive(Debug)]
pub struct Copied<'a> {
    from: &'a String,

    to: Option<&'a String>,

    force: bool,

    reverse: bool,
}

#[derive(Debug)]
pub struct List<'a> {
    path: &'a String,

    reverse: bool,

    file: bool,
}

impl<'a, P> Inject<'a, Parser<P>> for Operator<'a>
where
    P::Set: 'static,
    P: Policy<Error = Error>,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    type Ret = ();

    type Error = Error;

    fn inject(parser: &'a mut Parser<P>) -> Result<Self::Ret, Self::Error> {
        parser.add_opt("--debug")?.set_type("b");
        Ok(())
    }
}

impl<'a, P> Inject<'a, Parser<P>> for Copied<'a>
where
    P::Set: 'static,
    P: Policy<Error = Error>,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    type Ret = ();

    type Error = Error;

    fn inject(parser: &'a mut Parser<P>) -> Result<Self::Ret, Self::Error> {
        parser.add_opt("cp=c")?.set_value(false);
        parser.add_opt("--from")?.set_type("s").set_force(true);
        parser.add_opt("--to")?.set_type("s");
        parser.add_opt("--force")?.set_type("b");
        parser.add_opt("--reverse")?.set_type("b");
        Ok(())
    }
}

impl<'a, P> Inject<'a, Parser<P>> for List<'a>
where
    P::Set: 'static,
    P: Policy<Error = Error>,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    type Ret = ();

    type Error = Error;

    fn inject(parser: &'a mut Parser<P>) -> Result<Self::Ret, Self::Error> {
        parser.add_opt("ls=c")?.set_value(false);
        parser.add_opt("--path")?.set_type("s").set_force(true);
        parser.add_opt("--file")?.set_type("b");
        parser.add_opt("--reverse")?.set_type("b");
        Ok(())
    }
}

impl<'a, P> ExtractVal<'a, Parser<P>> for Operator<'a>
where
    P::Set: 'static,
    P: Policy<Error = Error>,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    type Error = Error;

    fn extract_new(parser: &'a Parser<P>) -> Result<Self, Self::Error> {
        let debug = *parser.find_val("--debug")?;

        Ok(Self {
            debug,
            copied: None,
            list: None,
        })
    }
}

impl<'a, P> ExtractValFor<'a, Parser<P>> for Operator<'a>
where
    P::Set: 'static,
    P: Policy<Error = Error>,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    type Error = Error;

    fn extract_for(&mut self, name: &str, parser: &'a Parser<P>) -> Result<&mut Self, Self::Error> {
        match name {
            "cp" => {
                if let Ok(&true) = parser.find_val::<bool>("cp") {
                    self.copied = Copied::extract_new(parser).ok();
                }
            }
            "ls" => {
                if let Ok(&true) = parser.find_val::<bool>("ls") {
                    self.list = List::extract_new(parser).ok();
                }
            }
            _ => {}
        }
        Ok(self)
    }
}

impl<'a, P> ExtractVal<'a, Parser<P>> for Copied<'a>
where
    P::Set: 'static,
    P: Policy<Error = Error>,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    type Error = Error;

    fn extract_new(parser: &'a Parser<P>) -> Result<Self, Self::Error> {
        let from = parser.find_val("--from")?;
        let to = parser.find_val("--to").ok();
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

impl<'a, P> ExtractVal<'a, Parser<P>> for List<'a>
where
    P::Set: 'static,
    P: Policy<Error = Error>,
    SetOpt<P::Set>: Opt,
    P::Set: Set + OptValidator + OptParser,
    <P::Set as OptParser>::Output: Information,
    SetCfg<P::Set>: Config + ConfigValue + Default,
{
    type Error = Error;

    fn extract_new(parser: &'a Parser<P>) -> Result<Self, Self::Error> {
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
    cote.add_sub_parser("ls", AFwdParser::default());
    cote.add_sub_parser("cp", AFwdParser::default());
    cote.link_parser("cote", vec!["ls", "cp"]);
    cote.inject_sub::<List>("ls")?;
    cote.inject_sub::<Copied>("cp")?;

    cote.run_mut(|ret, cote| {
        if ret?.is_some() {
            let mut op = cote.extract_val::<Operator>()?;

            cote.extract_subval_for(&mut op, "ls")?;
            cote.extract_subval_for(&mut op, "cp")?;

            dbg!(op);
        }
        Ok(())
    })?;

    Ok(())
}
