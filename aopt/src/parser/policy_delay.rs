use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;

use super::OptStyleManager;
use super::Policy;
use super::PolicySettings;
use super::Return;
use super::UserStyle;
use crate::args;
use crate::args::ArgInfo;
use crate::args::Args;
use crate::ctx::Ctx;
use crate::ctx::HandlerCollection;
use crate::ctx::InnerCtx;
use crate::ctx::Invoker;
use crate::guess::process_handler_ret;
use crate::guess::InnerCtxSaver;
use crate::guess::InvokeGuess;
use crate::guess::SimpleMatRet;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::parser::Action;
use crate::parser::FailManager;
use crate::set::OptValidator;
use crate::set::SetChecker;
use crate::set::SetExt;
use crate::set::SetOpt;
use crate::trace;
use crate::Error;
use crate::Uid;

#[derive(Debug, Clone, Default)]
pub struct DelayCtx<'a> {
    pub uids: Vec<Uid>,

    pub matched: Vec<Option<bool>>,

    pub inner_ctx: InnerCtx<'a>,
}

#[derive(Debug, Clone, Default)]
pub struct DelayCtxSaver<'a> {
    pub any_match: bool,

    pub consume: bool,

    pub delay_ctx: Vec<DelayCtx<'a>>,
}

/// [`DelayPolicy`] matching the command line arguments with [`Opt`] in the [`Set`](crate::set::Set).
/// The option would match failed if any special [`Error`] raised during option processing.
/// [`DelayPolicy`] will return Some([`Return`]) if match successful.
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
/// fn filter( f: fn(&PathBuf) -> bool)
///     -> impl Fn(&mut AHCSet, &mut Ctx) -> Result<Option<bool>, Error> {
///    move |set: &mut AHCSet, _| {
///        set["directory"].filter::<PathBuf>(f)?;
///        Ok(Some(true))
///    }
/// }
///
/// let mut parser = ADelayParser::default();
///
/// // POS will be process first, get the items under given directory
/// parser
///     .add_opt("directory=p@1")?
///     .set_pos_type::<PathBuf>()
///     .on(|_, ctx| {
///         let path = ctx.value::<PathBuf>()?;
///
///         Ok(Some(
///             path.read_dir()
///                 .map_err(|e| {
///                     aopt::failure!("Can not read directory {:?}: {:?}", path, e)
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
///     .on(move |set, _| {
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
pub struct DelayPolicy<S, Chk> {
    strict: bool,

    overload: bool,

    prepolicy: bool,

    checker: Chk,

    style_manager: OptStyleManager,

    no_delay_opt: Vec<String>,

    marker_s: PhantomData<S>,
}

impl<S, Chk> Clone for DelayPolicy<S, Chk>
where
    Chk: Clone,
{
    fn clone(&self) -> Self {
        Self {
            strict: self.strict,
            overload: self.overload,
            prepolicy: self.prepolicy,
            checker: self.checker.clone(),
            style_manager: self.style_manager.clone(),
            no_delay_opt: self.no_delay_opt.clone(),
            marker_s: self.marker_s,
        }
    }
}

impl<S, Chk> Debug for DelayPolicy<S, Chk>
where
    Chk: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelayPolicy")
            .field("strict", &self.strict)
            .field("overload", &self.overload)
            .field("prepolicy", &self.prepolicy)
            .field("checker", &self.checker)
            .field("style_manager", &self.style_manager)
            .field("no_delay_opt", &self.no_delay_opt)
            .finish()
    }
}

impl<S, Chk> Default for DelayPolicy<S, Chk>
where
    Chk: Default,
{
    fn default() -> Self {
        Self {
            strict: true,
            overload: false,
            prepolicy: false,
            checker: Chk::default(),
            style_manager: OptStyleManager::default(),
            no_delay_opt: vec![],
            marker_s: PhantomData,
        }
    }
}

impl<S, Chk> DelayPolicy<S, Chk>
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

