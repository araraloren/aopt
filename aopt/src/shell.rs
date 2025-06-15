use std::borrow::Cow;
use std::ffi::OsString;

use crate::acore::args::Args;
use crate::acore::opt::Opt;
use crate::acore::Error;
use crate::acore::HashMap;
use crate::ashell::shell::complete_cmd;
use crate::ashell::shell::complete_eq;
use crate::ashell::shell::complete_opt;
use crate::ashell::shell::complete_val;
use crate::ashell::shell::Complete;
use crate::ashell::shell::Shell;
use crate::opt::ConfigBuildInfer;
use crate::opt::ConfigValue;
use crate::opt::Style;
use crate::parser::HCOptSet;
use crate::parser::PolicySettings;
use crate::prelude::AFwdParser;
use crate::set::OptValidator;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetOpt;
use crate::set::SetValueFindExt;
use crate::trace;

pub use crate::ashell::script;
pub use crate::ashell::shell;
pub use crate::ashell::value;
pub use crate::ashell::Context;

#[derive(Debug)]
pub struct CompleteCli {
    pub args: Vec<OsString>,

    pub curr: OsString,

    pub prev: OsString,

    pub shell: String,

    pub script: bool,
}

impl CompleteCli {
    pub fn parse_env() -> Result<Self, Error> {
        Self::parse(Args::from_env())
    }

    /// Check and return the value of `--_completes` option from `std::env::args()`.
    pub fn parse(args: Args) -> Result<Self, Error> {
        let mut parser = AFwdParser::default();

        parser.policy_mut().set_prepolicy(true);

        parser.add_opt("--_shell!: Set shell type".infer::<String>())?;
        parser.add_opt("--_curr: Set the current word".infer::<OsString>())?;
        parser.add_opt("--_prev: Set the previous word".infer::<OsString>())?;

        let mut ret = parser.parse(args)?;
        let mut args = ret.take_args();
        let (curr, prev) = (parser.take_val("--_curr"), parser.take_val("--_prev"));

        args.remove(0);
        if let Ok(shell) = parser.take_val("--_shell") {
            let script = curr.is_err() && prev.is_err();
            let curr = curr.unwrap_or_default();
            let prev = prev.unwrap_or_default();

            Ok(Self {
                shell,
                args,
                curr,
                prev,
                script,
            })
        } else {
            Err(crate::error!("can not get complete arguments"))
        }
    }

    pub fn get_context<O: Opt>(&self) -> Result<Context<'_, O>, Error> {
        Ok(Context::new(&self.args, &self.curr, &self.prev))
    }

    pub fn gen_with<F>(&self, mut func: F) -> Result<(), Error>
    where
        F: FnMut(&mut Box<dyn crate::shell::script::Generator<Err = Error>>) -> Result<(), Error>,
    {
        if self.script {
            let mut m = crate::ashell::script::Manager::default();

            func(m.find_mut(&self.shell)?)
        } else {
            Err(crate::error!(
                "can not generate string: script = {}",
                self.script
            ))
        }
    }

    pub fn write_stdout(&self, name: &str, bin: &str) -> Result<(), Error> {
        self.gen_with(|g| {
            print!("{}", g.generate(name, bin)?);
            std::io::Write::flush(&mut std::io::stdout())
                .map_err(|e| crate::error!("can not flush stdout: {e:?}"))?;
            Ok(())
        })
    }

    pub fn complete<'a, O, W, F>(&self, mut func: F) -> Result<(), Error>
    where
        W: std::io::Write + 'a,
        O: crate::opt::Opt + 'a,
        F: FnMut(
            &mut Box<dyn crate::shell::shell::Shell<O, W, Err = Error> + 'a>,
        ) -> Result<(), Error>,
    {
        if !self.script {
            let mut m = crate::ashell::shell::Manager::<'a, O, W>::default();

            func(m.find_mut(&self.shell)?)
        } else {
            Err(crate::error!(
                "can not perform completion: script = {}",
                self.script
            ))
        }
    }
}

#[derive(Debug)]
pub struct HCOptSetManager<'a, S> {
    optset: HCOptSet<'a, S>,
    suboptset: HashMap<String, HCOptSetManager<'a, S>>,
}

