use std::fmt::Debug;
use std::marker::PhantomData;

use super::process_non_opt;
use super::process_opt;
use super::Guess;
use super::GuessNOACfg;
use super::GuessOptCfg;
use super::NOAGuess;
use super::OptGuess;
use super::Policy;
use super::UserStyle;
use crate::arg::ArgParser;
use crate::arg::Args;
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
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Str;

#[derive(Debug, Clone)]
pub struct ForwardPolicy<S> {
    strict: bool,

    marker_s: PhantomData<S>,
}

impl<S> Default for ForwardPolicy<S> {
    fn default() -> Self {
        Self {
            strict: true,
            marker_s: PhantomData::default(),
        }
    }
}

impl<S: 'static> APolicyExt<S, RawVal> for ForwardPolicy<S> {
    fn new_set<T>() -> T
    where
        T: ASetExt + Set + OptParser,
    {
        T::new_set()
    }

    fn new_services<T>() -> T
    where
        T: AServiceExt<S, RawVal>,
    {
        T::new_services()
    }
}

impl<S> ForwardPolicy<S>
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

    pub fn get_strict(&self) -> bool {
        self.strict
    }
}

impl<S> Policy for ForwardPolicy<S>
where
    S::Opt: Opt,
    S: Set + OptParser + PreSet + Debug + 'static,
{
    type Ret = bool;

    type Value = RawVal;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        args: Args,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        ser.ser::<CheckService<S>>()?.pre_check(set)?;

        // take the invoke service, avoid borrow the ser
        let mut is = ser.take_ser::<InvokeService<S>>()?;
        let stys = [
            UserStyle::EqualWithValue,
            UserStyle::Argument,
            UserStyle::Boolean,
            UserStyle::CombinedOption,
            UserStyle::EmbeddedValue,
        ];
        let args = Arc::new(args);
        let args_len = args.len();
        let mut noa_args = Args::default();
        let mut iter = args.iter().enumerate();
        let mut opt_ctx = Ctx::new_opt();

        opt_ctx.opt_mut()?.set_args(args.clone()).set_len(args_len);

        while let Some((idx, (opt, arg))) = iter.next() {
            let mut matched = false;
            let mut consume = false;
            let arg = arg.map(|v| Arc::new(v.clone()));

            if let Ok(clopt) = opt.parse(set.pre()) {
                for style in stys.iter() {
                    if let Some(mut proc) = OptGuess::new()
                        .guess(style, GuessOptCfg::new(idx, args_len, arg.clone(), &clopt))?
                    {
                        opt_ctx.opt_mut()?.set_idx(idx);
                        process_opt::<S>(&opt_ctx, set, ser, &mut proc, &mut is, true)?;
                        if proc.is_mat() {
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
                if !matched && self.get_strict() {
                    let name = clopt.name();
                    return Err(Error::sp_invalid_option_name(
                        name.cloned().unwrap_or_else(|| Str::default()),
                    ));
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

        ser.ser::<CheckService<S>>()?.opt_check(set)?;

        let noa_args = Arc::new(noa_args);
        let noa_len = noa_args.len();
        let mut noa_ctx = Ctx::new_noa();

        noa_ctx
            .noa_mut()?
            .set_args(noa_args.clone())
            .set_len(noa_args.len());

        // when style is pos, noa index is [1..=len]
        if noa_args.len() > 0 {
            if let Some(mut proc) = NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(noa_args.clone(), 1, noa_len),
            )? {
                noa_ctx.opt_mut()?.set_idx(1);
                process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is)?;
            }

            ser.ser::<CheckService<S>>()?.cmd_check(set)?;

            for idx in 0..noa_len {
                if let Some(mut proc) = NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), idx + 1, noa_len),
                )? {
                    noa_ctx.opt_mut()?.set_idx(idx + 1);
                    process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is)?;
                }
            }
        } else {
            ser.ser::<CheckService<S>>()?.cmd_check(set)?;
        }
        ser.ser::<CheckService<S>>()?.pos_check(set)?;

        let main_args = noa_args;
        let mut main_ctx = noa_ctx;

        main_ctx.noa_mut()?.set_idx(0);
        if let Some(mut proc) =
            NOAGuess::new().guess(&UserStyle::Main, GuessNOACfg::new(main_args, 0, noa_len))?
        {
            process_non_opt::<S>(&main_ctx, set, ser, &mut proc, &mut is)?;
        }

        ser.ser::<CheckService<S>>()?.post_check(set)?;
        ser.reg(is);

        Ok(Some(true))
    }
}
