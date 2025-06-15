use std::borrow::Cow;

use aopt::prelude::ConfigValue;
use aopt::prelude::Opt;
use aopt::prelude::OptValidator;
use aopt::prelude::Set;
use aopt::prelude::SetCfg;
use aopt::prelude::SetOpt;
use aopt::prelude::SetValueFindExt;
use aopt::prelude::Style;
use aopt::shell::shell;
use aopt::shell::shell::complete_eq;
use aopt::shell::shell::complete_opt;
use aopt::shell::shell::complete_val;
use aopt::shell::shell::Shell;
use aopt::shell::Context;
use aopt::trace;

use crate::parser::Parser;
use crate::Error;

pub use aopt::shell::get_complete_cli;
pub use aopt::shell::shell::Complete;
pub use aopt::shell::CompleteCli;

impl<'a, S> Complete<SetOpt<S>> for Parser<'a, S>
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
        let mut parser = self;
        let mut flags = vec![false; args.len()];
        let mut cmds = vec![];
        let mut parsers = vec![self];

        for (idx, arg) in args.iter().enumerate() {
            if let Some(arg) = arg.to_str() {
                for cmd in parser.optset().iter().filter(|v| v.mat_style(Style::Cmd)) {
                    trace!("finding `{}` in `{}`", arg, parser.name());
                    if cmd.mat_name(Some(arg)) || cmd.mat_alias(arg) {
                        parser = parser.find_parser(cmd.name())?;

                        flags[idx] = true;
                        cmds.push(cmd);
                        parsers.push(parser);
                        trace!("find cmd `{}` in args at `{}`", arg, idx);
                        break;
                    }
                }
            }
        }

        let mut available_cmds = vec![];

        // find cmd if val is none
        if let (Some(parser), None) = (parsers.last(), &val) {
            trace!("try complete cmd");
            let arg = arg.to_str().unwrap_or_default();

            for opt in parser.iter().filter(|v| v.mat_style(Style::Cmd)) {
                for name in std::iter::once(opt.name())
                    .chain(
                        opt.alias()
                            .iter()
                            .flat_map(|v| v.iter().map(|v| v.as_str())),
                    )
                    .filter(|v| v.starts_with(arg))
                {
                    trace!("available cmd -> {name}");
                    available_cmds.push((name, opt));
                }
            }
        }

        // find option value like [arg=val]
        if let (Some(arg), Some(val)) = (arg.to_str(), val.as_ref()) {
            let bytes = val.as_encoded_bytes();

            trace!("search.1 vals with arg=`{}`, val=`{}`", arg, val.display());
            for p in parsers
                .iter()
                .filter(|v| v.split(&Cow::Borrowed(arg)).is_ok())
            {
                complete_eq(arg, bytes, p.iter(), values, |name, val, opt| {
                    s.write_eq(name, val, opt)
                })?;
            }
        }

        let mut found_val = false;

        // find option value like [arg val]
        if let (Some(arg), Some(val)) = (prev.to_str(), Some(&arg)) {
            let bytes = val.as_encoded_bytes();

            trace!("search.2 vals with arg=`{}`, val=`{}`", arg, val.display());
            for p in parsers
                .iter()
                .filter(|v| v.split(&Cow::Borrowed(arg)).is_ok())
            {
                found_val = found_val
                    || complete_val(arg, bytes, p.iter(), values, |val, opt| {
                        s.write_val(val, opt)
                    })?;
            }
        }

        if !found_val && !available_cmds.is_empty() {
            for (cmd, opt) in available_cmds {
                s.write_cmd(cmd, opt)?;
            }
            return s.finish();
        }

        // find option if val is none
        if let (Some(arg), None) = (arg.to_str(), val) {
            trace!("search option with arg=`{}`", arg);
            for p in parsers
                .iter()
                .filter(|v| v.split(&Cow::Borrowed(arg)).is_ok())
            {
                complete_opt(arg, p.iter(), |name, opt| s.write_opt(name, opt))?;
            }
        }

        s.finish()
    }
}