impl<'a, S> HCOptSetManager<'a, S>
where
    S: Set,
    SetOpt<S>: Opt,
{
    pub fn new(optset: HCOptSet<'a, S>) -> Self {
        Self {
            optset,
            suboptset: HashMap::default(),
        }
    }

    pub fn with_optset(mut self, optset: HCOptSet<'a, S>) -> Self {
        self.optset = optset;
        self
    }

    pub fn with_submanager(mut self, name: &str, optset: HCOptSet<'a, S>) -> Result<Self, Error> {
        self.add_submanager(name, optset)?;
        Ok(self)
    }

    pub fn set_optset(&mut self, optset: HCOptSet<'a, S>) -> &mut Self {
        self.optset = optset;
        self
    }

    pub fn add_submanager(
        &mut self,
        name: &str,
        optset: HCOptSet<'a, S>,
    ) -> Result<&mut Self, Error> {
        if self
            .optset
            .iter()
            .filter(|v| v.mat_style(aopt_core::opt::Style::Cmd))
            .any(|v| v.name() == name)
        {
            self.suboptset
                .insert(name.to_string(), HCOptSetManager::new(optset));
            Ok(self)
        } else {
            Err(crate::error!("not a sub command name: {name}"))
        }
    }

    pub fn optset(&self) -> &HCOptSet<'a, S> {
        &self.optset
    }

    pub fn find_submanager(&self, name: &str) -> Result<&HCOptSetManager<'a, S>, Error> {
        self.suboptset
            .get(name)
            .ok_or_else(|| crate::error!("can not find suboptset: {name}"))
    }
}

impl<'a, S> Complete<SetOpt<S>> for HCOptSetManager<'a, S>
where
    SetOpt<S>: Opt,
    SetCfg<S>: ConfigValue + Default,
    S: Set + OptValidator + SetValueFindExt,
{
    type Out = ();
    type Ctx<'b> = Context<'b, SetOpt<S>>;
    type Err = Error;

    fn complete<T, W>(&self, s: &mut T, ctx: &mut Self::Ctx<'_>) -> Result<Self::Out, Self::Err>
    where
        T: Shell<SetOpt<S>, W>,
    {
        let Context {
            args,
            arg,
            val,
            prev,
            values,
        } = ctx;

        trace!("complete -> prev = {}", prev.display());
        trace!("complete -> arg = {}", arg.display());
        trace!("complete -> val = {:?}", val.as_ref().map(|v| v.display()));
        trace!("complete -> args = {:?}", args);

        let mut s = shell::wrapref(s);
        let mut manager = self;
        let mut flags = vec![false; args.len()];
        let mut cmds = vec![];
        let mut sub_managers = vec![self];

        for (idx, arg) in args.iter().enumerate() {
            if let Some(arg) = arg.to_str() {
                trace!("finding `{}`", arg);
                for cmd in manager.optset().iter().filter(|v| v.mat_style(Style::Cmd)) {
                    trace!("checking `{}`", cmd.name());
                    if cmd.mat_name(Some(arg)) || cmd.mat_alias(arg) {
                        manager = manager.find_submanager(cmd.name())?;

                        flags[idx] = true;
                        cmds.push(cmd);
                        sub_managers.push(manager);
                        trace!("find cmd `{}` in args at `{}`", arg, idx);
                        break;
                    }
                }
            }
        }

        let optsets: Vec<_> = sub_managers.iter().map(|v| v.optset()).collect();

        // find cmd if val is none
        if val.is_none() {
            if let Some(optset) = optsets.last() {
                trace!("try complete cmd");
                if complete_cmd(
                    arg.to_str().unwrap_or_default(),
                    optset.iter(),
                    |cmd, opt| s.write_cmd(cmd, opt),
                )? {
                    return Ok(());
                }
            }
        }

        // find option value like [arg=val]
        if let (Some(arg), Some(val)) = (arg.to_str(), val.as_ref()) {
            let bytes = val.as_encoded_bytes();

            trace!("search.1 vals with arg=`{}`, val=`{}`", arg, val.display());
            for p in optsets
                .iter()
                .filter(|v| v.split(&Cow::Borrowed(arg)).is_ok())
            {
                complete_eq(arg, bytes, p.iter(), values, |name, val, opt| {
                    s.write_eq(name, val, opt)
                })?;
            }
        }

        // find option value like [arg val]
        if let (Some(arg), Some(val)) = (prev.to_str(), Some(&arg)) {
            let bytes = val.as_encoded_bytes();

            trace!("search.2 vals with arg=`{}`, val=`{}`", arg, val.display());
            for p in optsets
                .iter()
                .filter(|v| v.split(&Cow::Borrowed(arg)).is_ok())
            {
                complete_val(arg, bytes, p.iter(), values, |val, opt| {
                    s.write_val(val, opt)
                })?;
            }
        }

        // find option if val is none
        if val.is_none() {
            if let Some(arg) = arg.to_str() {
                trace!("search option with arg=`{}`", arg);
                for p in optsets
                    .iter()
                    .filter(|v| v.split(&Cow::Borrowed(arg)).is_ok())
                {
                    complete_opt(arg, p.iter(), |name, opt| s.write_opt(name, opt))?;
                }
            }
        }

        s.finish()
    }
}

/// Return [`Arguments`] if command line arguments has `--_shell` option.
pub fn get_complete_cli() -> Result<CompleteCli, Error> {
    CompleteCli::parse_env()
}
