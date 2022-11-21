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
use crate::args::ArgParser;
use crate::args::Args;
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

    /// Return the NOA index base on 1.
    pub fn noa_idx(idx: usize) -> usize {
        idx + 1
    }
}

impl<S> Policy for PrePolicy<S>
where
    S::Opt: Opt,
    S: Set + OptParser + Pre + Debug + 'static,
{
    type Ret = Args;

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
            Self::ig_failure(set.opt_mut(id)?.init(ser))?;
        }
        Self::ig_failure(ser.ser_check()?.pre_check(set))?;

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

            if let Ok(clopt) = opt.parse_arg(set.prefix()) {
                for style in opt_styles.iter() {
                    let ret = Self::ig_failure(
                        OptGuess::new()
                            .guess(style, GuessOptCfg::new(idx, args_len, arg.clone(), &clopt)),
                    )?;

                    if let Some(Some(mut proc)) = ret {
                        opt_ctx.set_idx(idx);
                        if Self::ig_failure(process_opt::<S>(
                            &opt_ctx, set, ser, &mut proc, &mut is, true,
                        ))?
                        .is_some()
                        {
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
            }

            // if consume the argument, skip it
            if matched && consume {
                iter.next();
            } else if !matched {
                // add it to NOA if current argument not matched
                noa_args.push(args[idx].clone());
            }
        }

        Self::ig_failure(ser.ser_check()?.opt_check(set))?;

        let ret = noa_args.clone();
        let noa_args = Arc::new(noa_args);
        let noa_len = noa_args.len();
        let mut noa_ctx = Ctx::default();

        noa_ctx.set_args(noa_args.clone()).set_total(noa_args.len());

        if noa_args.len() > 0 {
            if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(noa_args.clone(), Self::noa_idx(0), noa_len),
            ))? {
                noa_ctx.set_idx(Self::noa_idx(0));
                Self::ig_failure(process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is))?;
            }

            Self::ig_failure(ser.ser_check()?.cmd_check(set))?;

            for idx in 0..noa_len {
                if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), Self::noa_idx(idx), noa_len),
                ))? {
                    noa_ctx.set_idx(Self::noa_idx(idx));
                    Self::ig_failure(process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is))?;
                }
            }
        } else {
            Self::ig_failure(ser.ser_check()?.cmd_check(set))?;
        }

        Self::ig_failure(ser.ser_check()?.pos_check(set))?;

        let main_args = noa_args;
        let mut main_ctx = noa_ctx;

        // set 0 for Main's index
        main_ctx.set_idx(0);

        if let Some(Some(mut proc)) = Self::ig_failure(
            NOAGuess::new().guess(&UserStyle::Main, GuessNOACfg::new(main_args, 0, noa_len)),
        )? {
            Self::ig_failure(process_non_opt::<S>(
                &main_ctx, set, ser, &mut proc, &mut is,
            ))?;
        }

        Self::ig_failure(ser.ser_check()?.post_check(set))?;
        ser.register(is);

        Ok(Some(ret))
    }
}
