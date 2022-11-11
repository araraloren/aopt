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
use crate::astr;
use crate::ctx::Ctx;
use crate::ext::ServicesExt;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::prelude::SetExt;
use crate::proc::Process;
use crate::ser::InvokeService;
use crate::ser::Services;
use crate::set::Pre;
use crate::set::Set;
use crate::Arc;
use crate::Error;

/// Forward process the option before any
/// NOA([`Cmd`](crate::opt::OptStyle::Cmd), [`Pos`](crate::opt::OptStyle::Pos) and [`Main`](crate::opt::OptStyle::Main)).
///
/// You can get the value of any option in the handler of NOA.
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// ser.ser_usrval_mut()?
///     .insert(ser::Value::new(vec!["foo", "bar"]));
///
/// let filter_id = set.add_opt("--filter=b/")?.run()?;
/// let pos_id = set
///     .add_opt("pos=p@*")?
///     .set_initiator(ValInitiator::empty::<String>())
///     .run()?;
/// ser.ser_invoke_mut::<ASet>()?
///     .register_ser(
///         pos_id,
///         move |_: Uid,
///                 _: &mut ASet,
///                 ser: &mut Services,
///                 filter: ser::Value<Vec<&str>>,
///                 mut value: ctx::Value<String>| {
///             let do_filter = bool::val(filter_id, ser)?;
///             let valid = if *do_filter {
///                 !filter.iter().any(|&v| v == value.as_str())
///             } else {
///                 true
///             };
///
///             Ok(valid.then(|| value.take()))
///         },
///     )
///     .with_default();
///
/// let args = Args::new(["set", "42", "foo", "bar"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// let values = String::vals(pos_id, &ser)?;
///
/// assert_eq!(values[0], "set");
/// assert_eq!(values[1], "42");
///
/// let args = Args::new(["--/filter", "set", "42", "foo", "bar"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// let values = String::vals(pos_id, &ser)?;
///
/// assert_eq!(values[0], "set");
/// assert_eq!(values[1], "42");
/// assert_eq!(values[2], "foo");
/// assert_eq!(values[3], "bar");
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Forward<S> {
    strict: bool,

    marker_s: PhantomData<S>,
}

impl<S> Default for Forward<S> {
    fn default() -> Self {
        Self {
            strict: true,
            marker_s: PhantomData::default(),
        }
    }
}

impl<S> Forward<S>
where
    S::Opt: Opt,
    S: Set + OptParser + Debug + 'static,
{
    pub fn new(strict: bool) -> Self {
        Self {
            strict,
            ..Default::default()
        }
    }

    /// In strict mode, if an argument looks like an option (it matched any option prefix),
    /// then it must matched.
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

impl<S> Policy for Forward<S>
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
        let stys = [
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

            if let Ok(clopt) = opt.parse(set.prefix()) {
                for style in stys.iter() {
                    if let Some(mut proc) = OptGuess::new()
                        .guess(style, GuessOptCfg::new(idx, args_len, arg.clone(), &clopt))?
                    {
                        opt_ctx.set_idx(idx);
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

        ser.ser_check()?.opt_check(set)?;

        let noa_args = Arc::new(noa_args);
        let noa_len = noa_args.len();
        let mut noa_ctx = Ctx::default();

        noa_ctx.set_args(noa_args.clone()).set_total(noa_args.len());

        // when style is pos, noa index is [1..=len]
        if noa_args.len() > 0 {
            if let Some(mut proc) = NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(noa_args.clone(), 1, noa_len),
            )? {
                noa_ctx.set_idx(1);
                process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is)?;
            }

            ser.ser_check()?.cmd_check(set)?;

            for idx in 0..noa_len {
                if let Some(mut proc) = NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), idx + 1, noa_len),
                )? {
                    noa_ctx.set_idx(idx + 1);
                    process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is)?;
                }
            }
        } else {
            ser.ser_check()?.cmd_check(set)?;
        }
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
