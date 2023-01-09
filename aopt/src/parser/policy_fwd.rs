use std::fmt::Debug;
use std::marker::PhantomData;

use super::process::ProcessCtx;
use super::process_non_opt;
use super::process_opt;
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
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::prelude::Invoker;
use crate::prelude::ServicesExt;
use crate::proc::Process;
use crate::set::OptValidator;
use crate::set::SetOpt;
use crate::Arc;
use crate::Error;

/// [`FwdPolicy`] matching the command line arguments with [`Opt`] in the [`Set`](crate::set::Set).
/// The option will match failed if any special [`Error`] raised during option processing.
/// [`FwdPolicy`] will return `Some(true)` if match successful.
/// [`FwdPolicy`] process the option before any
/// NOA([`Cmd`](crate::opt::Style::Cmd), [`Pos`](crate::opt::Style::Pos) and [`Main`](crate::opt::Style::Main)).
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
/// let mut policy = AFwdPolicy::default();
/// let mut set = policy.default_set();
/// let mut inv = policy.default_inv();
/// let mut ser = policy.default_ser();
///
/// ser.ser_usrval_mut()
///     .insert(ser::Value::new(vec!["foo", "bar"]));
///
/// let filter_id = set.add_opt("--/filter=b")?.run()?;
/// let pos_id = set
///     .add_opt("pos=p@*")?
///     .set_initiator(ValInitiator::empty::<String>())
///     .run()?;
/// inv.entry(pos_id)
///     .on(move |_: &mut ASet,
///                 ser: &mut ASer,
///                 filter: ser::Value<Vec<&str>>,
///                 mut value: ctx::Value<String>| {
///             let not_filter = ser.sve_val::<bool>(filter_id)?;
///             let valid = if !*not_filter {
///                 !filter.iter().any(|&v| v == value.as_str())
///             } else {
///                 true
///             };
///
///             Ok(valid.then(|| value.take()))
///         },
///     );
///
/// let args = Args::new(["set", "42", "foo", "bar"].into_iter());
///
/// for opt in set.iter_mut() {
///     opt.init(&mut ser)?;
/// }
/// policy.parse(&mut set, &mut inv, &mut ser, Arc::new(args))?;
///
/// let values = ser.sve_vals::<String>(pos_id)?;
///
/// assert_eq!(values[0], "set");
/// assert_eq!(values[1], "42");
///
/// let args = Args::new(["--/filter", "set", "42", "foo", "bar"].into_iter());
///
/// for opt in set.iter_mut() {
///     opt.init(&mut ser)?;
/// }
/// policy.parse(&mut set, &mut inv, &mut ser, Arc::new(args))?;
///
/// let values = ser.sve_vals::<String>(pos_id)?;
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
pub struct FwdPolicy<Set, Ser> {
    strict: bool,

    checker: SetChecker<Set>,

    styles: Vec<UserStyle>,

    marker_s: PhantomData<(Set, Ser)>,
}

impl<Set, Ser> Default for FwdPolicy<Set, Ser> {
    fn default() -> Self {
        Self {
            strict: true,
            styles: vec![
                UserStyle::EqualWithValue,
                UserStyle::Argument,
                UserStyle::Boolean,
                UserStyle::CombinedOption,
                UserStyle::EmbeddedValue,
            ],
            checker: SetChecker::default(),
            marker_s: PhantomData::default(),
        }
    }
}

impl<Set, Ser> FwdPolicy<Set, Ser>
where
    SetOpt<Set>: Opt,
    Ser: ServicesExt,
    Set: crate::set::Set + OptParser + Debug + 'static,
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

    pub fn get_strict(&self) -> bool {
        self.strict
    }

    pub fn checker(&self) -> &SetChecker<Set> {
        &self.checker
    }

    /// Return the NOA index base on 1.
    pub fn noa_idx(idx: usize) -> usize {
        idx + 1
    }
}

