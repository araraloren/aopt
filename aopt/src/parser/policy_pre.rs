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
use crate::set::Ctor;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::proc::Process;
use crate::ser::Services;
use crate::set::Pre;
use crate::set::Set;
use crate::Arc;
use crate::Error;

/// [`PrePolicy`] matching the command line arguments with [`Opt`] in the [`Set`].
/// [`PrePolicy`] will skip any special [`Error`] during [`parse`](Policy::parse) process.
/// [`PrePolicy`] will return the left `NOA`s after parsing.
/// [`PrePolicy`] don't consume the `NOA` when process [`NOAMatch`](crate::proc::NOAMatch).
#[derive(Debug, Clone)]
pub struct PrePolicy<S> {
    strict: bool,

    marker_s: PhantomData<S>,
}

impl<S> Default for PrePolicy<S> {
    fn default() -> Self {
        Self {
            strict: false,

            marker_s: PhantomData::default(),
        }
    }
}

impl<S> PrePolicy<S> {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    /// In strict mode, if an argument looks like an option (it matched any option prefix),
    /// then it must matched, otherwise it will be discarded.
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
    <S::Ctor as Ctor>::Opt: Opt,
    S: Set + OptParser + Pre + Debug + 'static,
{
    type Ret = Args;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        ser: &mut Services,
        args: Arc<Args>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        for opt in set.iter_mut() {
            Self::ig_failure(opt.init(ser))?;
        }
        Self::ig_failure(ser.ser_check()?.pre_check(set))?;

        let opt_styles = [
            UserStyle::EqualWithValue,
            UserStyle::Argument,
            UserStyle::Boolean,
            UserStyle::CombinedOption,
            UserStyle::EmbeddedValue,
        ];
        let args_len = args.len();
        let mut noa_args = Args::default();
        let mut iter = args.guess_iter().enumerate();
        let mut opt_ctx = Ctx::default();

        opt_ctx.set_args(args.clone()).set_total(args_len);

        while let Some((idx, (opt, arg))) = iter.next() {
            let mut matched = false;
            let mut consume = false;
            let mut like_opt = false;
            let arg = arg.map(|v| Arc::new(v.clone()));

            if let Ok(clopt) = opt.parse_arg(set.prefix()) {
                like_opt = true;
                for style in opt_styles.iter() {
                    let ret = Self::ig_failure(
                        OptGuess::new()
                            .guess(style, GuessOptCfg::new(idx, args_len, arg.clone(), &clopt)),
                    )?;

                    if let Some(Some(mut proc)) = ret {
                        opt_ctx.set_idx(idx);
                        if Self::ig_failure(process_opt::<S>(&opt_ctx, set, ser, &mut proc, true))?
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
            } else if !matched && !self.get_strict() || !like_opt {
                // add it to NOA if current argument not matched
                // and not in strict mode or the argument not like an option
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
                Self::ig_failure(process_non_opt::<S>(&noa_ctx, set, ser, &mut proc))?;
            }

            Self::ig_failure(ser.ser_check()?.cmd_check(set))?;

            for idx in 0..noa_len {
                if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), Self::noa_idx(idx), noa_len),
                ))? {
                    noa_ctx.set_idx(Self::noa_idx(idx));
                    Self::ig_failure(process_non_opt::<S>(&noa_ctx, set, ser, &mut proc))?;
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
            Self::ig_failure(process_non_opt::<S>(&main_ctx, set, ser, &mut proc))?;
        }

        Self::ig_failure(ser.ser_check()?.post_check(set))?;

        Ok(Some(ret))
    }
}
