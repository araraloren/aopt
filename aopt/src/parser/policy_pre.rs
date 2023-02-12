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
use super::OptStyleManager;
use super::Policy;
use super::ReturnVal;
use super::SetChecker;
use super::UserStyle;
use super::UserStyleManager;
use crate::args::ArgParser;
use crate::args::Args;
use crate::ctx::Ctx;
use crate::ctx::Invoker;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::proc::Process;
use crate::set::OptValidator;
use crate::set::SetOpt;
use crate::ARef;
use crate::Error;

/// [`PrePolicy`] matching the command line arguments with [`Opt`] in the [`Set`](crate::set::Set).
/// [`PrePolicy`] will skip any special [`Error`] during [`parse`](Policy::parse) process.
/// [`PrePolicy`] will return Some([`ReturnVal`]) if match successful.
/// [`PrePolicy`] don't consume the `NOA` when process [`NOAMatch`](crate::proc::NOAMatch).
///
/// # Example
/// ```rust
/// # use aopt::getopt;
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// # use std::ops::Deref;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut parser = AFwdParser::default();
/// let mut cfg_loader = APreParser::default();
///
/// parser
///     .add_opt("-check=s")?
///     .on(|set: &mut ASet, _: &mut ASer, ext: ctx::Value<String>| {
///         let mut found = false;
///
///         for name in ["-c", "-cxx"] {
///             if let Some(opt) = set.find(name)? {
///                 if let Ok(file) = opt.vals::<String>() {
///                     if file.contains(ext.deref()) {
///                         found = true;
///                     }
///                 }
///             }
///         }
///         Ok(Some(found))
///     })?;
/// cfg_loader.set_app_data(parser)?;
/// cfg_loader.add_opt("--load=s")?.on(
///     |_: &mut ASet, ser: &mut ASer, mut cfg: ctx::Value<String>| {
///         let parser = ser.sve_val_mut::<AFwdParser>()?;
///
///         match cfg.as_str() {
///             "cxx" => {
///                 parser.add_opt_i::<String>("-cxx")?.set_values(
///                     ["cxx", "cpp", "c++", "cc", "hpp", "hxx", "h"]
///                         .map(|v| v.to_owned())
///                         .to_vec(),
///                 );
///             }
///             "c" => {
///                 parser
///                     .add_opt("-c=s")?
///                     .set_values_t(["c", "h"].map(|v| v.to_owned()).to_vec());
///             }
///             _ => {
///                 panic!("Unknow configuration name")
///             }
///         }
///
///         Ok(Some(cfg.take()))
///     },
/// )?;
///
/// let ret = getopt!(
///     Args::from_array(["--load", "cxx", "-check", "cc"]),
///     &mut cfg_loader
/// )?;
/// let next_args = ret.unwrap().ret.clone_args();
/// let mut parser = cfg_loader.service_mut().sve_take_val::<AFwdParser>()?;
///
/// getopt!(Args::from_vec(next_args), &mut parser)?;
///
/// assert!(*parser.find_val::<bool>("-check")?);
///
/// // pass the parser to AppService
/// cfg_loader.set_app_data(parser)?;
///
/// let ret = getopt!(
///     Args::from_array(["--load", "c", "-check", "c"]),
///     &mut cfg_loader
/// )?;
/// let next_args = ret.unwrap().ret.clone_args();
/// let mut parser = cfg_loader.service_mut().sve_take_val::<AFwdParser>()?;
///
/// getopt!(Args::from_vec(next_args), &mut parser)?;
///
/// assert!(*parser.find_val::<bool>("-check")?);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PrePolicy<Set, Ser> {
    strict: bool,

    style_manager: OptStyleManager,

    checker: SetChecker<Set>,

    marker_s: PhantomData<(Set, Ser)>,
}

impl<Set, Ser> Default for PrePolicy<Set, Ser> {
    fn default() -> Self {
        Self {
            strict: false,
            style_manager: OptStyleManager::default(),
            checker: SetChecker::default(),
            marker_s: PhantomData::default(),
        }
    }
}

impl<Set, Ser> PrePolicy<Set, Ser> {
    pub fn new(strict: bool, styles: OptStyleManager) -> Self {
        Self {
            strict,
            style_manager: styles,
            ..Self::default()
        }
    }

