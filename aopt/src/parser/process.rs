use crate::ctx::Ctx;
use crate::ctx::InnerCtx;
use crate::opt::Opt;
use crate::parser::CtxSaver;
use crate::prelude::HandlerCollection;
use crate::proc::Match;
use crate::proc::NOAProcess;
use crate::proc::OptProcess;
use crate::proc::Process;
use crate::set::SetOpt;
use crate::Error;
use crate::Uid;

use super::FailManager;

pub struct ProcessCtx<'a, Set, Ser, Inv> {
    pub idx: usize,

    pub tot: usize,

    pub ctx: &'a mut Ctx,

    pub set: &'a mut Set,

    pub inv: &'a mut Inv,

    pub ser: &'a mut Ser,
}

pub fn process_callback_ret(
    ret: Result<bool, Error>,
    mut when_ret: impl FnMut(bool) -> Result<(), Error>,
    mut when_fail: impl FnMut(&Error) -> Result<(), Error>,
) -> Result<bool, Error> {
    match ret {
        Ok(ret) => {
            (when_ret)(ret)?;
            Ok(ret)
        }
        Err(e) => {
            if e.is_failure() {
                (when_fail)(&e)?;
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

pub fn process_opt<'a, Set, Ser, Inv>(
    ProcessCtx {
        idx,
        tot,
        ctx,
        set,
        inv,
        ser,
    }: ProcessCtx<Set, Ser, Inv>,
    proc: &mut OptProcess<Set>,
    manager: &mut FailManager,
    invoke: bool,
) -> Result<Vec<CtxSaver>, Error>
where
    SetOpt<Set>: Opt,
    Set: crate::set::Set,
    Inv: HandlerCollection<'a, Set, Ser>,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys();
    let mut savers = vec![];

    crate::trace_log!("Opt process {:?}", proc);
    for uid in keys {
        match proc.process(uid, set) {
            Ok(index) => {
                if let Some(index) = index {
                    let mat = proc.get_match(index).unwrap(); // always true

                    // save the context
                    savers.push(CtxSaver {
                        uid,
                        idx: index,
                        ctx: InnerCtx::default()
                            .with_idx(idx)
                            .with_total(tot)
                            .with_uid(uid) // current uid == uid in matcher
                            .with_name(mat.name().cloned())
                            .with_style(mat.style())
                            .with_arg(mat.clone_arg()),
                    });
                }
            }
            Err(e) => {
                if !e.is_failure() {
                    return Err(e);
                } else {
                    manager.push(e);
                }
            }
        }
    }
    if proc.status() && invoke {
        for saver in savers {
            let uid = saver.uid;
            let fail = |e: &Error| {
                manager.push(e.clone());
                Ok(())
            };

            ctx.set_inner_ctx(Some(saver.ctx));
            // undo the process if option callback return None
            if !process_callback_ret(inv.invoke_fb(&uid, set, ser, ctx), |_| Ok(()), fail)? {
                proc.undo(set)?;
                break;
            }
        }
        Ok(vec![])
    } else {
        (!proc.status()).then(|| proc.undo(set));

        Ok(savers)
    }
}

pub fn process_non_opt<'a, Set, Ser, Inv>(
    ProcessCtx {
        idx,
        tot,
        ctx,
        set,
        inv,
        ser,
    }: ProcessCtx<Set, Ser, Inv>,
    proc: &mut NOAProcess<Set>,
    manager: &mut FailManager,
) -> Result<Vec<CtxSaver>, Error>
where
    SetOpt<Set>: Opt,
    Set: crate::set::Set,
    Inv: HandlerCollection<'a, Set, Ser>,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys();

    crate::trace_log!("NOA process {:?}", proc);
    for uid in keys {
        match proc.process(uid, set) {
            Ok(index) => {
                if let Some(index) = index {
                    let mat = proc.get_match(index).unwrap(); // always true
                    let fail = |e: &Error| {
                        manager.push(e.clone());
                        Ok(())
                    };

                    ctx.set_inner_ctx(Some(
                        InnerCtx::default()
                            .with_idx(idx)
                            .with_total(tot)
                            .with_style(mat.style())
                            .with_name(mat.name().cloned())
                            .with_arg(mat.clone_arg())
                            .with_uid(uid), // current uid == uid in matcher
                    ));

                    if !process_callback_ret(inv.invoke_fb(&uid, set, ser, ctx), |_| Ok(()), fail)?
                    {
                        proc.undo(set)?;
                    }
                    proc.reset();
                }
            }
            Err(e) => {
                if !e.is_failure() {
                    return Err(e);
                } else {
                    manager.push(e);
                }
            }
        }
    }
    Ok(vec![])
}