impl<S, Chk> DelayPolicy<S, Chk> {
    /// Enable strict mode, if argument is an option, it must be matched.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    pub fn with_styles(mut self, styles: Vec<UserStyle>) -> Self {
        self.style_manager.set(styles);
        self
    }

    pub fn with_no_delay(mut self, name: impl Into<String>) -> Self {
        self.no_delay_opt.push(name.into());
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

    pub fn with_prepolicy(mut self, prepolicy: bool) -> Self {
        self.prepolicy = prepolicy;
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

    pub(crate) fn filter<T, E: Into<Error>>(
        prepolicy: bool,
        res: Result<T, E>,
    ) -> Result<Option<T>, Error> {
        let res = res.map_err(Into::into);

        if !prepolicy {
            res.map(|v| Some(v))
        } else {
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
}

impl<S, Chk> PolicySettings for DelayPolicy<S, Chk> {
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
        Some(&self.no_delay_opt)
    }

    fn overload(&self) -> bool {
        self.overload
    }

    fn prepolicy(&self) -> bool {
        self.prepolicy
    }

    fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.strict = strict;
        self
    }

    fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.style_manager.set(styles);
        self
    }

    fn set_no_delay(&mut self, name: impl Into<String>) -> &mut Self {
        self.no_delay_opt.push(name.into());
        self
    }

    fn set_overload(&mut self, overload: bool) -> &mut Self {
        self.overload = overload;
        self
    }

    fn set_prepolicy(&mut self, prepolicy: bool) -> &mut Self {
        self.prepolicy = prepolicy;
        self
    }
}

impl<S, Chk> DelayPolicy<S, Chk>
where
    SetOpt<S>: Opt,
    Chk: SetChecker<S>,
    S: crate::set::Set + OptParser,
{
    // ignore failure
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn invoke_opt_callback<'a, 'b, Inv>(
        &mut self,
        uid: Uid,
        ctx: &mut Ctx<'a>,
        set: &mut S,
        inv: &mut Inv,
        fail: &mut FailManager,
        inner_ctx: InnerCtx<'a>,
    ) -> Result<bool, Error>
    where
        Inv: HandlerCollection<'b, S>,
    {
        let fail = |e: Error| {
            fail.push(e);
            Ok(())
        };

        ctx.set_inner_ctx(Some(inner_ctx.with_uid(uid)));
        let ret = process_handler_ret(inv.invoke_fb(&uid, set, ctx), |_| Ok(()), fail)?;

        set.opt_mut(uid)?.set_matched(ret);
        Ok(ret)
    }

    pub fn process_delay_ctx<'a, 'b, Inv>(
        &mut self,
        ctx: &mut Ctx<'a>,
        set: &mut S,
        inv: &mut Inv,
        fail: &mut FailManager,
        saver: DelayCtxSaver<'a>,
    ) -> Result<SimpleMatRet, Error>
    where
        Inv: HandlerCollection<'b, S>,
    {
        let any_match = saver.any_match;
        let consume = saver.consume;

        for delay_ctx in saver.delay_ctx {
            let inner_ctx = delay_ctx.inner_ctx;
            let mut matched = false;

            trace!("invoke the handler: Inner = {:?}", &inner_ctx);
            for (uid, cache_matched) in delay_ctx.uids.iter().zip(delay_ctx.matched.iter()) {
                let ret = if let Some(cache_matched) = cache_matched {
                    *cache_matched
                } else {
                    self.invoke_opt_callback(
                        *uid,
                        ctx,
                        set,
                        inv,
                        fail,
                        inner_ctx.clone().with_uid(*uid),
                    )?
                };

                // if it matched,
                // so the policy_inner_ctx matched
                // and inner_ctx_saver matched,
                // should return immediately
                if any_match && ret {
                    return Ok(SimpleMatRet::new(true, consume));
                }
                matched = matched || ret;
            }
            if !any_match && !matched {
                return Ok(SimpleMatRet::new(false, false));
            }
        }
        Ok(SimpleMatRet::new(true, consume))
    }

    pub fn save_or_call<'a, 'b, 'c, Inv>(
        &mut self,
        guess: &mut InvokeGuess<'a, 'b, S, Inv>,
        saver: InnerCtxSaver<'b>,
        contexts: &mut Vec<DelayCtxSaver<'b>>,
    ) -> Result<Option<SimpleMatRet>, Error>
    where
        Inv: HandlerCollection<'c, S>,
    {
        let any_match = saver.any_match;
        let consume = saver.consume;
        let mut delay_ctx = vec![];

        for policy in saver.policy_ctx {
            let len = policy.uids.len();
            let inner_ctx = policy.inner_ctx.clone();
            let mut matched = Vec::with_capacity(len);

            for uid in policy.uids.iter() {
                let name = guess.set.opt(*uid)?.name();

                if self.no_delay_opt.iter().any(|v| v == name) {
                    let ret = self.invoke_opt_callback(
                        *uid,
                        guess.ctx,
                        guess.set,
                        guess.inv,
                        guess.fail,
                        inner_ctx.clone().with_uid(*uid),
                    )?;

                    // if it matched,
                    // so the policy_inner_ctx matched
                    // and inner_ctx_saver matched,
                    // should return immediately
                    if any_match && ret {
                        return Ok(Some(SimpleMatRet::new(true, consume)));
                    } else {
                        matched.push(Some(ret));
                    }
                } else {
                    matched.push(None);
                }
            }
            if !any_match && matched.iter().all(|v| v == &Some(false)) {
                return Ok(Some(SimpleMatRet::new(false, false)));
            } else {
                delay_ctx.push(DelayCtx {
                    uids: policy.uids,
                    matched,
                    inner_ctx: policy.inner_ctx,
                });
            }
        }
        if !delay_ctx.is_empty() {
            contexts.push(DelayCtxSaver {
                any_match,
                consume,
                delay_ctx,
            })
        }
        Ok(None)
    }
}