    /// In strict mode, if an argument looks like an option (it matched any option prefix),
    /// then it must matched, otherwise it will be discarded.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    pub fn with_styles(mut self, styles: Vec<UserStyle>) -> Self {
        self.style_manager.set(styles);
        self
    }

    pub fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.strict = strict;
        self
    }

    pub fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.style_manager.set(styles);
        self
    }

    pub fn strict(&self) -> bool {
        self.strict
    }

    pub fn checker(&self) -> &SetChecker<Set> {
        &self.checker
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

    pub(crate) fn noa_cmd() -> usize {
        1
    }

    pub(crate) fn noa_main() -> usize {
        0
    }

    pub(crate) fn noa_pos(idx: usize) -> usize {
        idx
    }
}

impl<Set, Ser> UserStyleManager for PrePolicy<Set, Ser> {
    fn style_manager(&self) -> &OptStyleManager {
        &self.style_manager
    }

    fn style_manager_mut(&mut self) -> &mut OptStyleManager {
        &mut self.style_manager
    }
}

impl<Set, Ser> PrePolicy<Set, Ser>
where
    SetOpt<Set>: Opt,
    Ser: 'static,
    Set: crate::set::Set + OptParser + OptValidator + Debug + 'static,
{
    pub(crate) fn parse_impl(
        &mut self,
        ctx: &mut Ctx,
        set: &mut <Self as Policy>::Set,
        inv: &mut <Self as Policy>::Inv,
        ser: &mut <Self as Policy>::Ser,
    ) -> Result<(), <Self as Policy>::Error> {
        Self::ig_failure(self.checker().pre_check(set))?;

        let opt_styles = &self.style_manager;
        let args = ctx.orig_args().clone();
        let args_len = args.len();
        let mut noa_args = Args::default();
        let mut iter = args.guess_iter().enumerate();

        ctx.set_args(args.clone());
        while let Some((idx, (opt, arg))) = iter.next() {
            let mut matched = false;
            let mut consume = false;
            let mut like_opt = false;
            let arg = arg.map(|v| ARef::new(v.clone()));

            if let Ok(clopt) = opt.parse_arg() {
                if let Some(name) = clopt.name() {
                    if let Some(valid) =
                        Self::ig_failure(set.check(name.as_str()).map_err(Into::into))?
                    {
                        if valid {
                            like_opt = true;
                            for style in opt_styles.iter() {
                                let ret = Self::ig_failure(OptGuess::new().guess(
                                    style,
                                    GuessOptCfg::new(idx, args_len, arg.clone(), &clopt, set),
                                ))?;

                                if let Some(Some(mut proc)) = ret {
                                    if Self::ig_failure(process_opt(
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
                                    ))?
                                    .is_some()
                                    {
                                        if proc.status() {
                                            matched = true;
                                        }
                                        if proc.is_consume() {
                                            consume = true;
                                        }
                                        if matched {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // if consume the argument, skip it
            if matched && consume {
                iter.next();
            } else if !matched && !self.strict() || !like_opt {
                // add it to NOA if current argument not matched
                // and not in strict mode or the argument not like an option
                noa_args.push(args[idx].clone());
            }
        }

        Self::ig_failure(self.checker().opt_check(set))?;

        let noa_args = ARef::new(noa_args);
        let noa_len = noa_args.len();

        ctx.set_args(noa_args.clone());
        if noa_len > 0 {
            if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(noa_args.clone(), Self::noa_cmd(), noa_len),
            ))? {
                Self::ig_failure(process_non_opt(
                    ProcessCtx {
                        ctx,
                        set,
                        inv,
                        ser,
                        tot: noa_len,
                        idx: Self::noa_cmd(),
                    },
                    &mut proc,
                ))?;
            }

            Self::ig_failure(self.checker().cmd_check(set))?;

            for idx in 1..noa_len {
                if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), Self::noa_pos(idx), noa_len),
                ))? {
                    Self::ig_failure(process_non_opt(
                        ProcessCtx {
                            ctx,
                            set,
                            inv,
                            ser,
                            tot: noa_len,
                            idx: Self::noa_pos(idx),
                        },
                        &mut proc,
                    ))?;
                }
            }
        } else {
            Self::ig_failure(self.checker().cmd_check(set))?;
        }

        Self::ig_failure(self.checker().pos_check(set))?;

        let main_args = noa_args;
        let main_len = main_args.len();

        // set 0 for Main's index
        if let Some(Some(mut proc)) = Self::ig_failure(NOAGuess::new().guess(
            &UserStyle::Main,
            GuessNOACfg::new(main_args, Self::noa_main(), noa_len),
        ))? {
            Self::ig_failure(process_non_opt(
                ProcessCtx {
                    ctx,
                    set,
                    inv,
                    ser,
                    tot: main_len,
                    idx: Self::noa_main(),
                },
                &mut proc,
            ))?;
        }

        Self::ig_failure(self.checker().post_check(set))?;

        Ok(())
    }
}

impl<Set, Ser> Policy for PrePolicy<Set, Ser>
where
    SetOpt<Set>: Opt,
    Ser: 'static,
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
        args: ARef<Args>,
    ) -> Result<Self::Ret, Self::Error> {
        let mut ctx = Ctx::default().with_orig_args(args.clone()).with_args(args);

        match self.parse_impl(&mut ctx, set, inv, ser) {
            Ok(_) => Ok(ReturnVal::new(ctx)),
            Err(e) => {
                if e.is_failure() {
                    Ok(ReturnVal::new(ctx).with_failure(e))
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use std::any::TypeId;
    use std::ops::Deref;

    use crate::opt::Cmd;
    use crate::opt::Pos;
    use crate::prelude::*;
    use crate::ARef;
    use crate::Error;
    use crate::RawVal;

    #[test]
    fn testing_1() {
        assert!(testing_1_main().is_ok());
    }

    fn testing_1_main() -> Result<(), Error> {
        fn check_opt_val<T: std::fmt::Debug + PartialEq + ErasedTy + 'static>(
            opt: &AOpt,
            uid: Uid,
            name: &str,
            vals: Option<Vec<T>>,
            force: bool,
            action: &Action,
            type_id: &TypeId,
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
            assert_eq!(
                opt.r#type(),
                type_id,
                "type_id not equal for {}({})",
                opt_uid,
                opt.name(),
            );
            assert_eq!(opt.index(), index, "option index not equal: {:?}", index);
            if let Ok(opt_vals) = opt.vals::<T>() {
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

        let mut policy = APrePolicy::default();
        let mut set = policy.default_set();
        let mut inv = policy.default_inv();
        let mut ser = policy.default_ser();
        let args = Args::from_array([
            "app", // 0
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
            "left",
            "--wopt=98",
            "剩余的",
            "--ropt=23",
            "-r",
            "--s我的",
        ]);

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
            .on(|set: &mut ASet, _: &mut ASer, val: ctx::Value<i64>| {
                assert_eq!(
                    set["--hopt"].val::<i64>().ok(),
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
        set.add_opt("--mopt=f")?.set_value_t(1.02f64).run()?;
        set.add_opt("--nopt=f")?.set_action(Action::Set).run()?;

        // 16
        set.add_opt("--oopt=s!")?.add_alias("-o");
        set.add_opt("--popt=s")?.run()?;
        inv.entry(set.add_opt("--qopt=s")?.run()?)
            .on(|_: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>| Ok(Some(val.take())))
            .then(
                |uid: Uid,
                 set: &mut ASet,
                 _: &mut ASer,
                 raw: Option<&RawVal>,
                 val: Option<String>| {
                    if let Some(val) = val {
                        // let's put the value to `popt`
                        set["--popt"].accessor_mut().push(val);
                        if let Some(raw) = raw {
                            set[uid].rawvals_mut()?.push(raw.clone());
                        }
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                },
            );

        // 19
        set.add_opt("--开关=b")?;
        set.add_opt("--值=s")?;
        set.add_opt("--りょう=i")?;
        set.add_opt("test_cmd=c")?;

        let set_uid = set.add_opt("set=c")?.run()?;
        let bpos_uid = set
            .add_opt("bpos=p@[2,3]")?
            .set_pos_type_only::<u64>()
            .run()?;
        let cpos_uid = set
            .add_opt_i::<Pos<String>>("cpos@4..5")?
            .set_validator(ValValidator::contains2(vec!["average", "plus"]))
            .run()?;
        let dpos_uid = set.add_opt("dpos=p@5..7")?.set_action(Action::Set).run()?;
        let epos_uid = set.add_opt("epos=p@7..9")?.run()?;

        inv.entry(set.add_opt("main=m")?.run()?).on(
            move |set: &mut ASet, _: &mut ASer, idx: ctx::Index, name: ctx::Name| {
                let copt = &set["--copt"];
                let dopt = &set["--/dopt"];
                let bpos = &set["bpos"];
                let cpos = &set[cpos_uid];
                let dpos = &set[dpos_uid];
                let epos = &set["epos"];

                assert_eq!(idx.deref(), &0);
                assert_eq!(name.deref(), "app");
                check_opt_val::<String>(
                    epos,
                    epos_uid,
                    "epos",
                    Some(vec!["反转".to_owned(), "翻转".to_owned()]),
                    false,
                    &Action::App,
                    &TypeId::of::<Pos>(),
                    Some(&Index::Range(7, Some(9))),
                    None,
                )?;
                check_opt_val::<String>(
                    dpos,
                    dpos_uid,
                    "dpos",
                    Some(vec!["program -- software".to_owned()]),
                    false,
                    &Action::Set,
                    &TypeId::of::<Pos>(),
                    Some(&Index::Range(5, Some(7))),
                    None,
                )?;
                check_opt_val(
                    cpos,
                    cpos_uid,
                    "cpos",
                    Some(vec![2.31]),
                    false,
                    &Action::App,
                    &TypeId::of::<Pos<String>>(),
                    Some(&Index::Range(4, Some(5))),
                    None,
                )?;
                check_opt_val::<u64>(
                    bpos,
                    bpos_uid,
                    "bpos",
                    Some(vec![32, 64]),
                    false,
                    &Action::App,
                    &TypeId::of::<Pos<u64>>(),
                    Some(&Index::list(vec![2, 3])),
                    None,
                )?;
                check_opt_val::<u64>(
                    copt,
                    2,
                    "--copt",
                    Some(vec![1]),
                    true,
                    &Action::Cnt,
                    &TypeId::of::<bool>(),
                    None,
                    None,
                )?;
                check_opt_val(
                    dopt,
                    3,
                    "--/dopt",
                    Some(vec![true]),
                    true,
                    &Action::Set,
                    &TypeId::of::<bool>(),
                    None,
                    None,
                )?;
                Ok(Some(true))
            },
        );
        inv.entry(epos_uid).on(
            |set: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>, idx: ctx::Index| {
                let ropt = &set["--开关"];
                let sopt = &set["--值"];
                let topt = &set["--りょう"];

                check_opt_val::<i64>(
                    topt,
                    19,
                    "--りょう",
                    Some(vec![88]),
                    false,
                    &Action::App,
                    &TypeId::of::<i64>(),
                    None,
                    None,
                )?;
                check_opt_val::<String>(
                    sopt,
                    18,
                    "--值",
                    Some(vec![String::from("恍恍惚惚")]),
                    false,
                    &Action::App,
                    &TypeId::of::<String>(),
                    None,
                    None,
                )?;
                check_opt_val(
                    ropt,
                    17,
                    "--开关",
                    Some(vec![true]),
                    false,
                    &Action::Set,
                    &TypeId::of::<bool>(),
                    None,
                    None,
                )?;
                assert!(idx.deref() == &7 || idx.deref() == &8);
                Ok(Some(val.take()))
            },
        );
        inv.entry(dpos_uid).on(
            |set: &mut ASet, _: &mut ASer, mut val: ctx::Value<String>, idx: ctx::Index| {
                let oopt = &set["--oopt"];
                let popt = &set["--popt"];
                let qopt = &set["--qopt"];

                check_opt_val::<String>(
                    qopt,
                    16,
                    "--qopt",
                    None,
                    false,
                    &Action::App,
                    &TypeId::of::<String>(),
                    None,
                    None,
                )?;
                check_opt_val(
                    popt,
                    15,
                    "--popt",
                    Some(vec![String::from("cpp"), String::from("rust")]),
                    false,
                    &Action::App,
                    &TypeId::of::<String>(),
                    None,
                    None,
                )?;
                check_opt_val(
                    oopt,
                    14,
                    "--oopt",
                    Some(vec![String::from("lily")]),
                    true,
                    &Action::App,
                    &TypeId::of::<String>(),
                    None,
                    Some(vec![("-o")]),
                )?;
                assert!(idx.deref() == &5 || idx.deref() == &6);
                match set["dpos"].val::<String>() {
                    Ok(last_val) => Ok(Some(format!("{} -- {}", last_val, val.take()))),
                    Err(_) => Ok(Some(val.take())),
                }
            },
        );
        inv.entry(cpos_uid).on(
            |set: &mut ASet, _: &mut ASer, val: ctx::Value<String>, idx: ctx::Index| {
                let lopt = &set["--lopt"];
                let mopt = &set["--mopt"];
                let nopt = &set["--nopt"];

                check_opt_val(
                    nopt,
                    13,
                    "--nopt",
                    Some(vec![3.12]),
                    false,
                    &Action::Set,
                    &TypeId::of::<f64>(),
                    None,
                    None,
                )?;
                check_opt_val::<f64>(
                    mopt,
                    12,
                    "--mopt",
                    Some(vec![1.02]),
                    false,
                    &Action::App,
                    &TypeId::of::<f64>(),
                    None,
                    None,
                )?;
                check_opt_val::<f64>(
                    lopt,
                    11,
                    "--lopt",
                    Some(vec![2.79]),
                    true,
                    &Action::App,
                    &TypeId::of::<f64>(),
                    None,
                    Some(vec![("-l")]),
                )?;
                assert!(idx.deref() == &4);

                let mut sum = 0.0;

                for uid in [lopt, mopt, nopt].iter().map(|v| v.uid()) {
                    sum += set[uid].val::<f64>()?;
                }

                match val.deref().as_str() {
                    "average" => Ok(Some(sum / 3.0)),
                    "plus" => Ok(Some(sum)),
                    _ => Ok(None),
                }
            },
        );
        inv.entry(bpos_uid).on(
            |set: &mut ASet, _: &mut ASer, val: ctx::Value<u64>, idx: ctx::Index| {
                let jopt = &set["--jopt"];
                let kopt = &set["--kopt"];

                check_opt_val::<u64>(
                    jopt,
                    9,
                    "--jopt",
                    Some(vec![2]),
                    false,
                    &Action::App,
                    &TypeId::of::<u64>(),
                    None,
                    None,
                )?;
                check_opt_val::<u64>(
                    kopt,
                    10,
                    "--kopt",
                    Some(vec![4]),
                    false,
                    &Action::Set,
                    &TypeId::of::<u64>(),
                    None,
                    None,
                )?;
                assert!(idx.deref() == &2 || idx.deref() == &3);
                Ok(Some(val.deref() * set["--alias-k"].val::<u64>()?))
            },
        );
        inv.entry(set_uid).on(
            move |set: &mut ASet,
                  _: &mut ASer,
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
                    iopt,
                    8,
                    "--iopt",
                    Some(vec![84, -21, 21]),
                    false,
                    &Action::App,
                    &TypeId::of::<i64>(),
                    None,
                    Some(vec![("--iopt-alias1")]),
                )?;
                check_opt_val::<i64>(
                    hopt,
                    7,
                    "--hopt",
                    Some(vec![48]),
                    true,
                    &Action::App,
                    &TypeId::of::<i64>(),
                    None,
                    None,
                )?;
                check_opt_val::<i64>(
                    gopt,
                    6,
                    "--gopt",
                    None,
                    false,
                    &Action::App,
                    &TypeId::of::<i64>(),
                    None,
                    None,
                )?;

                check_opt_val(
                    fopt,
                    5,
                    "--/fopt",
                    Some(vec![true]),
                    false,
                    &Action::Set,
                    &TypeId::of::<bool>(),
                    None,
                    Some(vec![("-/fopt")]),
                )?;
                check_opt_val(
                    eopt,
                    4,
                    "--eopt",
                    Some(vec![true]),
                    false,
                    &Action::Set,
                    &TypeId::of::<bool>(),
                    None,
                    Some(vec![("+eopt")]),
                )?;
                check_opt_val(
                    bopt,
                    1,
                    "--/bopt",
                    Some(vec![false]),
                    false,
                    &Action::Set,
                    &TypeId::of::<bool>(),
                    None,
                    None,
                )?;
                check_opt_val(
                    aopt,
                    0,
                    "--aopt",
                    Some(vec![false]),
                    false,
                    &Action::Set,
                    &TypeId::of::<bool>(),
                    None,
                    None,
                )?;
                check_opt_val::<String>(
                    apos,
                    set_uid,
                    "set",
                    None,
                    true,
                    &Action::Set,
                    &TypeId::of::<Cmd>(),
                    Some(&Index::forward(1)),
                    None,
                )?;
                Ok(Some(value.take()))
            },
        );
        for opt in set.iter_mut() {
            opt.init()?;
        }
        let ret = policy.parse(&mut set, &mut inv, &mut ser, ARef::new(args));

        assert!(ret.is_ok());
        let ret = ret.unwrap();
        let args = ret.args();

        for (idx, arg) in [
            "app",
            "set",
            "8",
            "16",
            "average",
            "program",
            "software",
            "反转",
            "翻转",
            "left",
            "--wopt=98",
            "剩余的",
            "--ropt=23",
            "-r",
            "--s我的",
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(args[idx].get_str(), Some(*arg));
        }
        Ok(())
    }
}
