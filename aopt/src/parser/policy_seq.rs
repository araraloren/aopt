use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;

use super::FailManager;
use super::OptStyleManager;
use super::Policy;
use super::PolicySettings;
use super::Return;
use super::UserStyle;
use crate::args;
use crate::args::ArgInfo;
use crate::args::Args;
use crate::ctx::Ctx;
use crate::ctx::Invoker;
use crate::guess::InvokeGuess;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::parser::Action;
use crate::set::OptValidator;
use crate::set::SetChecker;
use crate::set::SetOpt;
use crate::trace;
use crate::Error;

/// [`SeqPolicy`] matching the command line arguments with [`Opt`] in the [`Set`](crate::set::Set).
/// The option would match failed if any special [`Error`] raised during option processing.
/// [`SeqPolicy`] will return Some([`Return`]) if match successful.
/// [`SeqPolicy`] process the option before any
/// NOA([`Cmd`](crate::opt::Style::Cmd), [`Pos`](crate::opt::Style::Pos) and [`Main`](crate::opt::Style::Main)).
/// During parsing, you can get the value of any option in the handler of NOA.
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AFwdPolicy::default();
/// let mut set = AHCSet::default();
/// let mut inv = AInvoker::default();
/// let filter_id = set.add_opt("--/filter=b")?.run()?;
/// let pos_id = set.add_opt("pos=p@*")?
///                 .set_pos_type::<String>()
///                 .set_values(vec![])
///                 .run()?;
///
/// inv.entry(pos_id).on(
///     move |set, ctx| {
///         let filter = set.app_data::<Vec<&str>>()?;
///         let value = ctx.value::<String>()?;
///         let not_filter = set[filter_id].val::<bool>()?;
///         let valid = if !*not_filter {
///             !filter.iter().any(|&v| v == value.as_str())
///         } else {
///             true
///         };
///
///         Ok(valid.then(|| value))
///     },
/// );
///
/// let args = Args::from(["app", "set", "42", "foo", "bar"]);
///
/// for opt in set.iter_mut() {
///     opt.init()?;
/// }
/// set.set_app_data(vec!["foo", "bar"]);
/// policy.parse(&mut set, &mut inv, args)?;
///
/// let values = set[pos_id].vals::<String>()?;
///
/// assert_eq!(values[0], "set");
/// assert_eq!(values[1], "42");
///
/// let args = Args::from(["app", "--/filter", "set", "42", "foo", "bar"]);
///
/// for opt in set.iter_mut() {
///     opt.init()?;
/// }
///
/// policy.parse(&mut set, &mut inv, args)?;
/// let values = set[pos_id].vals::<String>()?;
///
/// assert_eq!(values[0], "set");
/// assert_eq!(values[1], "42");
/// assert_eq!(values[2], "foo");
/// assert_eq!(values[3], "bar");
/// #
/// # Ok(())
/// # }
/// ```
pub struct SeqPolicy<S, Chk> {
    strict: bool,

    overload: bool,

    checker: Chk,

    style_manager: OptStyleManager,

    marker_s: PhantomData<S>,
}

impl<S, Chk> Clone for SeqPolicy<S, Chk>
where
    Chk: Clone,
{
    fn clone(&self) -> Self {
        Self {
            strict: self.strict,
            overload: self.overload,
            checker: self.checker.clone(),
            style_manager: self.style_manager.clone(),
            marker_s: self.marker_s,
        }
    }
}

impl<S, Chk> Debug for SeqPolicy<S, Chk>
where
    Chk: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SeqPolicy")
            .field("strict", &self.strict)
            .field("overload", &self.overload)
            .field("checker", &self.checker)
            .field("style_manager", &self.style_manager)
            .finish()
    }
}

impl<S, Chk> Default for SeqPolicy<S, Chk>
where
    Chk: Default,
{
    fn default() -> Self {
        Self {
            strict: true,
            overload: false,
            style_manager: OptStyleManager::default(),
            checker: Chk::default(),
            marker_s: PhantomData,
        }
    }
}

impl<S, Chk> SeqPolicy<S, Chk>
where
    Chk: Default,
{
    pub fn new(strict: bool, style: OptStyleManager) -> Self {
        Self {
            strict,
            style_manager: style,
            ..Default::default()
        }
    }
}

