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
use super::ReturnVal;
use super::UserStyle;
use crate::args::ArgParser;
use crate::args::Args;
use crate::astr;
use crate::ctx::Ctx;
use crate::ext::ServicesExt;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::proc::Process;
use crate::ser::Services;
use crate::set::Ctor;
use crate::set::Pre;
use crate::set::Set;
use crate::Arc;
use crate::Error;

/// [`DelayPolicy`] matching the command line arguments with [`Opt`] in the [`Set`].
/// The option will match failed if any special [`Error`] raised during option processing.
/// [`DelayPolicy`] will return `Some(true)` if match successful.
/// [`DelayPolicy`] process the option first, but not invoke the handler of option.
/// The handler will be called after [`Cmd`](crate::opt::Style::Cmd) NOA and [`Pos`](crate::opt::Style::Pos) NOA processed.
/// In last, [`DelayPolicy`] will process [`Main`](crate::opt::Style::Main) NOA.
///
/// # Example
/// ```rust
/// # use aopt::getopt;
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// # use aopt::RawVal;
/// # use std::path::PathBuf;
/// #
/// # fn main() -> Result<(), Error> {
/// fn path_storer(
///     uid: Uid,
///     set: &mut ASet,
///     ser: &mut ASer,
///     raw: Option<&RawVal>,
///     vals: Option<Vec<PathBuf>>,
/// ) -> Result<Option<()>, Error> {
///     if let Some(vals) = vals {
///         let mut action = set[uid].action().clone();
///
///         for val in vals {
///             action.process(uid, set, ser, raw, Some(val))?;
///         }
///         Ok(Some(()))
///     } else {
///         Ok(None)
///     }
/// }
///
/// let filter = |f: fn(&PathBuf) -> bool| {
///     move |set: &mut ASet, ser: &mut ASer| {
///         let uid = set["directory"].uid();
///
///         PathBuf::sve_filter(uid, ser, f)?;
///         Ok(Some(true))
///     }
/// };
///
/// let mut parser = ADelayParser::default();
///
/// // POS will be process first, get the items under given directory
/// parser
///     .add_opt("directory=p@1")?
///     .set_values(Vec::<PathBuf>::new())
///     .on(|_: &mut ASet, _: &mut ASer, path: ctx::Value<PathBuf>| {
///         Ok(Some(
///             path.read_dir()
///                 .map_err(|e| {
///                     Error::raise_failure(format!("Can not read directory {:?}: {:?}", path, e))
///                 })?
///                 .map(|v| v.unwrap().path())
///                 .collect::<Vec<PathBuf>>(),
///         ))
///     })?
///     .then(path_storer);
///
/// // filter the item if any option set
/// parser
///     .add_opt("--file=b")?
///     .add_alias("-f")
///     .on(filter(|path: &PathBuf| !path.is_file()))?;
/// parser
///     .add_opt("--dir=b")?
///     .add_alias("-d")
///     .on(filter(|path: &PathBuf| !path.is_dir()))?;
/// parser
///     .add_opt("--sym-link=b")?
///     .add_alias("-s")
///     .on(filter(|path: &PathBuf| !path.is_symlink()))?;
///
/// // Main will be process latest, display the items
/// parser
///     .add_opt("main=m")?
///     .on(move |set: &mut ASet, ser: &mut ASer| {
///         if let Ok(vals) = PathBuf::sve_vals(set["directory"].uid(), ser) {
///             for val in vals {
///                 println!("{:?}", val);
///             }
///         }
///         Ok(Some(true))
///     })?;
///
/// getopt!(std::env::args().skip(1), &mut parser)?;
/// #
/// # Ok(())
/// # }
/// ```
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
    <S::Ctor as Ctor>::Opt: Opt,
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

    pub fn invoke_opt_callback(&mut self, set: &mut S, ser: &mut Services) -> Result<(), Error> {
        for saver in std::mem::take(&mut self.contexts) {
            invoke_callback_opt(saver, set, ser)?;
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
    <S::Ctor as Ctor>::Opt: Opt,
    S: Set + OptParser + Pre + Debug + 'static,
{
    type Ret = ReturnVal;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        ser: &mut Services,
        args: Arc<Args>,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        for opt in set.iter_mut() {
            opt.init(ser)?;
        }
        ser.ser_check()?.pre_check(set)?;

        // take the invoke service, avoid borrow the ser
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
            let arg = arg.map(|v| Arc::new(v.clone()));

            // parsing current argument
            if let Ok(clopt) = opt.parse_arg(set.prefix()) {
                for style in opt_styles.iter() {
                    if let Some(mut proc) = OptGuess::new()
                        .guess(style, GuessOptCfg::new(idx, args_len, arg.clone(), &clopt))?
                    {
                        opt_ctx.set_idx(idx);
                        let ret = process_opt::<S>(&opt_ctx, set, ser, &mut proc, false)?;

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

        let ret = noa_args.clone();
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
                process_non_opt::<S>(&noa_ctx, set, ser, &mut proc)?;
            }

            ser.ser_check()?.cmd_check(set)?;

            for idx in 0..noa_len {
                if let Some(mut proc) = NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), Self::noa_idx(idx), noa_len),
                )? {
                    noa_ctx.set_idx(Self::noa_idx(idx));
                    process_non_opt::<S>(&noa_ctx, set, ser, &mut proc)?;
                }
            }
        } else {
            ser.ser_check()?.cmd_check(set)?;
        }

        // after cmd and pos callback invoked, invoke the callback of option
        self.invoke_opt_callback(set, ser)?;

        ser.ser_check()?.opt_check(set)?;

        ser.ser_check()?.pos_check(set)?;

        let main_args = noa_args;
        let mut main_ctx = noa_ctx;

        main_ctx.set_idx(0);
        if let Some(mut proc) =
            NOAGuess::new().guess(&UserStyle::Main, GuessNOACfg::new(main_args, 0, noa_len))?
        {
            process_non_opt::<S>(&main_ctx, set, ser, &mut proc)?;
        }

        ser.ser_check()?.post_check(set)?;

        Ok(Some(ReturnVal::new(ret.into_inner(), true)))
    }
}

