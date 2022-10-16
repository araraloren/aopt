use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

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
use crate::RawString;
use crate::Str;

/// [`PrePolicy`] matching the command line arguments with [`Opt`] in the [`Set`].
/// [`PrePolicy`] will skip any special [`Error`] during [`parse`](Policy::parse) process.
/// [`PrePolicy`] will return the left `NOA`s after parsing.
/// [`PrePolicy`] don't consume the `NOA` when process [`NOAMatch`](crate::proc::NOAMatch).
#[derive(Debug, Clone)]
pub struct PrePolicy<S> {
    marker_s: PhantomData<S>,
}

impl<S> Default for PrePolicy<S> {
    fn default() -> Self {
        Self {
            marker_s: PhantomData::default(),
        }
    }
}

impl<S: 'static> APolicyExt<S, RawString> for PrePolicy<S> {
    fn new_set<T>() -> T
    where
        T: ASetExt + Set + OptParser,
    {
        T::new_set()
    }

    fn new_services<T>() -> T
    where
        T: AServiceExt<S, RawString>,
    {
        T::new_services()
    }
}

impl<S> PrePolicy<S> {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    /// Ignore failure when parsing.
    pub fn ig_failure<T: Default>(res: Result<T, Error>) -> Result<Option<T>, Error> {
        match res {
            Ok(val) => Ok(Some(val)),
            Err(e) => {
                if e.is_failure() {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }
}

impl<S> Policy for PrePolicy<S>
where
    S::Opt: Opt,
    S: Set + OptParser + PreSet + Debug + 'static,
{
    type Ret = Vec<RawString>;

    type Value = RawString;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        args: Args,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        Self::ig_failure(ser.ser::<CheckService<S, RawString>>()?.pre_check(set))?;

        // take the invoke service, avoid borrow the ser
        let mut is = ser.take_ser::<InvokeService<S, RawString>>()?;
        let opt_styles = [
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
                for style in opt_styles.iter() {
                    let ret = Self::ig_failure(
                        OptGuess::new()
                            .guess(style, GuessOptCfg::new(idx, args_len, arg.clone(), &clopt)),
                    )?;

                    if ret.is_none() {
                        continue;
                    }
                    if let Some(Some(mut proc)) = ret {
                        opt_ctx.opt_mut()?.set_idx(idx);
                        if Self::ig_failure(process_opt::<S>(
                            &opt_ctx, set, ser, &mut proc, &mut is, true,
                        ))?
                        .is_none()
                        {
                            continue;
                        }
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
            }

            // if consume the argument, skip it
            if matched && consume {
                iter.next();
            } else if !matched {
                // add it to NOA if current argument not matched
                noa_args.push(args[idx].clone());
            }
        }

        Self::ig_failure(ser.ser::<CheckService<S, RawString>>()?.opt_check(set))?;

        // save the return value
        let pre_ret = noa_args.deref().clone();
        let noa_args = Arc::new(noa_args);
        let noa_len = noa_args.len();
        let mut noa_ctx = Ctx::new_noa();

        noa_ctx
            .noa_mut()?
            .set_args(noa_args.clone())
            .set_len(noa_args.len());

        if noa_args.len() > 0 {
            if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(noa_args.clone(), 1, noa_len),
            ))? {
                noa_ctx.noa_mut()?.set_idx(1);
                Self::ig_failure(process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is))?;
            }

            Self::ig_failure(ser.ser::<CheckService<S, RawString>>()?.cmd_check(set))?;

            for (idx, arg) in noa_args.iter().enumerate() {
                if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), idx + 1, noa_len),
                ))? {
                    noa_ctx.noa_mut()?.set_idx(idx + 1);
                    Self::ig_failure(process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is))?;
                }
            }
        } else {
            Self::ig_failure(ser.ser::<CheckService<S, RawString>>()?.cmd_check(set))?;
        }

        Self::ig_failure(ser.ser::<CheckService<S, RawString>>()?.pos_check(set))?;

        let main_args = noa_args;
        let mut main_ctx = noa_ctx;

        main_ctx.noa_mut()?.set_idx(0);

        if let Some(Some(mut proc)) = Self::ig_failure(
            NOAGuess::new().guess(&UserStyle::Main, GuessNOACfg::new(main_args, 0, noa_len)),
        )? {
            Self::ig_failure(process_non_opt::<S>(
                &main_ctx, set, ser, &mut proc, &mut is,
            ))?;
        }

        ser.ser::<CheckService<S, RawString>>()?.post_check(set)?;
        ser.reg(is);

        Ok(Some(pre_ret))
    }
}
