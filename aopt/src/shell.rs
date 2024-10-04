mod guess;
mod policy;

use std::ffi::OsStr;
use std::io::Write;

pub use self::guess::CompleteGuess;
pub use self::guess::CompleteRet;
pub use self::policy::CompletePolicy;

use crate::args::Args;
use crate::ctx::Ctx;
use crate::ctx::Invoker;
use crate::ext::AFwdParser;
use crate::ext::APolicyExt;
use crate::ext::ASer;
use crate::ext::ASet;
use crate::opt::Action;
use crate::opt::ConfigBuildInfer;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::opt::Style;
use crate::parser::Parser;
use crate::parser::Policy;
use crate::parser::PolicyParser;
use crate::prelude::MutOpt;
use crate::prelude::SetValueFindExt;
use crate::set::OptValidator;
use crate::set::SetOpt;
use crate::value::raw2str;
use crate::value::RawValParser;
use crate::Error;
use crate::Uid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shell {
    Zsh,

    Bash,

    Fish,
}

impl RawValParser for Shell {
    type Error = Error;

    fn parse(raw: Option<&OsStr>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let name = raw2str(raw)?;
        let name = name.to_lowercase();

        match name.as_str() {
            "zsh" => Ok(Shell::Zsh),
            "bash" => Ok(Shell::Bash),
            "fish" => Ok(Shell::Fish),
            _ => Err(crate::raise_failure!("Unknow shell type: {}", name).with_uid(ctx.uid()?)),
        }
    }
}

pub type ACompletePolicy = CompletePolicy<ASet, ASer>;

pub type ACompleteParser<'a> = Parser<'a, ACompletePolicy>;

