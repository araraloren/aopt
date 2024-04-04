use std::fmt::Debug;
use std::marker::PhantomData;

use crate::args::ArgParser;
use crate::args::Args;
use crate::ctx::Ctx;
use crate::ctx::Invoker;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::parser::OptStyleManager;
use crate::parser::Policy;
use crate::parser::PolicySettings;
use crate::parser::ReturnVal;
use crate::parser::UserStyle;
use crate::set::OptValidator;
use crate::set::SetOpt;
use crate::shell::CompleteGuess;
use crate::ARef;
use crate::AStr;
use crate::Error;

pub struct CompletePolicy<Set, Ser> {
    strict: bool,

    process_pos: bool,

    style_manager: OptStyleManager,

    marker_s: PhantomData<(Set, Ser)>,
}

impl<Set, Ser> Clone for CompletePolicy<Set, Ser> {
    fn clone(&self) -> Self {
        Self {
            strict: self.strict,
            process_pos: self.process_pos,
            style_manager: self.style_manager.clone(),
            marker_s: self.marker_s,
        }
    }
}

impl<Set, Ser> Debug for CompletePolicy<Set, Ser> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FwdPolicy")
            .field("strict", &self.strict)
            .field("process_pos", &self.process_pos)
            .field("style_manager", &self.style_manager)
            .finish()
    }
}

impl<Set, Ser> Default for CompletePolicy<Set, Ser> {
    fn default() -> Self {
        Self {
            strict: true,
            process_pos: false,
            style_manager: OptStyleManager::default(),
            marker_s: PhantomData,
        }
    }
}

impl<Set, Ser> CompletePolicy<Set, Ser> {
    pub fn new(strict: bool, style: OptStyleManager) -> Self {
        Self {
            strict,
            style_manager: style,
            ..Default::default()
        }
    }
}

impl<Set, Ser> CompletePolicy<Set, Ser> {
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

    pub fn with_process_pos(mut self, process_pos: bool) -> Self {
        self.process_pos = process_pos;
        self
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

    pub fn process_pos(&self) -> bool {
        self.process_pos
    }
}

impl<Set, Ser> PolicySettings for CompletePolicy<Set, Ser> {
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

    fn no_delay(&self) -> Option<&[AStr]> {
        None
    }

    fn overload(&self) -> bool {
        false
    }

    fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.strict = strict;
        self
    }

    fn set_styles(&mut self, styles: Vec<UserStyle>) -> &mut Self {
        self.style_manager.set(styles);
        self
    }

    fn set_no_delay(&mut self, _: impl Into<AStr>) -> &mut Self {
        self
    }

    fn set_overload(&mut self, _: bool) -> &mut Self {
        self
    }
}

impl<Set, Ser> CompletePolicy<Set, Ser>
where
    SetOpt<Set>: Opt,
    Set: crate::set::Set + OptParser + OptValidator,
{
    pub(crate) fn parse_impl(
        &mut self,
        ctx: &mut Ctx,
        set: &mut <Self as Policy>::Set,
        inv: &mut <Self as Policy>::Inv<'_>,
        ser: &mut <Self as Policy>::Ser,
    ) -> Result<(), <Self as Policy>::Error> {
        let opt_styles = &self.style_manager;
        let args = ctx.orig_args().clone();
        let tot = args.len();
        let mut noa_args = Args::default();
        let mut iter = args.guess_iter().enumerate();

        ctx.set_args(args.clone());
        while let Some((idx, (opt, next))) = iter.next() {
            let mut matched = false;
            let mut consume = false;

            if let Ok(clopt) = opt.parse_arg() {
                let name = clopt.name;

                if set.check(name.as_str()).map_err(Into::into)? {
                    let arg = clopt.value;
                    let mut guess = CompleteGuess {
                        idx,
                        arg,
                        set,
                        inv,
                        ser,
                        tot,
                        ctx,
                        next: next.cloned(),
                        name: Some(name.clone()),
                    };

                    for style in opt_styles.iter() {
                        if let Some(ret) = guess.guess_complete(style)? {
                            (matched, consume) = (ret.matched, ret.consume);
                        }
                        if matched {
                            break;
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
        let tot = noa_args.len();

        ctx.set_args(noa_args.clone());
        // when style is pos, noa index is [1..=len]
        if tot > 0 {
            let name = noa_args
                .get(Self::noa_cmd())
                .and_then(|v| v.get_str())
                .map(AStr::from);
            let mut guess = CompleteGuess {
                set,
                inv,
                ser,
                tot,
                name,
                ctx,
                arg: None,
                next: None,
                idx: Self::noa_cmd(),
            };

            guess.guess_complete(&UserStyle::Cmd)?;

            if self.process_pos {
                let mut guess = CompleteGuess {
                    set,
                    inv,
                    ser,
                    tot,
                    ctx,
                    name: None,
                    arg: None,
                    next: None,
                    idx: Self::noa_cmd(),
                };

                for idx in 1..tot {
                    guess.idx = Self::noa_pos(idx);
                    guess.name = noa_args
                        .get(Self::noa_pos(idx))
                        .and_then(|v| v.get_str())
                        .map(AStr::from);
                    guess.guess_complete(&UserStyle::Pos)?;
                }
            }
        }

        let main_args = noa_args;
        let tot = main_args.len();

        ctx.set_args(main_args.clone());
        let name = main_args
            .get(Self::noa_main())
            .and_then(|v| v.get_str())
            .map(AStr::from);
        let mut guess = CompleteGuess {
            set,
            inv,
            ser,
            tot,
            name,
            ctx,
            arg: None,
            next: None,
            idx: Self::noa_main(),
        };

        guess.guess_complete(&UserStyle::Main)?;
        Ok(())
    }
}

impl<Set, Ser> Policy for CompletePolicy<Set, Ser>
where
    SetOpt<Set>: Opt,
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