impl<S, Chk> DelayPolicy<S, Chk>
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
        let pre = self.prepolicy();
        let opt_styles = self.style_manager.clone();
        let args: Vec<_> = orig.iter().map(|v| v.as_os_str()).collect();
        let total = args.len();
        let mut contexts: Vec<DelayCtxSaver> = vec![];
        let mut lefts = vec![];
        let mut opt_fail = FailManager::default();
        let mut iter2 = args::iter2(&args).enumerate();

        trace!("parsing {ctx:?} using delay policy");
        // set option args, and args length
        ctx.set_args(args.clone());
        while let Some((idx, (opt, next))) = iter2.next() {
            let mut matched = false;
            let mut consume = false;
            let mut stopped = false;
            let mut like_opt = false;

            // parsing current argument
            if let Ok(ArgInfo { name, value }) = ArgInfo::parse(opt) {
                trace!(
                    "guess name: {:?} value: {:?} & next: {:?}",
                    name,
                    value,
                    next
                );
                if let Some(true) = Self::filter(pre, set.check(&name))? {
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

                    like_opt = true;
                    for style in opt_styles.iter() {
                        if let Some(Some(ret)) =
                            Self::filter(pre, guess.guess_and_collect(style, overload))?
                        {
                            // pretend we are matched, cause it is delay
                            matched = true;
                            consume = ret.consume;
                            if let Some(ret) = self.save_or_call(&mut guess, ret, &mut contexts)? {
                                // if the call returned, set the real return value
                                (matched, consume) = (ret.matched, ret.consume);
                            }
                            if matched {
                                match guess.ctx.policy_act() {
                                    Action::Stop => {
                                        stopped = true;
                                        guess.ctx.reset_policy_act();
                                        break;
                                    }
                                    Action::Quit => return Ok(()),
                                    Action::Null => {}
                                }
                                break;
                            }
                        }
                    }
                    if !pre && !stopped && !matched && self.strict() {
                        return Err(opt_fail.cause(Error::sp_not_found(name)));
                    }
                }
                if !like_opt {
                    trace!("`{:?}` not like option", opt);
                }
            }
            if stopped {
                // skip current, put left argument to noa args
                lefts.extend(iter2.map(|(_, (a, _))| *a));
                break;
            }
            // if consume the argument, skip it
            if matched && consume {
                iter2.next();
            } else if !matched {
                // add it to NOA if current argument not matched
                lefts.push(*opt);
            }
        }

        let args = lefts;
        let total = args.len();
        let mut pos_fail = FailManager::default();
        let mut cmd_fail = FailManager::default();
        let mut prev_ctx = ctx.clone();

        ctx.set_args(args.clone());
        // when style is pos, noa index is [1..=len]
        if total > 0 {
            let name = crate::str::osstr_to_str_i(&args, Self::noa_cmd());
            let mut guess = InvokeGuess {
                set,
                inv,
                total,
                name,
                ctx,
                arg: None,
                next: None,
                fail: &mut cmd_fail,
                idx: Self::noa_cmd(),
            };

            trace!("guess Cmd = {:?}", guess.name);
            Self::filter(pre, guess.guess_and_invoke(&UserStyle::Cmd, overload))?;
            if let Action::Quit = ctx.policy_act() {
                return Ok(());
            }
            cmd_fail.process_check(self.checker().cmd_check(set))?;

            let mut guess = InvokeGuess {
                set,
                inv,
                total,
                ctx,
                name: None,
                arg: None,
                next: None,
                fail: &mut pos_fail,
                idx: Self::noa_cmd(),
            };

            for idx in 1..total {
                guess.idx = Self::noa_pos(idx);
                guess.name = crate::str::osstr_to_str_i(&args, Self::noa_pos(idx));
                trace!("guess Pos argument = {:?} @ {}", guess.name, guess.idx);
                Self::filter(pre, guess.guess_and_invoke(&UserStyle::Pos, overload))?;
                match guess.ctx.policy_act() {
                    Action::Stop => {
                        guess.ctx.reset_policy_act();
                        break;
                    }
                    Action::Quit => return Ok(()),
                    Action::Null => {}
                }
            }
        } else {
            cmd_fail.process_check(self.checker().cmd_check(set))?;
        }

        trace!("in delay policy, invoke the handler of option");
        // after cmd and pos callback invoked, invoke the callback of option
        for saver in contexts {
            if let Some(ret) = Self::filter(
                pre,
                self.process_delay_ctx(&mut prev_ctx, set, inv, &mut opt_fail, saver),
            )? {
                if ret.matched {
                    match prev_ctx.policy_act() {
                        Action::Stop => {
                            prev_ctx.reset_policy_act();
                            break;
                        }
                        Action::Quit => return Ok(()),
                        Action::Null => {}
                    }
                }
                if !pre && !ret.matched && self.strict() {
                    return Err(
                        opt_fail.cause(crate::error!("option match failed, Ctx = {:?}", prev_ctx))
                    );
                }
            }
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
        Self::filter(pre, guess.guess_and_invoke(&UserStyle::Main, overload))?;
        main_fail.process_check(self.checker().post_check(set))?;
        Ok(())
    }
}

