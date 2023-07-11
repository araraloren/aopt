use std::fmt::Debug;
use std::marker::PhantomData;

use super::process_callback_ret;
use super::process_non_opt;
use super::process_opt;
use super::CtxSaver;
use super::Guess;
use super::GuessNOACfg;
use super::GuessOptCfg;
use super::NOAGuess;
use super::OptGuess;
use super::OptStyleManager;
use super::Policy;
use super::PolicySettings;
use super::ProcessCtx;
use super::ReturnVal;
use super::UserStyle;
use crate::args::ArgParser;
use crate::args::Args;
use crate::astr;
use crate::ctx::Ctx;
use crate::ctx::Invoker;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::parser::FailManager;
use crate::prelude::HandlerCollection;
use crate::prelude::SetExt;
use crate::proc::Process;
use crate::set::OptValidator;
use crate::set::SetChecker;
use crate::set::SetOpt;
use crate::trace_log;
use crate::ARef;
use crate::Error;
use crate::Str;

/// [`DelayPolicy`] matching the command line arguments with [`Opt`] in the [`Set`](crate::set::Set).
/// The option would match failed if any special [`Error`] raised during option processing.
/// [`DelayPolicy`] will return Some([`ReturnVal`]) if match successful.
/// [`DelayPolicy`] processes the option first but does not invoke the handler of option.
/// The handler will be called after [`Cmd`](crate::opt::Style::Cmd) NOA and [`Pos`](crate::opt::Style::Pos) NOA processed.
/// In last, [`DelayPolicy`] will process [`Main`](crate::opt::Style::Main) NOA.
/// During parsing, you can't get the value of any option in the handler of NOA.
///
/// # Example
/// ```rust
/// # use aopt::getopt;
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// # use std::path::PathBuf;
/// #
/// # fn main() -> Result<(), Error> {
/// let filter = |f: fn(&PathBuf) -> bool| {
///     move |set: &mut ASet, _: &mut ASer| {
///         set["directory"].filter::<PathBuf>(f)?;
///         Ok(Some(true))
///     }
/// };
///
/// let mut parser = ADelayParser::default();
///
/// // POS will be process first, get the items under given directory
/// parser
///     .add_opt("directory=p@1")?
///     .set_pos_type::<PathBuf>()
///     .on(|_: &mut ASet, _: &mut ASer, path: ctx::Value<PathBuf>| {
///         Ok(Some(
///             path.read_dir()
///                 .map_err(|e| {
///                     aopt::raise_failure!("Can not read directory {:?}: {:?}", path, e)
///                 })?
///                 .map(|v| v.unwrap().path())
///                 .collect::<Vec<PathBuf>>(),
///         ))
///     })?
///     .then(VecStore);
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
///     .on(move |set: &mut ASet, _: &mut ASer| {
///         if let Ok(vals) = set["directory"].vals::<PathBuf>() {
///             for val in vals {
///                 println!("{:?}", val);
///             }
///         }
///         Ok(Some(true))
///     })?;
///
/// getopt!(Args::from_env(), &mut parser)?;
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct DelayPolicy<Set, Ser, Chk> {
    strict: bool,

    contexts: Vec<CtxSaver>,

    checker: Chk,

    style_manager: OptStyleManager,

    no_delay_opt: Vec<Str>,

    marker_s: PhantomData<(Set, Ser)>,
}

impl<Set, Ser, Chk> Debug for DelayPolicy<Set, Ser, Chk>
where
    Chk: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelayPolicy")
            .field("strict", &self.strict)
            .field("contexts", &self.contexts)
            .field("checker", &self.checker)
            .field("style_manager", &self.style_manager)
            .field("no_delay_opt", &self.no_delay_opt)
            .finish()
    }
}

