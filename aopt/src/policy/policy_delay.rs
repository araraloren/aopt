use std::fmt::Debug;
use std::marker::PhantomData;

use super::common::invoke_callback_opt;
use super::process_non_opt;
use super::process_opt;
use super::CtxSaver;
use super::Guess;
use super::GuessNOACfg;
use super::GuessOptCfg;
use super::NOAGuess;
use super::OptGuess;
use super::Policy;
use super::UserStyle;
use crate::arg::Args;
use crate::arg::CLOptParser;
use crate::astr;
use crate::ctx::Ctx;
use crate::ext::APolicyExt;
use crate::ext::AServiceExt;
use crate::ext::ASetExt;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::proc::Process;
use crate::ser::CheckService;
use crate::ser::InvokeService;
use crate::ser::Services;
use crate::ser::ServicesExt;
use crate::set::PreSet;
use crate::set::Set;
use crate::Error;
use crate::Str;

#[derive(Debug, Clone)]
pub struct DelayPolicy<S, V> {
    strict: bool,

    contexts: Vec<CtxSaver>,

    marker_s: PhantomData<(S, V)>,
}

impl<S, V> Default for DelayPolicy<S, V>
where
    V: From<Str>,
    S: Set + OptParser,
{
    fn default() -> Self {
        Self {
            strict: true,
            contexts: vec![],
            marker_s: PhantomData::default(),
        }
    }
}

impl<S: 'static, V: 'static> APolicyExt<S, V> for DelayPolicy<S, V> {
    fn new_set<T>() -> T
    where
        T: ASetExt + Set + OptParser,
    {
        T::new_set()
    }

    fn new_services<T>() -> T
    where
        T: AServiceExt<S, V>,
    {
        T::new_services()
    }
}

impl<S, V> DelayPolicy<S, V>
where
    V: From<Str> + 'static,
    S::Opt: Opt,
    S: Set + OptParser + Debug + 'static,
{
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    /// Enable strict mode, if argument is an option, it must be matched.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    pub fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.strict = strict;
        self
    }

    pub fn strict(&self) -> bool {
        self.strict
    }

    pub fn invoke_opt_callback(
        &mut self,
        set: &mut S,
        ser: &mut Services,
        inv_ser: &mut InvokeService<S, V>,
    ) -> Result<(), Error> {
        for saver in std::mem::take(&mut self.contexts) {
            invoke_callback_opt(saver, set, ser, inv_ser)?;
        }
        Ok(())
    }
}

impl<S, V> Policy for DelayPolicy<S, V>
where
    V: From<Str> + 'static,
    S::Opt: Opt,
    S: Set + OptParser + PreSet + Debug + 'static,
{
    type Ret = bool;

    type Value = V;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        args: Args,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        ser.ser::<CheckService<S, V>>()?.pre_check(set)?;

        // take the invoke service, avoid borrow the ser
        let mut is = ser.take_ser::<InvokeService<S, V>>()?;
        let opt_styles = [
            UserStyle::EqualWithValue,
            UserStyle::Argument,
            UserStyle::Boolean,
            UserStyle::CombinedOption,
            UserStyle::EmbeddedValue,
        ];
        let mut noa_args = Args::default();
        let mut iter = args.iter();
        let mut parser = CLOptParser::default();
        let mut opt_ctx = Ctx::default().with_args(args.clone()).with_len(args.len());

        while let Some(_) = iter.next() {
            let mut matched = false;
            let mut consume = false;

            // parsing current argument
            if let Ok(clopt) = iter.parse(&mut parser, set.pre()) {
                for style in opt_styles.iter() {
                    if let Some(mut proc) =
                        OptGuess::new().guess(style, GuessOptCfg::new(&iter, clopt.clone()))?
                    {
                        opt_ctx.set_idx(iter.idx());
                        let ret =
                            process_opt::<S, V>(&opt_ctx, set, ser, &mut proc, &mut is, false)?;

                        if proc.is_mat() {
                            self.contexts.extend(ret);
                            matched = true;
                        }
                        if proc.consume() {
                            consume = true;
                        }
                        if matched {
                            break;
                        }
                    }
                }
                if !matched && self.strict() {
                    let name = clopt.name();
                    return Err(Error::sp_invalid_option_name(name.unwrap_or_default()));
                }
            }

            if matched && consume {
                // if consume the argument, skip it
                let _ = iter.next();
            } else if !matched {
                // add it to NOA if current argument not matched
                if let Some(arg) = iter.cur() {
                    noa_args.push(arg.clone());
                }
            }
        }

        // when style is pos, noa index is [1..=len]
        if noa_args.len() > 0 {
            let mut noa_ctx = Ctx::default()
                .with_args(noa_args.clone())
                .with_len(noa_args.len());

            if let Some(mut proc) = NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(&noa_args, noa_args[0].clone(), 1),
            )? {
                noa_ctx.set_idx(1);
                process_non_opt::<S, V>(&noa_ctx, set, ser, &mut proc, &mut is)?;
            }

            ser.ser::<CheckService<S, V>>()?.cmd_check(set)?;

            for (idx, arg) in noa_args.iter().enumerate() {
                if let Some(mut proc) = NOAGuess::new()
                    .guess(&UserStyle::Pos, GuessNOACfg::new(&noa_args, arg, idx + 1))?
                {
                    noa_ctx.set_idx(idx + 1);
                    process_non_opt::<S, V>(&noa_ctx, set, ser, &mut proc, &mut is)?;
                }
            }
        } else {
            ser.ser::<CheckService<S, V>>()?.cmd_check(set)?;
        }

        // after cmd and pos callback invoked, invoke the callback of option
        self.invoke_opt_callback(set, ser, &mut is)?;

        ser.ser::<CheckService<S, V>>()?.opt_check(set)?;

        ser.ser::<CheckService<S, V>>()?.pos_check(set)?;

        let main_args = args;
        let main_ctx = opt_ctx.set_idx(0);

        if let Some(mut proc) = NOAGuess::new().guess(
            &UserStyle::Main,
            GuessNOACfg::new(&main_args, astr("Main"), 0),
        )? {
            process_non_opt::<S, V>(&main_ctx, set, ser, &mut proc, &mut is)?;
        }

        ser.ser::<CheckService<S, V>>()?.post_check(set)?;
        ser.reg(is);

        Ok(Some(true))
    }
}