impl<S, Chk> SeqPolicy<S, Chk> {
    /// In strict mode, if an argument looks like an option (it matched any option prefix),
    /// then it must matched.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    pub fn with_styles(mut self, styles: Vec<UserStyle>) -> Self {
        self.style_manager.set(styles);
        self
    }

    pub fn with_checker(mut self, checker: Chk) -> Self {
        self.checker = checker;
        self
    }

    pub fn with_overload(mut self, overload: bool) -> Self {
        self.overload = overload;
        self
    }

    pub fn set_checker(&mut self, checker: Chk) -> &mut Self {
        self.checker = checker;
        self
    }

    pub fn checker(&self) -> &Chk {
        &self.checker
    }

    pub fn checker_mut(&mut self) -> &mut Chk {
        &mut self.checker
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

impl<Set, Chk> PolicySettings for SeqPolicy<Set, Chk> {
    fn style_manager(&self) -> &OptStyleManager {
        &self.style_manager
    }

    fn style_manager_mut(&mut self) -> &mut OptStyleManager {
        &mut self.style_manager
    }

    fn strict(&self) -> bool {
        self.strict
    }

    fn styles(&self) -> &[UserStyle] {
        &self.style_manager
    }

    fn no_delay(&self) -> Option<&[String]> {
        None
    }

    fn overload(&self) -> bool {
        self.overload
    }

    fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.strict = strict;
        self
    }

    fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.style_manager.set(styles);
        self
    }

    fn set_no_delay(&mut self, _: impl Into<String>) -> &mut Self {
        self
    }

    fn set_overload(&mut self, overload: bool) -> &mut Self {
        self.overload = overload;
        self
    }
}

impl<S, Chk> SeqPolicy<S, Chk>
where
    SetOpt<S>: Opt,
    Chk: SetChecker<S>,
    S: crate::set::Set + OptParser + OptValidator,
{
    pub(crate) fn parse_impl<'a>(
        &mut self,
        set: &mut <Self as Policy>::Set,
        inv: &mut <Self as Policy>::Inv<'_>,
        orig: &'a Args,
        ctx: &mut Ctx<'a>,
    ) -> Result<(), <Self as Policy>::Error> {
        self.checker().pre_check(set).map_err(|e| e.into())?;

        let overload = self.overload();
        let opt_styles = &self.style_manager;
        let mut args: Vec<_> = orig.iter().map(|v| v.as_os_str()).collect();
        let total = args.len();
        let iter_args = args.clone();
        let mut lefts = vec![];
        let mut opt_fail = FailManager::default();
        let mut pos_fail = FailManager::default();
        let mut cmd_fail = Some(FailManager::default());
        let mut iter2 = args::iter2(&iter_args).enumerate();
        let mut noa_index = 0;

        trace!("parsing {ctx:?} using seq policy");
        ctx.set_args(args.clone());
        while let Some((idx, (opt, next))) = iter2.next() {
            let mut matched = false;
            let mut consume = false;
            let mut stopped = false;

            if let Ok(ArgInfo { name, value }) = ArgInfo::parse(opt) {
                trace!(
                    "guess name: {:?} value: {:?} & next: {:?}",
                    name,
                    value,
                    next
                );
                if set.check(&name).map_err(Into::into)? {
                    let arg = value.clone();
                    let next = next.map(|v| Cow::Borrowed(*v));
                    let mut guess = InvokeGuess {
                        idx,
                        arg,
                        set,
                        inv,
                        total,
                        ctx,
                        next,
                        fail: &mut opt_fail,
                        name: Some(name.clone()),
                    };

                    for style in opt_styles.iter() {
                        if let Some(ret) = guess.guess_and_invoke(style, overload)? {
                            (matched, consume) = (ret.matched, ret.consume);
                        }
                        match guess.ctx.policy_act() {
                            Action::Stop => {
                                stopped = true;
                                guess.ctx.reset_policy_act();
                                break;
                            }
                            Action::Quit => return Ok(()),
                            Action::Null => {}
                        }
                        if matched {
                            break;
                        }
                    }
                    if !stopped && !matched && self.strict() {
                        return Err(opt_fail.cause(Error::sp_not_found(name)));
                    }
                } else {
                    trace!("`{:?}` not like option", opt);
                }
            }
            if stopped {
                // skip current, put left argument to noa args
                lefts.extend(iter2.map(|(_, (a, _))| *a));
                break;
            }
            // if consume the argument, skip it
            if matched {
                args.remove(noa_index);
                if consume {
                    iter2.next();
                    if noa_index < args.len() {
                        args.remove(noa_index);
                    }
                }
                ctx.set_args(args.clone());
            } else {
                // process it as NOA if current argument not matched
                if noa_index == Self::noa_cmd() {
                    if let Some(mut cmd_fail) = cmd_fail.take() {
                        let name = crate::str::osstr_to_str_i(&args, Self::noa_cmd());
                        let mut guess = InvokeGuess {
                            set,
                            inv,
                            total: args.len(),
                            name,
                            ctx,
                            arg: None,
                            next: None,
                            fail: &mut cmd_fail,
                            idx: Self::noa_cmd(),
                        };

                        trace!("guess Cmd = {:?}", guess.name);
                        guess.guess_and_invoke(&UserStyle::Cmd, overload)?;
                        if let Action::Quit = ctx.policy_act() {
                            return Ok(());
                        }
                        cmd_fail.process_check(self.checker().cmd_check(set))?;
                    }
                }
                if noa_index >= 1 {
                    let mut guess = InvokeGuess {
                        set,
                        inv,
                        total: args.len(),
                        ctx,
                        name: None,
                        arg: None,
                        next: None,
                        fail: &mut pos_fail,
                        idx: Self::noa_pos(noa_index),
                    };

                    guess.name = crate::str::osstr_to_str_i(&args, Self::noa_pos(noa_index));
                    trace!("guess Pos argument = {:?} @ {}", guess.name, guess.idx);
                    guess.guess_and_invoke(&UserStyle::Pos, overload)?;
                    match guess.ctx.policy_act() {
                        Action::Stop => {
                            guess.ctx.reset_policy_act();
                            break;
                        }
                        Action::Quit => return Ok(()),
                        Action::Null => {}
                    }
                }
                noa_index += 1;
            }
        }

        // when style is pos, noa index is [1..=len]
        if let Some(cmd_fail) = cmd_fail {
            cmd_fail.process_check(self.checker().cmd_check(set))?;
        }
        opt_fail.process_check(self.checker().opt_check(set))?;

        pos_fail.process_check(self.checker().pos_check(set))?;

        let name = crate::str::osstr_to_str_i(&ctx.args, Self::noa_main());
        let mut main_fail = FailManager::default();
        let mut guess = InvokeGuess {
            set,
            inv,
            total,
            name,
            ctx,
            arg: None,
            next: None,
            fail: &mut main_fail,
            idx: Self::noa_main(),
        };

        trace!("guess Main {:?}", guess.name);
        guess.guess_and_invoke(&UserStyle::Main, overload)?;
        main_fail.process_check(self.checker().post_check(set))?;
        Ok(())
    }
}

