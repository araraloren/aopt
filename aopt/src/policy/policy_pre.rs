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

/// [`PrePolicy`] matching the command line arguments with [`Opt`] in the [`Set`].
/// [`PrePolicy`] will skip any special [`Error`] during [`parse`](Policy::parse) process.
/// [`PrePolicy`] will return the left `NOA`s after parsing.
/// [`PrePolicy`] don't consume the `NOA` when process [`NOAMatch`](crate::proc::NOAMatch).
#[derive(Debug, Clone)]
pub struct PrePolicy<S, V> {
    marker_s: PhantomData<(S, V)>,
}

impl<S, V> Default for PrePolicy<S, V> {
    fn default() -> Self {
        Self {
            marker_s: PhantomData::default(),
        }
    }
}

impl<S: 'static, V: 'static> APolicyExt<S, V> for PrePolicy<S, V> {
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

impl<S, V> PrePolicy<S, V> {
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

impl<S, V> Policy for PrePolicy<S, V>
where
    V: From<Str> + 'static,
    S::Opt: Opt,
    S: Set + OptParser + PreSet + Debug + 'static,
{
    type Ret = Vec<Str>;

    type Value = V;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        args: Args,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        Self::ig_failure(ser.ser::<CheckService<S, V>>()?.pre_check(set))?;

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

            if let Ok(clopt) = iter.parse(&mut parser, set.pre()) {
                for style in opt_styles.iter() {
                    let ret = Self::ig_failure(
                        OptGuess::new().guess(style, GuessOptCfg::new(&iter, clopt.clone())),
                    )?;

                    if ret.is_none() {
                        continue;
                    }
                    if let Some(Some(mut proc)) = ret {
                        opt_ctx.set_idx(iter.idx());
                        if Self::ig_failure(process_opt::<S, V>(
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
                if let Some(arg) = iter.cur() {
                    noa_args.push(arg.clone());
                }
            }
        }

        Self::ig_failure(ser.ser::<CheckService<S, V>>()?.opt_check(set))?;

        // save the return value
        let pre_ret = noa_args.deref().clone();

        if noa_args.len() > 0 {
            let mut noa_ctx = Ctx::default()
                .with_args(noa_args.clone())
                .with_len(noa_args.len());

            if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(&noa_args, noa_args[0].clone(), 1),
            ))? {
                noa_ctx.set_idx(1);
                Self::ig_failure(process_non_opt::<S, V>(
                    &noa_ctx, set, ser, &mut proc, &mut is,
                ))?;
            }

            Self::ig_failure(ser.ser::<CheckService<S, V>>()?.cmd_check(set))?;

            for (idx, arg) in noa_args.iter().enumerate() {
                if let Some(Some(mut proc)) = Self::ig_failure(
                    NOAGuess::new()
                        .guess(&UserStyle::Pos, GuessNOACfg::new(&noa_args, arg, idx + 1)),
                )? {
                    noa_ctx.set_idx(idx + 1);
                    Self::ig_failure(process_non_opt::<S, V>(
                        &noa_ctx, set, ser, &mut proc, &mut is,
                    ))?;
                }
            }
        } else {
            Self::ig_failure(ser.ser::<CheckService<S, V>>()?.cmd_check(set))?;
        }

        Self::ig_failure(ser.ser::<CheckService<S, V>>()?.pos_check(set))?;

        let main_args = args;
        let main_ctx = opt_ctx.set_idx(0);

        if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
            &UserStyle::Main,
            GuessNOACfg::new(&main_args, astr("Main"), 0),
        ))? {
            Self::ig_failure(process_non_opt::<S, V>(
                main_ctx, set, ser, &mut proc, &mut is,
            ))?;
        }

        ser.ser::<CheckService<S, V>>()?.post_check(set)?;
        ser.reg(is);

        Ok(Some(pre_ret))
    }
}