impl<S, Chk> Policy for DelayPolicy<S, Chk>
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

    use crate::opt::ConfigBuildInfer;
    use crate::opt::Pos;
    use crate::prelude::*;
    use crate::Error;
    use std::any::TypeId;

    #[test]
    fn testing_1() {
        assert!(testing_prepolicy().is_ok());
        assert!(testing_non_prepolicy().is_ok());
    }

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

    fn testing_non_prepolicy() -> Result<(), Error> {
        let mut policy = ADelayPolicy::default();
        let mut inv = AInvoker::default();
        let mut set = AHCSet::default();

        let args = Args::from([
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

        inv.entry(set.add_opt("--no-delay".infer::<bool>())?.run()?)
            .on(|set, _| {
                assert_eq!(set["filter"].val::<bool>()?, &false);
                Ok(Some(true))
            });
        policy.set_no_delay("--no-delay");

        inv.entry(set.add_opt("--positive=b")?.add_alias("+>").run()?)
            .on(|set, _| {
                set["args"].filter::<f64>(|v: &f64| v <= &0.0)?;
                Ok(Some(true))
            });
        inv.entry(set.add_opt("--bigger-than=f")?.add_alias("+>").run()?)
            .on(|set, ctx| {
                let val = ctx.value::<f64>()?;
                assert_eq!(set["filter"].val::<bool>()?, &true);
                // this is a vec![vec![], ..]
                Ok(Some(set["args"].filter::<f64>(|v: &f64| v <= &val)?))
            });
        inv.entry(set.add_opt("main=m")?.run()?)
            .on(move |set, ctx| {
                let args = &set["args"];
                let bopt = &set["--bigger-than"];
                let app = ctx.value::<String>()?;

                assert_eq!(app, "app");
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
            });

        for opt in set.iter_mut() {
            opt.init()?;
        }
        assert!(!policy.parse(&mut set, &mut inv, args.clone())?.status());
        policy.set_strict(false);
        for opt in set.iter_mut() {
            opt.init()?;
        }
        assert!(policy.parse(&mut set, &mut inv, args)?.status());
        Ok(())
    }

    fn testing_prepolicy() -> Result<(), Error> {
        let mut policy = ADelayPolicy::default().with_prepolicy(true);
        let mut inv = AInvoker::default();
        let mut set = AHCSet::default();

        set.validator_mut().add_prefix("+");

        set.add_opt("set=c")?;
        set.add_opt("filter=c")?;

        let args_uid = set
            .add_opt("args=p@2..")?
            .set_pos_type_only::<f64>()
            .set_values(vec![])
            .run()?;

        inv.entry(set.add_opt("--no-delay".infer::<bool>())?.run()?)
            .on(|set, _| {
                assert_eq!(set["filter"].val::<bool>()?, &false);
                Ok(Some(true))
            });
        policy.set_no_delay("--no-delay");

        inv.entry(set.add_opt("--positive=b")?.add_alias("+>").run()?)
            .on(|set, _| {
                set["args"].filter::<f64>(|v: &f64| v <= &0.0)?;
                Ok(Some(true))
            });
        inv.entry(set.add_opt("--bigger-than=f")?.add_alias("+>").run()?)
            .on(|set, ctx| {
                let val = ctx.value::<f64>()?;
                assert_eq!(set["filter"].val::<bool>()?, &true);
                // this is a vec![vec![], ..]
                Ok(Some(set["args"].filter::<f64>(|v: &f64| v <= &val)?))
            });
        inv.entry(set.add_opt("main=m")?.run()?)
            .on(move |set, ctx| {
                let args = &set["args"];
                let bopt = &set["--bigger-than"];
                let app = ctx.value::<String>()?;

                assert_eq!(app, "app");
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
            });

        let args = Args::from([
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

        set.init()?;
        let ret = policy.parse(&mut set, &mut inv, args.clone());

        assert!(ret.is_ok());
        let ret = ret.unwrap();
        let next_args = ret.args();

        assert_eq!(
            [
                "app",
                "filter",
                "bar",
                "8",
                "42",
                "--option-ignored",
                "88",
                "lily",
                "66",
                "11",
            ],
            next_args
        );

        // init
        set.init()?;
        policy.set_prepolicy(false);
        // we need clear the user define value in value storer
        let _ = set["--bigger-than"].take_vals::<Vec<f64>>();
        assert!(!policy.parse(&mut set, &mut inv, args)?.status());
        Ok(())
    }
}