impl<S, Chk> Policy for SeqPolicy<S, Chk>
where
    SetOpt<S>: Opt,
    Chk: SetChecker<S>,
    S: crate::set::Set + OptParser + OptValidator,
{
    type Ret = Return;

    type Set = S;

    type Inv<'a> = Invoker<'a, S>;

    type Error = Error;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv<'_>,

        orig: Args,
    ) -> Result<Self::Ret, Self::Error> {
        let mut ctx = Ctx::default().with_orig(orig.clone());

        match self.parse_impl(set, inv, &orig, &mut ctx) {
            Ok(_) => Ok(Return::new(ctx)),
            Err(e) => {
                if e.is_failure() {
                    Ok(Return::new(ctx).with_failure(e))
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
    use std::ffi::OsStr;

    use crate::opt::Cmd;
    use crate::opt::ConfigBuildInfer;
    use crate::opt::Pos;
    use crate::prelude::*;
    use crate::Error;

    #[test]
    fn testing_1() {
        assert!(testing_1_main().is_ok());
    }

    fn testing_1_main() -> Result<(), Error> {
        #[allow(clippy::too_many_arguments)]
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
            assert_eq!(opt.name(), name, "name not equal -{}({})-", opt_uid, name);
            assert_eq!(
                opt.force(),
                force,
                "option force required not equal -{}({})-: {}",
                opt_uid,
                name,
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

        let mut policy = AFwdPolicy::default();
        let mut set = AHCSet::default();
        let mut inv = AInvoker::default();
        let args = Args::from([
            "app",
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
            "反转",     //7
            "--值=恍恍惚惚",
            "--qopt",
            "rust",
            "翻转", // 8
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
            .on(|set, ctx| {
                assert_eq!(
                    set["--hopt"].val::<i64>().ok(),
                    None,
                    "Option can set in any order, not access it in option"
                );
                Ok(Some(ctx.value::<i64>()? + 21))
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
            .on(|_, ctx| Ok(Some(ctx.value::<String>()?)))
            .then(
                |uid: Uid, set: &mut AHCSet, raw: Option<&OsStr>, val: Option<String>| {
                    if let Some(val) = val {
                        // let's put the value to `popt`
                        set["--popt"].accessor_mut().push(val);
                        if let Some(raw) = raw {
                            set[uid].rawvals_mut()?.push(raw.to_os_string());
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
            .add_opt("cpos@4..5".infer::<Pos<String>>())?
            .set_validator(ValValidator::contains2(vec!["average", "plus"]))
            .run()?;
        let dpos_uid = set.add_opt("dpos=p@5..7")?.set_action(Action::Set).run()?;
        let epos_uid = set.add_opt("epos=p@7..")?.run()?;

        inv.entry(set.add_opt("main=m")?.run()?)
            .on(move |set, ctx| {
                let copt = &set["--copt"];
                let dopt = &set["--/dopt"];
                let bpos = &set["bpos"];
                let cpos = &set[cpos_uid];
                let dpos = &set[dpos_uid];
                let epos = &set["epos"];
                let idx = ctx.idx()?;
                let name = ctx.name()?;

                assert_eq!(idx, 0);
                assert_eq!(name.map(|v| v.as_ref()), Some("app"));
                check_opt_val::<String>(
                    epos,
                    epos_uid,
                    "epos",
                    Some(vec!["反转".to_owned(), "翻转".to_owned()]),
                    false,
                    &Action::App,
                    &TypeId::of::<Pos>(),
                    Some(&Index::Range(7, None)),
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
            });
        inv.entry(epos_uid).on(|set, ctx| {
            let ropt = &set["--开关"];
            let sopt = &set["--值"];
            let topt = &set["--りょう"];
            let idx = ctx.idx()?;
            let val = ctx.value::<String>()?;

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
            assert!(idx == 7 || idx == 8);
            Ok(Some(val))
        });
        inv.entry(dpos_uid).on(|set, ctx| {
            let oopt = &set["--oopt"];
            let popt = &set["--popt"];
            let qopt = &set["--qopt"];
            let idx = ctx.idx()?;
            let val = ctx.value::<String>()?;

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
            assert!(idx == 5 || idx == 6);
            match set["dpos"].val::<String>() {
                Ok(last_val) => Ok(Some(format!("{} -- {}", last_val, val))),
                Err(_) => Ok(Some(val)),
            }
        });
        inv.entry(cpos_uid).on(|set, ctx| {
            let lopt = &set["--lopt"];
            let mopt = &set["--mopt"];
            let nopt = &set["--nopt"];
            let idx = ctx.idx()?;
            let val = ctx.value::<String>()?;

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
            assert!(idx == 4);

            let mut sum = 0.0;

            for uid in [lopt, mopt, nopt].iter().map(|v| v.uid()) {
                sum += set[uid].val::<f64>()?;
            }

            match val.as_str() {
                "average" => Ok(Some(sum / 3.0)),
                "plus" => Ok(Some(sum)),
                _ => Ok(None),
            }
        });
        inv.entry(bpos_uid).on(|set, ctx| {
            let jopt = &set["--jopt"];
            let kopt = &set["--kopt"];
            let idx = ctx.idx()?;
            let val = ctx.value::<u64>()?;

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
            assert!(idx == 2 || idx == 3);
            Ok(Some(val * set["--alias-k"].val::<u64>()?))
        });
        inv.entry(set_uid).on(move |set, ctx| {
            let uid = ctx.uid()?;
            let aopt = &set[0];
            let bopt = &set["--/bopt"];
            let apos = &set[uid];
            let eopt = &set["+eopt"];
            let fopt = &set["--/fopt=b"];
            let gopt = &set["--gopt"];
            let hopt = &set["--hopt"];
            let iopt = &set["--iopt"];
            let name = ctx.name()?;
            let value = ctx.value::<String>()?;

            assert_eq!(name.map(|v| v.as_ref()), Some("set"));
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
            Ok(Some(value))
        });
        for opt in set.iter_mut() {
            opt.init()?;
        }
        policy.parse(&mut set, &mut inv, args)?;
        Ok(())
    }
}
