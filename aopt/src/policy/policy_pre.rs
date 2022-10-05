use std::fmt::Debug;
use std::marker::PhantomData;

use super::process_non_opt;
use super::process_opt;
use super::APolicyExt;
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
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::proc::Process;
use crate::ser::AServiceExt;
use crate::ser::CheckService;
use crate::ser::InvokeService;
use crate::ser::Services;
use crate::ser::ServicesExt;
use crate::set::ASetExt;
use crate::set::Prefixed;
use crate::set::Set;
use crate::Error;
use crate::Str;

/// [`PrePolicy`] matching the command line arguments with [`Opt`] in the [`Set`].
/// [`PrePolicy`] will skip any special [`Error`] during [`parse`](Policy::parse) process.
/// [`PrePolicy`] will return the left `NOA`s after parsing.
/// [`PrePolicy`] don't consume the `NOA` when process [`NOAMatch`](crate::proc::NOAMatch).
#[derive(Debug, Clone)]
pub struct PrePolicy<S, V>
where
    V: From<Str>,
    S: Set + OptParser,
{
    marker_s: PhantomData<(S, V)>,
}

impl<S, V> Default for PrePolicy<S, V>
where
    V: From<Str>,
    S: Set + OptParser,
{
    fn default() -> Self {
        Self {
            marker_s: PhantomData::default(),
        }
    }
}

impl<S, V> APolicyExt<S, V> for PrePolicy<S, V>
where
    V: From<Str> + 'static,
    S::Opt: Opt,
    S: Set + OptParser + Debug + 'static,
{
    fn new_set<T>() -> T
    where
        T: ASetExt + Set + OptParser + Debug + 'static,
    {
        T::new_default()
    }

    fn new_services<T>() -> T
    where
        T: AServiceExt<S, V>,
    {
        T::new_default()
    }
}

impl<S, V> PrePolicy<S, V>
where
    V: From<Str> + 'static,
    S::Opt: Opt,
    S: Set + OptParser + Debug + 'static,
{
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
    S: Set + OptParser + Prefixed + Debug + 'static,
{
    type Ret = V;

    type Value = Vec<Str>;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        args: Args,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Value>, Self::Error> {
        Self::ig_failure(ser.get_service::<CheckService<S, V>>()?.pre_check(set))?;

        // take the invoke service, avoid borrow the ser
        let mut inv_ser = ser.take_service::<InvokeService<S, V>>()?;
        let opt_styles = [
            UserStyle::EqualWithValue,
            UserStyle::Argument,
            UserStyle::Boolean,
            UserStyle::CombinedOption,
            UserStyle::EmbeddedValue,
        ];
        let main_args = args.clone();
        let mut args = args;
        let mut left_args = vec![];
        let mut parser = CLOptParser::default();

        while !args.is_last() {
            let mut matched = false;
            let mut consume = false;

            if let Ok(clopt) = args.parse(&mut parser, set.get_prefix()) {
                for style in opt_styles.iter() {
                    let ret = Self::ig_failure(
                        OptGuess::new().guess(style, GuessOptCfg::new(&args, clopt.clone())),
                    )?;

                    if ret.is_none() {
                        continue;
                    }
                    if let Some(Some(mut proc)) = ret {
                        if Self::ig_failure(process_opt::<S, V>(
                            &args,
                            set,
                            ser,
                            &mut proc,
                            &mut inv_ser,
                            true,
                        ))?
                        .is_none()
                        {
                            continue;
                        }
                        if proc.is_matched() {
                            matched = true;
                        }
                        if proc.is_consume_argument() {
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
                args.skip();
            } else if !matched {
                // add it to NOA if current argument not matched
                if let Some(arg) = args.get_curr() {
                    left_args.push(arg.clone());
                }
            }

            // skip current argument
            args.skip();
        }

        Self::ig_failure(ser.get_service::<CheckService<S, V>>()?.opt_check(set))?;

        // save the return value
        let pre_ret = left_args.clone();
        let noa_total = left_args.len();

        if noa_total > 0 {
            let mut args = Args::from(left_args);

            if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(&args, args.get(0).cloned(), Some(1)),
            ))? {
                Self::ig_failure(process_non_opt::<S, V>(
                    &args,
                    set,
                    ser,
                    &mut proc,
                    &mut inv_ser,
                ))?;
            }

            Self::ig_failure(ser.get_service::<CheckService<S, V>>()?.cmd_check(set))?;

            while !args.is_last() {
                if let Some(Some(mut proc)) = Self::ig_failure(
                    NOAGuess::new().guess(&UserStyle::Pos, GuessNOACfg::new(&args, None, None)),
                )? {
                    Self::ig_failure(process_non_opt::<S, V>(
                        &args,
                        set,
                        ser,
                        &mut proc,
                        &mut inv_ser,
                    ))?;
                }
                args.skip();
            }
        } else {
            Self::ig_failure(ser.get_service::<CheckService<S, V>>()?.cmd_check(set))?;
        }

        Self::ig_failure(ser.get_service::<CheckService<S, V>>()?.pos_check(set))?;

        if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
            &UserStyle::Main,
            GuessNOACfg::new(&main_args, Some(astr("Main")), Some(0)),
        ))? {
            Self::ig_failure(process_non_opt::<S, V>(
                &main_args,
                set,
                ser,
                &mut proc,
                &mut inv_ser,
            ))?;
        }

        ser.get_service::<CheckService<S, V>>()?.post_check(set)?;
        ser.register(inv_ser);

        Ok(Some(pre_ret))
    }
}