impl<Set, Ser, Chk> Default for DelayPolicy<Set, Ser, Chk>
where
    Chk: Default,
{
    fn default() -> Self {
        Self {
            strict: true,
            contexts: vec![],
            checker: Chk::default(),
            style_manager: OptStyleManager::default(),
            no_delay_opt: vec![],
            marker_s: PhantomData::default(),
        }
    }
}

impl<Set, Ser, Chk> DelayPolicy<Set, Ser, Chk>
where
    Chk: Default,
{
    pub fn new(strict: bool, styles: OptStyleManager) -> Self {
        Self {
            strict,
            style_manager: styles,
            ..Self::default()
        }
    }
}

impl<Set, Ser, Chk> DelayPolicy<Set, Ser, Chk> {
    /// Enable strict mode, if argument is an option, it must be matched.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    pub fn with_styles(mut self, styles: Vec<UserStyle>) -> Self {
        self.style_manager.set(styles);
        self
    }

    pub fn with_no_delay(mut self, name: impl Into<Str>) -> Self {
        self.no_delay_opt.push(name.into());
        self
    }

    pub fn with_checker(mut self, checker: Chk) -> Self {
        self.checker = checker;
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

impl<Set, Ser, Chk> PolicySettings for DelayPolicy<Set, Ser, Chk> {
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

    fn no_delay(&self) -> Option<&[Str]> {
        Some(&self.no_delay_opt)
    }

    fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.strict = strict;
        self
    }

    fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.style_manager.set(styles);
        self
    }

    fn set_no_delay(&mut self, name: impl Into<Str>) -> &mut Self {
        self.no_delay_opt.push(name.into());
        self
    }
}

impl<Set, Ser, Chk> DelayPolicy<Set, Ser, Chk>
where
    SetOpt<Set>: Opt,
    Chk: SetChecker<Set>,
    Set: crate::set::Set + OptParser,
{
    // ignore failure
    pub fn invoke_opt_callback<'a, Inv>(
        &mut self,
        ctx: &mut Ctx,
        set: &mut Set,
        inv: &mut Inv,
        ser: &mut Ser,
        manager: &mut FailManager,
        saver: CtxSaver,
    ) -> Result<(), Error>
    where
        Inv: HandlerCollection<'a, Set, Ser>,
    {
        let uid = saver.uid;
        let fail = |e: &Error| {
            manager.push(e.clone());
            Ok(())
        };

        ctx.set_inner_ctx(Some(saver.ctx));
        if !process_callback_ret(inv.invoke_fb(&uid, set, ser, ctx), |_| Ok(()), fail)? {
            set.opt_mut(uid)?.set_matched(false);
        }
        Ok(())
    }

    pub fn save_or_call<'a, Inv>(
        &mut self,
        ctx: &mut Ctx,
        set: &mut Set,
        inv: &mut Inv,
        ser: &mut Ser,
        manager: &mut FailManager,
        saver: CtxSaver,
    ) -> Result<(), Error>
    where
        Inv: HandlerCollection<'a, Set, Ser>,
    {
        let name = set.opt(saver.uid)?.name();

        if self.no_delay_opt.contains(name) {
            return self.invoke_opt_callback(ctx, set, inv, ser, manager, saver);
        } else {
            self.contexts.push(saver);
        }

        Ok(())
    }
}

