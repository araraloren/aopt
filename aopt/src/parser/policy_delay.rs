use std::fmt::Debug;
use std::marker::PhantomData;

use super::invoke_callback_opt;
use super::process::ProcessCtx;
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
use super::SetChecker;
use super::UserStyle;
use crate::args::ArgParser;
use crate::args::Args;
use crate::astr;
use crate::ctx::Ctx;
use crate::ctx::Invoker;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::proc::Process;
use crate::ser::ServicesExt;
use crate::set::OptValidator;
use crate::set::SetOpt;
use crate::Arc;
use crate::Error;

/// [`DelayPolicy`] matching the command line arguments with [`Opt`] in the [`Set`](crate::set::Set).
/// The option will match failed if any special [`Error`] raised during option processing.
/// [`DelayPolicy`] will return Some([`ReturnVal`]) if match successful.
/// [`DelayPolicy`] process the option first, but not invoke the handler of option.
/// The handler will be called after [`Cmd`](crate::opt::Style::Cmd) NOA and [`Pos`](crate::opt::Style::Pos) NOA processed.
/// In last, [`DelayPolicy`] will process [`Main`](crate::opt::Style::Main) NOA.
/// During parsing, you can't get the value of any option in the handler of NOA.
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
///         ser.sve_filter::<PathBuf>(uid, f)?;
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
///         if let Ok(vals) = ser.sve_vals::<PathBuf>(set["directory"].uid()) {
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
pub struct DelayPolicy<Set, Ser> {
    strict: bool,

    contexts: Vec<CtxSaver>,

    checker: SetChecker<Set>,

    styles: Vec<UserStyle>,

    marker_s: PhantomData<(Set, Ser)>,
}

impl<Set, Ser> Default for DelayPolicy<Set, Ser> {
    fn default() -> Self {
        Self {
            strict: true,
            contexts: vec![],
            checker: SetChecker::default(),
            styles: vec![
                UserStyle::EqualWithValue,
                UserStyle::Argument,
                UserStyle::Boolean,
                UserStyle::CombinedOption,
                UserStyle::EmbeddedValue,
            ],
            marker_s: PhantomData::default(),
        }
    }
}

impl<Set, Ser> DelayPolicy<Set, Ser>
where
    SetOpt<Set>: Opt,
    Ser: ServicesExt + 'static,
    Set: crate::set::Set + OptParser + Debug + 'static,
{
    pub fn new(strict: bool, styles: Vec<UserStyle>) -> Self {
        Self {
            strict,
            styles,
            ..Self::default()
        }
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

    pub fn with_styles(mut self, styles: Vec<UserStyle>) -> Self {
        self.styles = styles;
        self
    }

    pub fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.styles = styles;
        self
    }

    pub fn user_styles(&self) -> &[UserStyle] {
        &self.styles
    }

    pub fn strict(&self) -> bool {
        self.strict
    }

    pub fn checker(&self) -> &SetChecker<Set> {
        &self.checker
    }

    pub fn invoke_opt_callback(
        &mut self,
        ctx: &mut Ctx,
        set: &mut Set,
        inv: &mut Invoker<Set, Ser>,
        ser: &mut Ser,
    ) -> Result<(), Error> {
        for saver in std::mem::take(&mut self.contexts) {
            let uid = saver.uid;

            ctx.set_inner_ctx(Some(saver.ctx));
            invoke_callback_opt(uid, ctx, set, inv, ser)?;
        }
        Ok(())
    }

    /// Return the NOA index base on 1.
    pub fn noa_idx(idx: usize) -> usize {
        idx + 1
    }
}

impl<Set, Ser> DelayPolicy<Set, Ser>
where
    SetOpt<Set>: Opt,
    Ser: ServicesExt + 'static,
    Set: crate::set::Set + OptParser + OptValidator + Debug + 'static,
{
    pub(crate) fn parse_impl(
        &mut self,
        ctx: &mut Ctx,
        set: &mut <Self as Policy>::Set,
        inv: &mut <Self as Policy>::Inv,
        ser: &mut <Self as Policy>::Ser,
    ) -> Result<(), <Self as Policy>::Error> {
        self.checker().pre_check(set)?;

        // take the invoke service, avoid borrow the ser
        let opt_styles = &self.styles;
        let args = ctx.orig_args().clone();
        let args_len = args.len();
        let mut noa_args = Args::default();
        let mut iter = args.guess_iter().enumerate();

        // set option args, and args length
        ctx.set_args(args.clone());
        while let Some((idx, (opt, arg))) = iter.next() {
            let mut matched = false;
            let mut consume = false;
            let arg = arg.map(|v| Arc::new(v.clone()));

            // parsing current argument
            if let Ok(clopt) = opt.parse_arg() {
                if let Some(name) = clopt.name() {
                    if set.check_name(name.as_str())? {
                        for style in opt_styles.iter() {
                            if let Some(mut proc) = OptGuess::new().guess(
                                style,
                                GuessOptCfg::new(idx, args_len, arg.clone(), &clopt),
                            )? {
                                let ret = process_opt(
                                    ProcessCtx {
                                        idx,
                                        ctx,
                                        set,
                                        inv,
                                        ser,
                                        tot: args_len,
                                    },
                                    &mut proc,
                                    false,
                                )?;

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
                                "{}",
                                clopt.name().unwrap_or(&default_str)
                            )));
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

        let noa_args = Arc::new(noa_args);
        let noa_len = noa_args.len();

        ctx.set_args(noa_args.clone());
        // when style is pos, noa index is [1..=len]
        if noa_args.len() > 0 {
            if let Some(mut proc) = NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(noa_args.clone(), Self::noa_idx(0), noa_len),
            )? {
                process_non_opt(
                    ProcessCtx {
                        ctx,
                        set,
                        inv,
                        ser,
                        tot: noa_len,
                        idx: Self::noa_idx(0),
                    },
                    &mut proc,
                )?;
            }

            self.checker().cmd_check(set)?;

            for idx in 0..noa_len {
                if let Some(mut proc) = NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), Self::noa_idx(idx), noa_len),
                )? {
                    process_non_opt(
                        ProcessCtx {
                            ctx,
                            set,
                            inv,
                            ser,
                            tot: noa_len,
                            idx: Self::noa_idx(idx),
                        },
                        &mut proc,
                    )?;
                }
            }
        } else {
            self.checker().cmd_check(set)?;
        }

        // after cmd and pos callback invoked, invoke the callback of option
        self.invoke_opt_callback(ctx, set, inv, ser)?;

        self.checker().opt_check(set)?;

        self.checker().pos_check(set)?;

        let main_args = noa_args;
        let main_len = main_args.len();

        ctx.set_args(main_args.clone());
        if let Some(mut proc) =
            NOAGuess::new().guess(&UserStyle::Main, GuessNOACfg::new(main_args, 0, main_len))?
        {
            process_non_opt(
                ProcessCtx {
                    ctx,
                    set,
                    inv,
                    ser,
                    tot: main_len,
                    idx: 0,
                },
                &mut proc,
            )?;
        }

        self.checker().post_check(set)?;
        Ok(())
    }
}

