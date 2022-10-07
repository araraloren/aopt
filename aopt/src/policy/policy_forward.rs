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
use crate::aext::APolicyExt;
use crate::aext::AServiceExt;
use crate::aext::ASetExt;
use crate::arg::Args;
use crate::arg::CLOptParser;
use crate::astr;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::proc::Process;
use crate::ser::CheckService;
use crate::ser::InvokeService;
use crate::ser::Services;
use crate::ser::ServicesExt;
use crate::set::Prefixed;
use crate::set::Set;
use crate::Error;
use crate::Str;

#[derive(Debug, Clone)]
pub struct ForwardPolicy<S, V> {
    strict: bool,

    marker_s: PhantomData<(S, V)>,
}

impl<S, V> Default for ForwardPolicy<S, V> {
    fn default() -> Self {
        Self {
            strict: true,
            marker_s: PhantomData::default(),
        }
    }
}

impl<S: 'static, V: 'static> APolicyExt<S, V> for ForwardPolicy<S, V> {
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

impl<S, V> ForwardPolicy<S, V>
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

    pub fn get_strict(&self) -> bool {
        self.strict
    }
}

impl<S, V> Policy for ForwardPolicy<S, V>
where
    V: From<Str> + 'static,
    S::Opt: Opt,
    S: Set + OptParser + Prefixed + Debug + 'static,
{
    type Ret = V;

    type Value = bool;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        args: Args,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Value>, Self::Error> {
        ser.get_service::<CheckService<S, V>>()?.pre_check(set)?;

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
                    if let Some(mut proc) =
                        OptGuess::new().guess(style, GuessOptCfg::new(&args, clopt.clone()))?
                    {
                        process_opt::<S, V>(&args, set, ser, &mut proc, &mut inv_ser, true)?;
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
                if !matched && self.get_strict() {
                    let name = clopt.get_name();
                    return Err(Error::sp_invalid_option_name(name.unwrap_or_default()));
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

        ser.get_service::<CheckService<S, V>>()?.opt_check(set)?;

        let noa_total = left_args.len();

        if noa_total > 0 {
            let mut args = Args::from(left_args);

            if let Some(mut proc) = NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(&args, args.get(0).cloned(), Some(1)),
            )? {
                process_non_opt::<S, V>(&args, set, ser, &mut proc, &mut inv_ser)?;
            }

            ser.get_service::<CheckService<S, V>>()?.cmd_check(set)?;

            while !args.is_last() {
                if let Some(mut proc) =
                    NOAGuess::new().guess(&UserStyle::Pos, GuessNOACfg::new(&args, None, None))?
                {
                    process_non_opt::<S, V>(&args, set, ser, &mut proc, &mut inv_ser)?;
                }
                args.skip();
            }
        } else {
            ser.get_service::<CheckService<S, V>>()?.cmd_check(set)?;
        }
        ser.get_service::<CheckService<S, V>>()?.pos_check(set)?;

        if let Some(mut proc) = NOAGuess::new().guess(
            &UserStyle::Main,
            GuessNOACfg::new(&main_args, Some(astr("Main")), Some(0)),
        )? {
            process_non_opt::<S, V>(&main_args, set, ser, &mut proc, &mut inv_ser)?;
        }

        ser.get_service::<CheckService<S, V>>()?.post_check(set)?;
        ser.register(inv_ser);

        Ok(Some(true))
    }
}