impl<Set, Ser, Chk> DelayPolicy<Set, Ser, Chk>
where
    SetOpt<Set>: Opt,
    Chk: SetChecker<Set>,
    Set: crate::set::Set + OptParser + OptValidator,
{
    pub(crate) fn parse_impl(
        &mut self,
        ctx: &mut Ctx,
        set: &mut <Self as Policy>::Set,
        inv: &mut <Self as Policy>::Inv<'_>,
        ser: &mut <Self as Policy>::Ser,
    ) -> Result<(), <Self as Policy>::Error> {
        self.checker().pre_check(set).map_err(|e| e.into())?;

        // take the invoke service, avoid borrow the ser
        let opt_styles = self.style_manager.clone();
        let args = ctx.orig_args().clone();
        let args_len = args.len();
        let mut noa_args = Args::default();
        let mut iter = args.guess_iter().enumerate();
        let mut opt_fail = FailManager::default();

        trace_log!("Parsing {ctx:?} using delay policy");
        // set option args, and args length
        ctx.set_args(args.clone());
        while let Some((idx, (opt, arg))) = iter.next() {
            let mut matched = false;
            let mut consume = false;
            let arg = arg.map(|v| ARef::new(v.clone()));

            // parsing current argument
            if let Ok(clopt) = opt.parse_arg() {
                if let Some(name) = clopt.name() {
                    if set.check(name.as_str()).map_err(Into::into)? {
                        for style in opt_styles.iter() {
                            if let Some(mut proc) = OptGuess::new().guess(
                                style,
                                GuessOptCfg::new(idx, args_len, arg.clone(), &clopt, set),
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
                                    &mut opt_fail,
                                    false,
                                )?;

                                if proc.status() {
                                    for saver in ret {
                                        self.save_or_call(
                                            ctx,
                                            set,
                                            inv,
                                            ser,
                                            &mut opt_fail,
                                            saver,
                                        )?;
                                    }
                                    matched = true;
                                }
                                if matched {
                                    if proc.is_consume() {
                                        consume = true;
                                    }
                                    break;
                                }
                            }
                        }
                        if !matched && self.strict() {
                            let default_str = astr("");

                            return Err(opt_fail.cause(Error::sp_option_not_found(format!(
                                "{}",
                                clopt.name().unwrap_or(&default_str)
                            ))));
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

        let noa_args = ARef::new(noa_args);
        let noa_len = noa_args.len();
        let mut pos_fail = FailManager::default();
        let mut cmd_fail = FailManager::default();

        ctx.set_args(noa_args.clone());
        // when style is pos, noa index is [1..=len]
        if noa_len > 0 {
            if let Some(mut proc) = NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(noa_args.clone(), Self::noa_cmd(), noa_len),
            )? {
                process_non_opt(
                    ProcessCtx {
                        ctx,
                        set,
                        inv,
                        ser,
                        tot: noa_len,
                        idx: Self::noa_cmd(),
                    },
                    &mut proc,
                    &mut cmd_fail,
                )?;
            }
            cmd_fail.process(self.checker().cmd_check(set))?;
            for idx in 1..noa_len {
                if let Some(mut proc) = NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), Self::noa_pos(idx), noa_len),
                )? {
                    process_non_opt(
                        ProcessCtx {
                            ctx,
                            set,
                            inv,
                            ser,
                            tot: noa_len,
                            idx: Self::noa_pos(idx),
                        },
                        &mut proc,
                        &mut pos_fail,
                    )?;
                }
            }
        } else {
            cmd_fail.process(self.checker().cmd_check(set))?;
        }

        // after cmd and pos callback invoked, invoke the callback of option
        for saver in std::mem::take(&mut self.contexts) {
            self.invoke_opt_callback(ctx, set, inv, ser, &mut opt_fail, saver)?;
        }

        opt_fail.process(self.checker().opt_check(set))?;
        pos_fail.process(self.checker().pos_check(set))?;

        let main_args = noa_args;
        let main_len = main_args.len();
        let mut main_fail = FailManager::default();

        ctx.set_args(main_args.clone());
        if let Some(mut proc) = NOAGuess::new().guess(
            &UserStyle::Main,
            GuessNOACfg::new(main_args, Self::noa_main(), main_len),
        )? {
            process_non_opt(
                ProcessCtx {
                    ctx,
                    set,
                    inv,
                    ser,
                    tot: main_len,
                    idx: Self::noa_main(),
                },
                &mut proc,
                &mut main_fail,
            )?;
        }
        main_fail.process(self.checker().post_check(set))?;
        Ok(())
    }
}