#[cfg(test)]
mod test {

    use std::ops::Deref;

    use crate::prelude::*;
    use crate::Arc;
    use crate::Error;

    #[test]
    fn testing_1() {
        assert!(testing_1_main().is_ok());
    }

    fn testing_1_main() -> Result<(), Error> {
        fn check_opt_val<T: std::fmt::Debug + PartialEq + 'static>(
            ser: &mut ASer,
            opt: &AOpt,
            uid: Uid,
            name: &str,
            prefix: Option<&str>,
            vals: Option<Vec<T>>,
            optional: bool,
            action: &Action,
            assoc: &Assoc,
            index: Option<&Index>,
            alias: Option<Vec<(&str, &str)>>,
            deactivate: bool,
        ) -> Result<(), Error> {
            let opt_uid = opt.uid();

            assert_eq!(opt_uid, uid);
            assert_eq!(opt.name(), name, "name not equal -{}-", opt_uid);
            assert_eq!(opt.prefix().map(|v| v.as_str()), prefix);
            assert_eq!(
                opt.optional(),
                optional,
                "optional not equal -{}-: {}",
                opt_uid,
                optional
            );
            assert_eq!(opt.action(), action, "action not equal for {}", opt_uid);
            assert_eq!(opt.assoc(), assoc, "assoc not equal for {}", opt_uid);
            assert_eq!(opt.idx(), index, "option index not equal: {:?}", index);
            assert_eq!(
                opt.is_deactivate(),
                deactivate,
                "deactivate style not matched!"
            );
            if let Ok(opt_vals) = T::sve_vals(opt_uid, ser) {
                if let Some(vals) = vals {
                    assert_eq!(
                        opt_vals.len(),
                        vals.len(),
                        "value length not equal for {}",
                        opt_uid
                    );
                    for (l, r) in opt_vals.iter().zip(vals.iter()) {
                        assert_eq!(
                            l, r,
                            "option value not equal -{}- : {:?} != {:?}",
                            opt_uid, l, r
                        );
                    }
                }
            } else {
                assert!(
                    vals.is_none(),
                    "found none, option value not equal: {:?}",
                    vals
                );
            }
            if let Some(opt_alias) = opt.alias() {
                if let Some(alias) = alias {
                    assert_eq!(opt_alias.len(), alias.len());
                    for (prefix, name) in alias {
                        assert!(
                            opt_alias.iter().any(|(p, n)| p == prefix && n == name),
                            "alias => {:?} <--> {}, {}",
                            &opt_alias,
                            prefix,
                            name,
                        );
                    }
                }
            } else {
                assert!(alias.is_none());
            }
            Ok(())
        }

        let mut policy = ADelayPolicy::default();
        let mut ser = policy.default_ser();
        let mut set = policy.default_set();

        let args = Args::new(
            [
                "filter",
                "+>",
                "foo",
                "bar",
                "8",
                "42",
                "--option-ignored",
                "88",
                "+>",
                "12.5",
                "lily",
                "66",
                "11",
            ]
            .into_iter(),
        );

        set.add_prefix("+");
        set.add_opt("set=c")?;
        set.add_opt("filter=c")?;
        let args_uid = set.add_opt("args=p@2..")?.set_assoc(Assoc::Flt).run()?;

        ser.ser_invoke_mut()?
            .entry(set.add_opt("--positive=b")?.add_alias("+>").run()?)
            .on(|set: &mut ASet, ser: &mut ASer| {
                f64::sve_filter(set["args=p"].uid(), ser, |v: &f64| v <= &0.0)?;
                Ok(Some(true))
            });
        ser.ser_invoke_mut()?
            .entry(set.add_opt("--bigger-than=f")?.add_alias("+>").run()?)
            .on(|set: &mut ASet, ser: &mut ASer, val: ctx::Value<f64>| {
                // this is a vec![vec![], ..]
                Ok(Some(f64::sve_filter(
                    set["args=p"].uid(),
                    ser,
                    |v: &f64| v <= val.deref(),
                )?))
            });
        ser.ser_invoke_mut()?
            .entry(set.add_opt("main=m")?.run()?)
            .on(move |set: &mut ASet, ser: &mut ASer| {
                let args = &set["args"];
                let bopt = &set["--bigger-than"];

                check_opt_val::<f64>(
                    ser,
                    args,
                    args_uid,
                    "args",
                    None,
                    Some(vec![42.0, 88.0, 66.0]),
                    true,
                    &Action::App,
                    &Assoc::Flt,
                    Some(&Index::Range(2, 0)),
                    None,
                    false,
                )?;
                check_opt_val::<Vec<f64>>(
                    ser,
                    bopt,
                    bopt.uid(),
                    "bigger-than",
                    Some("--"),
                    Some(vec![vec![8.0, 11.0]]),
                    true,
                    &Action::App,
                    &Assoc::Flt,
                    None,
                    None,
                    false,
                )?;
                Ok(Some(()))
            });

        let args = Arc::new(args);

        assert!(policy.parse(&mut set, &mut ser, args.clone()).is_err());
        policy.set_strict(false);
        policy.parse(&mut set, &mut ser, args)?;
        Ok(())
    }
}