impl<Set, Ser> APolicyExt<CompletePolicy<Set, Ser>> for CompletePolicy<Set, Ser>
where
    Ser: Default,
    Set: crate::set::Set + OptParser + OptValidator + Default,
{
    fn default_set(&self) -> Set {
        Set::default()
    }

    fn default_ser(&self) -> Ser {
        Ser::default()
    }

    fn default_inv<'a>(&self) -> <CompletePolicy<Set, Ser> as Policy>::Inv<'a> {
        Invoker::<Set, Ser>::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CompleteService<Set, Ser> {
    compl_policy: CompletePolicy<Set, Ser>,

    avail_cmd: Vec<Uid>,

    avail_opt: Vec<Uid>,

    avail_pos: Vec<Uid>,

    display_cmd: bool,

    incomplete_opt: Option<Uid>,
}

impl<Set, Ser> CompleteService<Set, Ser> {
    pub fn display_cmd(&self) -> bool {
        self.display_cmd
    }

    pub fn avail_cmd(&self) -> &[Uid] {
        &self.avail_cmd
    }

    pub fn avail_opt(&self) -> &[Uid] {
        &self.avail_opt
    }

    pub fn avail_pos(&self) -> &[Uid] {
        &self.avail_pos
    }

    pub fn incomplete_opt(&self) -> Option<&Uid> {
        self.incomplete_opt.as_ref()
    }

    pub fn reset(&mut self) -> &mut Self {
        self.display_cmd = true;
        self.avail_cmd.clear();
        self.avail_opt.clear();
        self.incomplete_opt = None;
        self
    }
}

impl<Set, Ser> CompleteService<Set, Ser>
where
    Set: crate::set::Set + OptParser + OptValidator,
{
    pub fn parse_with<P>(
        &mut self,
        args: Args,
        parser: &mut P,
    ) -> Result<<Self as Policy>::Ret, P::Error>
    where
        P: PolicyParser<CompleteService<Set, Ser>>,
    {
        parser.parse_policy(args, self)
    }

    pub fn write_complete_to<R: Write>(
        &mut self,
        set: &Set,
        writer: &mut R,
        shell: Shell,
    ) -> Result<(), Error>
    where
        SetOpt<Set>: Opt,
    {
        use crate::set::SetExt;

        if matches!(shell, Shell::Zsh) {
            writeln!(writer, "local -a subcmds\nsubcmds=(\n")
                .map_err(|e| crate::raise_error!("Can not write data: {:?}", e))?;
        }
        if self.display_cmd {
            for uid in self.avail_cmd.iter() {
                let opt = set.opt(*uid)?;
                let name = opt.name();
                let help = opt.help();

                Self::write_to(writer, name, help, shell)?;
                if let Some(alias) = opt.alias() {
                    for alias in alias {
                        Self::write_to(writer, alias, help, shell)?;
                    }
                }
            }
        } else {
            for uid in self.avail_opt.iter() {
                let opt = set.opt(*uid)?;
                let name = opt.name();
                let hint = opt.hint();
                let mut help = opt.help().to_string();

                if opt.mat_style(Style::Argument) && !hint.is_empty() {
                    if let Some((_, val)) = hint.split_once(' ') {
                        let val = val.trim();

                        if !val.is_empty() {
                            help = format!("{}: {}", help, val);
                        }
                    }
                }
                Self::write_to(writer, name, &help, shell)?;
                if let Some(alias) = opt.alias() {
                    for alias in alias {
                        Self::write_to(writer, alias, &help, shell)?;
                    }
                }
            }
            for uid in self.avail_pos.iter() {
                let opt = set.opt(*uid)?;
                let hint = opt.hint();
                let help = opt.help();

                Self::write_to(writer, hint, help, shell)?;
            }
        }
        if matches!(shell, Shell::Zsh) {
            writeln!(writer, ")\n_describe 'available values' subcmds\n")
                .map_err(|e| crate::raise_error!("Can not write data: {:?}", e))?;
        }
        Ok(())
    }

    fn write_to<W: Write>(
        writer: &mut W,
        hint: &str,
        help: &str,
        shell: Shell,
    ) -> Result<(), Error> {
        match shell {
            Shell::Zsh => {
                if help.is_empty() {
                    writeln!(writer, " '{}' ", hint)
                        .map_err(|e| crate::raise_error!("Can not write data: {:?}", e))?;
                } else {
                    writeln!(writer, " '{}:{}' ", hint, help)
                        .map_err(|e| crate::raise_error!("Can not write data: {:?}", e))?;
                }
            }
            Shell::Bash => {
                writeln!(writer, "{}", hint)
                    .map_err(|e| crate::raise_error!("Can not write data: {:?}", e))?;
            }
            Shell::Fish => {
                if help.is_empty() {
                    writeln!(writer, "{}", hint)
                        .map_err(|e| crate::raise_error!("Can not write data: {:?}", e))?;
                } else {
                    writeln!(writer, "{}\t\"{}\"", hint, help)
                        .map_err(|e| crate::raise_error!("Can not write data: {:?}", e))?;
                }
            }
        }
        Ok(())
    }

    fn process_last_arg(
        &mut self,
        set: &mut <Self as Policy>::Set,
        last: &OsStr,
    ) -> Result<(), Error> {
        #[allow(unused)]
        let mut win_os_string = None;
        let mut arg: &std::ffi::OsStr = last;

        #[allow(clippy::needless_option_as_deref)]
        if let Some((opt, _)) = crate::str::split_once(arg, '=') {
            win_os_string = Some(opt);
            arg = win_os_string.as_deref().unwrap();
        }

        self.set_incomplete_opt(
            set,
            arg.to_str()
                .ok_or_else(|| crate::raise_failure!("Can't convert value `{:?}` to str", arg))?,
        )
    }

    fn set_incomplete_opt(
        &mut self,
        set: &mut <Self as Policy>::Set,
        arg: &str,
    ) -> Result<(), Error> {
        let arg = std::borrow::Cow::Borrowed(arg);

        if set.split(&arg).is_ok() {
            for opt in set.iter() {
                if opt.mat_style(Style::Argument) && opt.name() == arg {
                    self.incomplete_opt = Some(opt.uid());
                    break;
                }
            }
        }
        Ok(())
    }
}

impl<Set, Ser> Policy for CompleteService<Set, Ser>
where
    SetOpt<Set>: Opt,
    Set: crate::set::Set + OptParser + OptValidator,
{
    type Ret = <CompletePolicy<Set, Ser> as Policy>::Ret;

    type Set = Set;

    type Inv<'a> = <CompletePolicy<Set, Ser> as Policy>::Inv<'a>;

    type Ser = Ser;

    type Error = Error;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv<'_>,
        ser: &mut Self::Ser,
        args: Args,
    ) -> Result<Self::Ret, Self::Error> {
        let tot = args.len();
        let last = args.last().cloned();
        let ret = <CompletePolicy<Set, Ser> as Policy>::parse(
            &mut self.compl_policy,
            set,
            inv,
            ser,
            args,
        )?;
        let mut need_cmd = true;

        for opt in set.iter() {
            if opt.mat_style(Style::Cmd) {
                if opt.matched() {
                    need_cmd = false;
                    self.avail_cmd.clear();
                } else {
                    self.avail_cmd.push(opt.uid());
                }
            }
        }
        if need_cmd && self.avail_cmd.is_empty() {
            self.display_cmd = false;
        } else {
            self.display_cmd = need_cmd;
        }
        if !self.display_cmd {
            for opt in set.iter() {
                if opt.mat_style(Style::Argument)
                    || opt.mat_style(Style::Boolean)
                    || opt.mat_style(Style::Combined)
                    || opt.mat_style(Style::Flag)
                {
                    let action = opt.action();
                    let uid = opt.uid();
                    let matched = opt.matched();

                    match action {
                        Action::Set => {
                            if !matched {
                                self.avail_opt.push(uid);
                            }
                        }
                        _ => {
                            self.avail_opt.push(uid);
                        }
                    }
                }
            }
        }
        if let Some(last) = last {
            self.process_last_arg(set, &last)?;
        }
        if self.incomplete_opt.is_none() && !self.display_cmd {
            for opt in set.iter() {
                if opt.mat_style(Style::Pos)
                    && !opt.matched()
                    && opt.mat_index(Some((tot, tot + 1)))
                {
                    self.avail_pos.push(opt.uid());
                }
            }
        }
        Ok(ret)
    }
}

/// Check and return the value of `--_completes` option from `std::env::args()`.
pub fn try_get_complete() -> Result<Option<(String, Shell)>, Error> {
    let mut try_parser = AFwdParser::default();

    try_parser.add_opt("--_completes: Get complete option or sub command".infer::<String>())?;
    try_parser
        .add_opt("--_shell!: Set shell type, support zsh fish bash".infer::<MutOpt<Shell>>())?;
    try_parser.parse_env()?;

    if let Ok(cl) = try_parser.take_val("--_completes") {
        Ok(Some((cl, *try_parser.find_val::<Shell>("--_shell")?)))
    } else {
        Ok(None)
    }
}