impl<Set, Ser, Chk> Policy for DelayPolicy<Set, Ser, Chk>
where
    SetOpt<Set>: Opt,
    Chk: SetChecker<Set>,
    Set: crate::set::Set + OptParser + OptValidator,
{
    type Ret = ReturnVal;

    type Set = Set;

    type Inv<'a> = Invoker<'a, Set, Ser>;

    type Ser = Ser;

    type Error = Error;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv<'_>,
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

    use crate::opt::Pos;
    use crate::prelude::*;
    use crate::ARef;
    use crate::Error;
    use std::any::TypeId;
    use std::ops::Deref;

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
                "type id not equal for {}({})",
                opt_uid,
                opt.name()
            );
            assert_eq!(opt.index(), index, "option index not equal: {:?}", index);
            if let Ok(opt_vals) = opt.vals::<T>() {
                if let Some(vals) = vals {
                    assert_eq!(
                        opt_vals.len(),
                        vals.len(),
                        "value length not equal for -{}- : {:?} != {:?}",
                        opt_uid,
                        opt_vals,
                        vals,
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

        let args = Args::from_array([
            "app",
            "filter",
            "+>",
            "foo",
            "bar",
            "--no-delay",
            "8",
            "42",
            "--option-ignored",
            "88",
            "+>",
            "12.5",
            "lily",
            "66",
            "11",
        ]);

        set.validator_mut().add_prefix("+");

        set.add_opt("set=c")?;
        set.add_opt("filter=c")?;

        let args_uid = set
            .add_opt("args=p@2..")?
            .set_pos_type_only::<f64>()
            .run()?;

        inv.entry(set.add_opt_i::<bool>("--no-delay")?.run()?).on(
            |set: &mut ASet, _: &mut ASer| {
                assert_eq!(set["filter"].val::<bool>()?, &false);
                Ok(Some(true))
            },
        );
        policy.set_no_delay("--no-delay");

        inv.entry(set.add_opt("--positive=b")?.add_alias("+>").run()?)
            .on(|set: &mut ASet, _: &mut ASer| {
                set["args"].filter::<f64>(|v: &f64| v <= &0.0)?;
                Ok(Some(true))
            });
        inv.entry(set.add_opt("--bigger-than=f")?.add_alias("+>").run()?)
            .on(|set: &mut ASet, _: &mut ASer, val: ctx::Value<f64>| {
                assert_eq!(set["filter"].val::<bool>()?, &true);
                // this is a vec![vec![], ..]
                Ok(Some(set["args"].filter::<f64>(|v: &f64| v <= val.deref())?))
            });
        inv.entry(set.add_opt("main=m")?.run()?).on(
            move |set: &mut ASet, _: &mut ASer, app: ctx::Value<String>| {
                let args = &set["args"];
                let bopt = &set["--bigger-than"];

                assert_eq!(app.deref(), "app");
                check_opt_val::<f64>(
                    args,
                    args_uid,
                    "args",
                    Some(vec![42.0, 88.0, 66.0]),
                    false,
                    &Action::App,
                    &TypeId::of::<Pos<f64>>(),
                    Some(&Index::Range(2, None)),
                    None,
                )?;
                check_opt_val::<Vec<f64>>(
                    bopt,
                    bopt.uid(),
                    "--bigger-than",
                    Some(vec![vec![8.0, 11.0]]),
                    false,
                    &Action::App,
                    &TypeId::of::<f64>(),
                    None,
                    None,
                )?;
                Ok(Some(()))
            },
        );

        let args = ARef::new(args);

        for opt in set.iter_mut() {
            opt.init()?;
        }
        assert!(!policy
            .parse(&mut set, &mut inv, &mut ser, args.clone())?
            .status());
        policy.set_strict(false);
        for opt in set.iter_mut() {
            opt.init()?;
        }
        assert!(policy.parse(&mut set, &mut inv, &mut ser, args)?.status());
        Ok(())
    }
}
