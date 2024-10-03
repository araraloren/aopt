use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::args;
use crate::args::ArgInfo;
use crate::args::Args;
use crate::ctx::Ctx;
use crate::ctx::Invoker;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::parser::OptStyleManager;
use crate::parser::Policy;
use crate::parser::PolicySettings;
use crate::parser::Return;
use crate::parser::UserStyle;
use crate::set::OptValidator;
use crate::set::SetOpt;
use crate::shell::CompleteGuess;
use crate::trace;
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
    pub(crate) fn parse_impl<'a>(
        &mut self,
        set: &mut <Self as Policy>::Set,
        inv: &mut <Self as Policy>::Inv<'_>,
        ser: &mut <Self as Policy>::Ser,
        orig: &'a Args,
        ctx: &mut Ctx<'a>,
    ) -> Result<(), <Self as Policy>::Error> {
        let opt_styles = &self.style_manager;
        let args: Vec<_> = orig.iter().map(|v| v.as_os_str()).collect();
        let total = args.len();
        let mut left = vec![];
        let mut iter = args::iter2(&args).enumerate();

        trace!("parsing {ctx:?} using fwd policy");
        ctx.set_args(args.clone());
        while let Some((idx, (opt, next))) = iter.next() {
            let mut matched = false;
            let mut consume = false;

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
                    let mut guess = CompleteGuess {
                        idx,
                        arg,
                        set,
                        inv,
                        ser,
                        total,
                        ctx,
                        next,
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
                left.push(*opt);
            }
        }

        let args = left;
        let total = args.len();

        ctx.set_args(args.clone());
        // when style is pos, noa index is [1..=len]
        if total > 0 {
            let name = crate::str::osstr_to_str_i(&args, Self::noa_cmd());
            let mut guess = CompleteGuess {
                set,
                inv,
                ser,
                total,
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
                    total,
                    ctx,
                    name: None,
                    arg: None,
                    next: None,
                    idx: Self::noa_cmd(),
                };

                for idx in 1..total {
                    guess.idx = Self::noa_pos(idx);
                    guess.name = crate::str::osstr_to_str_i(&args, Self::noa_pos(idx));
                    guess.guess_complete(&UserStyle::Pos)?;
                }
            }
        }

        let name = crate::str::osstr_to_str_i(&args, Self::noa_main());
        let mut guess = CompleteGuess {
            set,
            inv,
            ser,
            total,
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
    type Ret = Return;

    type Set = Set;

    type Inv<'a> = Invoker<'a, Set, Ser>;

    type Ser = Ser;

    type Error = Error;

    fn parse(
        &mut self,
        set: &mut Self::Set,
        inv: &mut Self::Inv<'_>,
        ser: &mut Self::Ser,
        orig: Args,
    ) -> Result<Self::Ret, Self::Error> {
        let mut ctx = Ctx::default().with_orig(orig.clone());

        match self.parse_impl(set, inv, ser, &orig, &mut ctx) {
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
