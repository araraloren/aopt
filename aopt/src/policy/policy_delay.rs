use std::fmt::Debug;
use std::marker::PhantomData;

use super::invoke_callback_opt;
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
use crate::args::ArgParser;
use crate::args::Args;
use crate::astr;
use crate::ctx::Ctx;
use crate::ext::ServicesExt;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::proc::Process;
use crate::ser::InvokeService;
use crate::ser::Services;
use crate::set::Pre;
use crate::set::Set;
use crate::set::SetExt;
use crate::Arc;
use crate::Error;

#[derive(Debug, Clone)]
pub struct DelayPolicy<S> {
    strict: bool,

    contexts: Vec<CtxSaver>,

    marker_s: PhantomData<S>,
}

impl<S> Default for DelayPolicy<S>
where
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

impl<S> DelayPolicy<S>
where
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
        inv_ser: &mut InvokeService<S>,
    ) -> Result<(), Error> {
        for saver in std::mem::take(&mut self.contexts) {
            invoke_callback_opt(saver, set, ser, inv_ser)?;
        }
        Ok(())
    }

    /// Return the NOA index base on 1.
    pub fn noa_idx(idx: usize) -> usize {
        idx + 1
    }
}

impl<S> Policy for DelayPolicy<S>
where
    S::Opt: Opt,
    S: Set + OptParser + Pre + Debug + 'static,
{
    type Ret = bool;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        args: Arc<Args>,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        let keys = set.keys().to_vec();

        for id in keys {
            set.opt_mut(id)?.init(ser)?;
        }
        ser.ser_check()?.pre_check(set)?;

        // take the invoke service, avoid borrow the ser
        let mut is = ser.take::<InvokeService<S>>()?;
        let opt_styles = [
            UserStyle::EqualWithValue,
            UserStyle::Argument,
            UserStyle::Boolean,
            UserStyle::CombinedOption,
            UserStyle::EmbeddedValue,
        ];
        let args_len = args.len();
        let mut noa_args = Args::default();
        let mut iter = args.iter().enumerate();
        let mut opt_ctx = Ctx::default();

        opt_ctx.set_args(args.clone()).set_total(args_len);
        while let Some((idx, (opt, arg))) = iter.next() {
            let mut matched = false;
            let mut consume = false;
            let arg = arg.map(|v| Arc::new(v.clone()));

            // parsing current argument
            if let Ok(clopt) = opt.parse_arg(set.prefix()) {
                for style in opt_styles.iter() {
                    if let Some(mut proc) = OptGuess::new()
                        .guess(style, GuessOptCfg::new(idx, args_len, arg.clone(), &clopt))?
                    {
                        opt_ctx.set_idx(idx);
                        let ret = process_opt::<S>(&opt_ctx, set, ser, &mut proc, &mut is, false)?;

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
                    let default_str = astr("");

                    return Err(Error::sp_invalid_option_name(format!(
                        "{}{}",
                        clopt.prefix().unwrap_or(&default_str),
                        clopt.name().unwrap_or(&default_str)
                    )));
                }
            }

            // if consume the argument, skip it
            if matched && consume {
                iter.next();
            } else if !matched {
                // add it to NOA if current argument not matched
                noa_args.push(args[idx].clone());
            }
        }

        let noa_args = Arc::new(noa_args);
        let noa_len = noa_args.len();
        let mut noa_ctx = Ctx::default();

        noa_ctx.set_args(noa_args.clone()).set_total(noa_args.len());
        // when style is pos, noa index is [1..=len]
        if noa_args.len() > 0 {
            if let Some(mut proc) = NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(noa_args.clone(), Self::noa_idx(0), noa_len),
            )? {
                noa_ctx.set_idx(Self::noa_idx(0));
                process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is)?;
            }

            ser.ser_check()?.cmd_check(set)?;

            for idx in 0..noa_len {
                if let Some(mut proc) = NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), Self::noa_idx(idx), noa_len),
                )? {
                    noa_ctx.set_idx(Self::noa_idx(idx));
                    process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is)?;
                }
            }
        } else {
            ser.ser_check()?.cmd_check(set)?;
        }

        // after cmd and pos callback invoked, invoke the callback of option
        self.invoke_opt_callback(set, ser, &mut is)?;

        ser.ser_check()?.opt_check(set)?;

        ser.ser_check()?.pos_check(set)?;

        let main_args = noa_args;
        let mut main_ctx = noa_ctx;

        main_ctx.set_idx(0);
        if let Some(mut proc) =
            NOAGuess::new().guess(&UserStyle::Main, GuessNOACfg::new(main_args, 0, noa_len))?
        {
            process_non_opt::<S>(&main_ctx, set, ser, &mut proc, &mut is)?;
        }

        ser.ser_check()?.post_check(set)?;
        ser.register(is);

        Ok(Some(true))
    }
}