impl<Set, Ser> FwdPolicy<Set, Ser>
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

        let opt_styles = &self.styles;
        let args = ctx.orig_args().clone();
        let args_len = args.len();
        let mut noa_args = Args::default();
        let mut iter = args.guess_iter().enumerate();
        let mut opt_ctx = Ctx::default();

        ctx.set_args(args.clone());
        while let Some((idx, (opt, arg))) = iter.next() {
            let mut matched = false;
            let mut consume = false;
            let arg = arg.map(|v| Arc::new(v.clone()));

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
                                    true,
                                )?;

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

        self.checker().opt_check(set)?;

        let ret = noa_args.clone();
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
        self.checker().pos_check(set)?;

        let main_args = noa_args;
        let main_len = main_args.len();

        ctx.set_args(main_args.clone());
        if let Some(mut proc) =
            NOAGuess::new().guess(&UserStyle::Main, GuessNOACfg::new(main_args, 0, noa_len))?
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

impl<Set, Ser> Policy for FwdPolicy<Set, Ser>
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
        let ctx = Ctx::default().with_orig_args(args.clone()).with_args(args);

        match self.parse_impl(&mut ctx, set, inv, ser) {
            Ok(ret) => Ok(ReturnVal::new(ctx, true)),
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
    use crate::RawVal;

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

        fn string_collection_validator(vals: Vec<&'static str>) -> ValValidator {
            ValValidator::new(
                move |_: &str, val: Option<&RawVal>, _: (usize, usize)| -> Result<bool, Error> {
                    Ok(val
                        .map(|v| v.get_str())
                        .flatten()
                        .map(|v| vals.contains(&v))
                        .unwrap_or_default())
                },
            )
        }

        fn index_validator(idxs: Vec<usize>) -> ValValidator {
            ValValidator::new(
                move |_: &str, _: Option<&RawVal>, idx: (usize, usize)| -> Result<bool, Error> {
                    Ok(idxs.contains(&idx.0))
                },
            )
        }

        let mut policy = AFwdPolicy::default();
        let mut set = policy.default_set();
        let mut inv = policy.default_inv();
        let mut ser = policy.default_ser();
        let args = Args::new(
            [
                "--copt",
                "--iopt=63",
                "--/dopt",
                "set", // 1
                "--iopt",
                "-42",
                "+eopt",
                "-/fopt",
                "8",       // 2
                "16",      // 3
                "average", // 4
                "--りょう",
                "88",
                "--jopt",
                "2",
                "--iopt-alias1",
                "0",
                "--nopt=8.99",
                "--hopt",
                "48",
                "--qopt=cpp",
                "--alias-k=4",
                "-l2.79",
                "--nopt",
                "3.12",
                "--开关",
                "-olily",
                "program",  // 5
                "software", // 6
                "反转",   //7
                "--值=恍恍惚惚",
                "--qopt",
                "rust",
                "翻转", // 8
            ]
            .into_iter(),
        );

        // add '+' to the prefix validator
        set.validator_mut().add_prefix("+");
        // 5
        set.add_opt("--aopt=b")?;
        set.add_opt("--/bopt=b")?.run()?;
        set.add_opt("--copt=b!")?.set_action(Action::Cnt);
        set.add_opt("--/dopt=b!")?.run()?;
        set.add_opt("--eopt=b")?.add_alias("+eopt").run()?;
        set.add_opt("--/fopt=b")?.add_alias("-/fopt").run()?;

        // 8
        set.add_opt("--gopt=i")?.run()?;
        set.add_opt("--hopt=i!")?.run()?;
        inv.entry(set.add_opt("--iopt=i")?.add_alias("--iopt-alias1").run()?)
            .on(|set: &mut ASet, ser: &mut ASer, val: ctx::Value<i64>| {
                assert_eq!(
                    ser.sve_val::<i64>(set["--hopt"].uid()).ok(),
                    None,
                    "Option can set in any order, not access it in option"
                );
                Ok(Some(val.deref() + 21))
            });

        // 10
        set.add_opt("--jopt=u")?.set_force(false).run()?;
        set.add_opt("--kopt=u")?
            .set_action(Action::Set)
            .add_alias("--alias-k")
            .run()?;

        // 13
        set.add_opt("--lopt=f!")?.add_alias("-l").run()?;
        set.add_opt("--mopt=f")?.set_value(1.02f64).run()?;
        set.add_opt("--nopt=f")?.set_action(Action::Set).run()?;

        // 16
        set.add_opt("--oopt=s!")?.add_alias("-o");
        set.add_opt("--popt=s")?.run()?;
        inv.entry(set.add_opt("--qopt=s")?.run()?)
            .on(|_: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>| Ok(Some(val.take())))
            .then(
                |uid: Uid,
                 set: &mut ASet,
                 ser: &mut ASer,
                 raw: Option<&RawVal>,
                 val: Option<String>| {
                    if let Some(val) = val {
                        // let's put the value to `popt`
                        ser.ser_val_mut().push(set["--popt"].uid(), val);
                        if let Some(raw) = raw {
                            ser.ser_rawval_mut().push(uid, raw.clone());
                        }
                        Ok(Some(()))
                    } else {
                        Ok(None)
                    }
                },
            );

        // 19
        set.add_opt("--开关=b")?;
        set.add_opt("--值=s")?;
        set.add_opt("--りょう=i")?;
        set.add_opt("test_cmd=c")?;

        let set_uid = set.add_opt("set=c")?.run()?;
        let bpos_uid = set.add_opt("bpos=p@[2,3]")?.set_assoc(Assoc::Uint).run()?;
        let cpos_uid = set
            .add_opt("cpos=p@4..5")?
            .set_validator(string_collection_validator(vec!["average", "plus"]))
            .run()?;
        let dpos_uid = set
            .add_opt("dpos=p@..7")?
            .set_validator(index_validator(vec![5, 6]))
            .set_action(Action::Set)
            .run()?;
        let epos_uid = set.add_opt("epos=p@7..")?.run()?;

        inv.entry(set.add_opt("main=m")?.run()?).on(
            move |set: &mut ASet, ser: &mut Services, idx: ctx::Index| {
                let copt = &set["--copt"];
                let dopt = &set["--/dopt"];
                let bpos = &set["bpos"];
                let cpos = &set[cpos_uid];
                let dpos = &set[dpos_uid];
                let epos = &set["epos"];

                assert_eq!(idx.deref(), &0);
                check_opt_val::<String>(
                    ser,
                    epos,
                    epos_uid,
                    "epos",
                    Some(vec!["反转".to_owned(), "翻转".to_owned()]),
                    false,
                    &Action::App,
                    &Assoc::Noa,
                    Some(&Index::Range(7, 0)),
                    None,
                )?;
                check_opt_val::<String>(
                    ser,
                    dpos,
                    dpos_uid,
                    "dpos",
                    Some(vec!["program -- software".to_owned()]),
                    false,
                    &Action::Set,
                    &Assoc::Noa,
                    Some(&Index::Range(0, 7)),
                    None,
                )?;
                check_opt_val(
                    ser,
                    cpos,
                    cpos_uid,
                    "cpos",
                    Some(vec![2.31]),
                    false,
                    &Action::App,
                    &Assoc::Noa,
                    Some(&Index::Range(4, 5)),
                    None,
                )?;
                check_opt_val::<u64>(
                    ser,
                    bpos,
                    bpos_uid,
                    "bpos",
                    Some(vec![32, 64]),
                    false,
                    &Action::App,
                    &Assoc::Uint,
                    Some(&Index::list(vec![2, 3])),
                    None,
                )?;
                check_opt_val::<u64>(
                    ser,
                    copt,
                    2,
                    "--copt",
                    Some(vec![1]),
                    true,
                    &Action::Cnt,
                    &Assoc::Bool,
                    None,
                    None,
                )?;
                check_opt_val(
                    ser,
                    dopt,
                    3,
                    "--/dopt",
                    Some(vec![true]),
                    true,
                    &Action::Set,
                    &Assoc::Bool,
                    None,
                    None,
                )?;
                Ok(Some(true))
            },
        );
        inv.entry(epos_uid).on(
            |set: &mut ASet, ser: &mut ASer, mut val: ctx::Value<String>, idx: ctx::Index| {
                let ropt = &set["--开关"];
                let sopt = &set["--值"];
                let topt = &set["--りょう"];

                check_opt_val::<i64>(
                    ser,
                    topt,
                    19,
                    "--りょう",
                    Some(vec![88]),
                    false,
                    &Action::App,
                    &Assoc::Int,
                    None,
                    None,
                )?;
                check_opt_val::<String>(
                    ser,
                    sopt,
                    18,
                    "--值",
                    Some(vec![String::from("恍恍惚惚")]),
                    false,
                    &Action::App,
                    &Assoc::Str,
                    None,
                    None,
                )?;
                check_opt_val(
                    ser,
                    ropt,
                    17,
                    "--开关",
                    Some(vec![true]),
                    false,
                    &Action::Set,
                    &Assoc::Bool,
                    None,
                    None,
                )?;
                assert!(idx.deref() == &7 || idx.deref() == &8);
                Ok(Some(val.take()))
            },
        );
        inv.entry(dpos_uid).on(
            |set: &mut ASet, ser: &mut ASer, mut val: ctx::Value<String>, idx: ctx::Index| {
                let oopt = &set["--oopt"];
                let popt = &set["--popt"];
                let qopt = &set["--qopt"];

                check_opt_val::<String>(
                    ser,
                    qopt,
                    16,
                    "--qopt",
                    None,
                    false,
                    &Action::App,
                    &Assoc::Str,
                    None,
                    None,
                )?;
                check_opt_val(
                    ser,
                    popt,
                    15,
                    "--popt",
                    Some(vec![String::from("cpp"), String::from("rust")]),
                    false,
                    &Action::App,
                    &Assoc::Str,
                    None,
                    None,
                )?;
                check_opt_val(
                    ser,
                    oopt,
                    14,
                    "--oopt",
                    Some(vec![String::from("lily")]),
                    true,
                    &Action::App,
                    &Assoc::Str,
                    None,
                    Some(vec![("-o")]),
                )?;
                assert!(idx.deref() == &5 || idx.deref() == &6);
                match ser.sve_val::<String>(set["dpos"].uid()) {
                    Ok(last_val) => Ok(Some(format!("{} -- {}", last_val, val.take()))),
                    Err(_) => Ok(Some(val.take())),
                }
            },
        );
        inv.entry(cpos_uid).on(
            |set: &mut ASet, ser: &mut ASer, val: ctx::Value<String>, idx: ctx::Index| {
                let lopt = &set["--lopt"];
                let mopt = &set["--mopt"];
                let nopt = &set["--nopt"];

                check_opt_val(
                    ser,
                    nopt,
                    13,
                    "--nopt",
                    Some(vec![3.12]),
                    false,
                    &Action::Set,
                    &Assoc::Flt,
                    None,
                    None,
                )?;
                check_opt_val::<f64>(
                    ser,
                    mopt,
                    12,
                    "--mopt",
                    Some(vec![1.02]),
                    false,
                    &Action::App,
                    &Assoc::Flt,
                    None,
                    None,
                )?;
                check_opt_val::<f64>(
                    ser,
                    lopt,
                    11,
                    "--lopt",
                    Some(vec![2.79]),
                    true,
                    &Action::App,
                    &Assoc::Flt,
                    None,
                    Some(vec![("-l")]),
                )?;
                assert!(idx.deref() == &4);

                let mut sum = 0.0;

                for uid in [lopt, mopt, nopt].iter().map(|v| v.uid()) {
                    sum += ser.sve_val::<f64>(uid)?;
                }

                match val.deref().as_str() {
                    "average" => Ok(Some(sum / 3.0)),
                    "plus" => Ok(Some(sum)),
                    _ => Ok(None),
                }
            },
        );
        inv.entry(bpos_uid).on(
            |set: &mut ASet, ser: &mut ASer, val: ctx::Value<u64>, idx: ctx::Index| {
                let jopt = &set["--jopt"];
                let kopt = &set["--kopt"];

                check_opt_val::<u64>(
                    ser,
                    jopt,
                    9,
                    "--jopt",
                    Some(vec![2]),
                    false,
                    &Action::App,
                    &Assoc::Uint,
                    None,
                    None,
                )?;
                check_opt_val::<u64>(
                    ser,
                    kopt,
                    10,
                    "--kopt",
                    Some(vec![4]),
                    false,
                    &Action::Set,
                    &Assoc::Uint,
                    None,
                    None,
                )?;
                assert!(idx.deref() == &2 || idx.deref() == &3);
                Ok(Some(
                    val.deref() * ser.sve_val::<u64>(set["--alias-k"].uid())?,
                ))
            },
        );
        inv.entry(set_uid).on(
            move |set: &mut ASet,
                  ser: &mut ASer,
                  uid: ctx::Uid,
                  name: ctx::Name,
                  mut value: ctx::Value<String>| {
                let aopt = &set[0];
                let bopt = &set["--/bopt"];
                let apos = &set[*uid.deref()];
                let eopt = &set["+eopt"];
                let fopt = &set["--/fopt=b"];
                let gopt = &set["--gopt"];
                let hopt = &set["--hopt"];
                let iopt = &set["--iopt"];

                assert_eq!(name.deref(), "set");
                check_opt_val::<i64>(
                    ser,
                    iopt,
                    8,
                    "--iopt",
                    Some(vec![84, -21, 21]),
                    false,
                    &Action::App,
                    &Assoc::Int,
                    None,
                    Some(vec![("--iopt-alias1")]),
                )?;
                check_opt_val::<i64>(
                    ser,
                    hopt,
                    7,
                    "--hopt",
                    Some(vec![48]),
                    true,
                    &Action::App,
                    &Assoc::Int,
                    None,
                    None,
                )?;
                check_opt_val::<i64>(
                    ser,
                    gopt,
                    6,
                    "--gopt",
                    None,
                    false,
                    &Action::App,
                    &Assoc::Int,
                    None,
                    None,
                )?;

                check_opt_val(
                    ser,
                    fopt,
                    5,
                    "--/fopt",
                    Some(vec![true]),
                    false,
                    &Action::Set,
                    &Assoc::Bool,
                    None,
                    Some(vec![("-/fopt")]),
                )?;
                check_opt_val(
                    ser,
                    eopt,
                    4,
                    "--eopt",
                    Some(vec![true]),
                    false,
                    &Action::Set,
                    &Assoc::Bool,
                    None,
                    Some(vec![("+eopt")]),
                )?;
                check_opt_val(
                    ser,
                    bopt,
                    1,
                    "--/bopt",
                    Some(vec![false]),
                    false,
                    &Action::Set,
                    &Assoc::Bool,
                    None,
                    None,
                )?;
                check_opt_val(
                    ser,
                    aopt,
                    0,
                    "--aopt",
                    Some(vec![false]),
                    false,
                    &Action::Set,
                    &Assoc::Bool,
                    None,
                    None,
                )?;
                check_opt_val::<String>(
                    ser,
                    apos,
                    set_uid,
                    "set",
                    None,
                    true,
                    &Action::Set,
                    &Assoc::Noa,
                    Some(&Index::forward(1)),
                    None,
                )?;
                Ok(Some(value.take()))
            },
        );
        for opt in set.iter_mut() {
            opt.init(&mut ser)?;
        }
        policy.parse(&mut set, &mut inv, &mut ser, Arc::new(args))?;
        Ok(())
    }
}