impl<Set, Ser> Policy for DelayPolicy<Set, Ser>
where
    SetOpt<Set>: Opt,
    Ser: ServicesExt + 'static,
    Set: crate::set::Set + OptParser + OptValidator + Debug + 'static,
{
    type Ret = ReturnVal;

    type Set = Set;

    type Inv = Invoker<Set, Ser>;

    type Ser = Ser;

    type Error = Error;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv,
        ser: &mut Self::Ser,
        args: Arc<Args>,
    ) -> Result<Self::Ret, Self::Error> {
        let mut ctx = Ctx::default().with_orig_args(args.clone()).with_args(args);

        match self.parse_impl(&mut ctx, set, inv, ser) {
            Ok(_) => Ok(ReturnVal::new(ctx, true)),
            Err(e) => {
                if e.is_failure() {
                    Ok(ReturnVal::new(ctx, false))
                } else {
                    Err(e)
                }
            }
        }
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
        fn check_opt_val<T: std::fmt::Debug + PartialEq + ErasedTy + 'static>(
            ser: &mut ASer,
            opt: &AOpt,
            uid: Uid,
            name: &str,
            vals: Option<Vec<T>>,
            force: bool,
            action: &Action,
            assoc: &Assoc,
            index: Option<&Index>,
            alias: Option<Vec<&str>>,
        ) -> Result<(), Error> {
            let opt_uid = opt.uid();

            assert_eq!(opt_uid, uid);
            assert_eq!(opt.name(), name, "name not equal -{}-", opt_uid);
            assert_eq!(
                opt.force(),
                force,
                "option force required not equal -{}-: {}",
                opt_uid,
                force
            );
            assert_eq!(opt.action(), action, "action not equal for {}", opt_uid);
            assert_eq!(opt.assoc(), assoc, "assoc not equal for {}", opt_uid);
            assert_eq!(opt.idx(), index, "option index not equal: {:?}", index);
            if let Ok(opt_vals) = ser.sve_vals::<T>(opt_uid) {
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
                    for name in alias {
                        assert!(
                            opt_alias.iter().any(|n| n == name),
                            "alias => {:?} <--> {}",
                            &opt_alias,
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
        let mut inv = policy.default_inv();
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

        set.validator_mut().add_prefix("+");

        set.add_opt("set=c")?;
        set.add_opt("filter=c")?;
        let args_uid = set.add_opt("args=p@2..")?.set_assoc(Assoc::Flt).run()?;

        inv.entry(set.add_opt("--positive=b")?.add_alias("+>").run()?)
            .on(|set: &mut ASet, ser: &mut ASer| {
                ser.sve_filter::<f64>(set["args=p"].uid(), |v: &f64| v <= &0.0)?;
                Ok(Some(true))
            });
        inv.entry(set.add_opt("--bigger-than=f")?.add_alias("+>").run()?)
            .on(|set: &mut ASet, ser: &mut ASer, val: ctx::Value<f64>| {
                // this is a vec![vec![], ..]
                Ok(Some(
                    ser.sve_filter::<f64>(set["args=p"].uid(), |v: &f64| v <= val.deref())?,
                ))
            });
        inv.entry(set.add_opt("main=m")?.run()?)
            .on(move |set: &mut ASet, ser: &mut ASer| {
                let args = &set["args"];
                let bopt = &set["--bigger-than"];

                check_opt_val::<f64>(
                    ser,
                    args,
                    args_uid,
                    "args",
                    Some(vec![42.0, 88.0, 66.0]),
                    false,
                    &Action::App,
                    &Assoc::Flt,
                    Some(&Index::Range(2, 0)),
                    None,
                )?;
                check_opt_val::<Vec<f64>>(
                    ser,
                    bopt,
                    bopt.uid(),
                    "--bigger-than",
                    Some(vec![vec![8.0, 11.0]]),
                    false,
                    &Action::App,
                    &Assoc::Flt,
                    None,
                    None,
                )?;
                Ok(Some(()))
            });

        let args = Arc::new(args);

        for opt in set.iter_mut() {
            opt.init(&mut ser)?;
        }
        assert!(!policy
            .parse(&mut set, &mut inv, &mut ser, args.clone())?
            .status());
        policy.set_strict(false);
        assert!(policy.parse(&mut set, &mut inv, &mut ser, args)?.status());
        Ok(())
    }
}
