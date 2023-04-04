use crate::ctx::Ctx;
use crate::ctx::InnerCtx;
use crate::ctx::Invoker;
use crate::opt::Opt;
use crate::parser::CtxSaver;
use crate::proc::Match;
use crate::proc::NOAProcess;
use crate::proc::OptProcess;
use crate::proc::Process;
use crate::set::SetOpt;
use crate::trace_log;
use crate::Error;
use crate::Uid;

pub struct ProcessCtx<'a, 'b, Set, Ser> {
    pub idx: usize,

    pub tot: usize,

    pub ctx: &'b mut Ctx,

    pub set: &'b mut Set,

    pub inv: &'b mut Invoker<'a, Set, Ser>,

    pub ser: &'b mut Ser,
}

/// Invoke the callback of option, map failure to false
pub fn invoke_callback_opt<Set, Ser>(
    uid: Uid,
    ctx: &mut Ctx,
    set: &mut Set,
    inv: &mut Invoker<Set, Ser>,
    ser: &mut Ser,
) -> Result<bool, Error>
where
    SetOpt<Set>: Opt,
    Ser: 'static,
    Set: crate::set::Set + 'static,
{
    match inv.has(uid) {
        true => {
            trace_log!("Invoke callback of {}", uid);
            inv.invoke(set, ser, ctx)
        }
        false => {
            trace_log!("Invoke default callback of {}", uid);
            inv.invoke_default(set, ser, ctx)
        }
    }
}

pub fn process_callback_ret(
    ret: Result<bool, Error>,
    mut func_ret: impl FnMut(bool) -> Result<(), Error>,
    mut func_fail: impl FnMut(&Error) -> Result<(), Error>,
) -> Result<bool, Error> {
    match ret {
        Ok(ret) => {
            (func_ret)(ret)?;
            Ok(ret)
        }
        Err(e) => {
            if e.is_failure() {
                (func_fail)(&e)?;
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

pub fn process_opt<Set, Ser>(
    ProcessCtx {
        idx,
        tot,
        ctx,
        set,
        inv,
        ser,
    }: ProcessCtx<Set, Ser>,
    proc: &mut OptProcess<Set>,
    invoke: bool,
) -> Result<Vec<CtxSaver>, Error>
where
    SetOpt<Set>: Opt,
    Ser: 'static,
    Set: crate::set::Set + 'static,
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
                    proc.set_failed_info(e.display());
                }
            }
        }
    }
    if proc.status() && invoke {
        for saver in savers {
            let uid = saver.uid;

            ctx.set_inner_ctx(Some(saver.ctx));
            // undo the process if option callback return None
            if !process_callback_ret(
                invoke_callback_opt(uid, ctx, set, inv, ser),
                |_| Ok(()),
                |e: &Error| {
                    proc.set_failed_info(e.display());
                    Ok(())
                },
            )? {
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

pub fn process_non_opt<Set, Ser>(
    ProcessCtx {
        idx,
        tot,
        ctx,
        set,
        inv,
        ser,
    }: ProcessCtx<Set, Ser>,
    proc: &mut NOAProcess<Set>,
) -> Result<Vec<CtxSaver>, Error>
where
    SetOpt<Set>: Opt,
    Ser: 'static,
    Set: crate::set::Set + 'static,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys().to_vec();

    crate::trace_log!("NOA process {:?}", proc);
    for uid in keys {
        match proc.process(uid, set) {
            Ok(index) => {
                if let Some(index) = index {
                    let mat = proc.get_match(index).unwrap(); // always true

                    ctx.set_inner_ctx(Some(
                        InnerCtx::default()
                            .with_idx(idx)
                            .with_total(tot)
                            .with_style(mat.style())
                            .with_name(mat.name().cloned())
                            .with_arg(mat.clone_arg())
                            .with_uid(uid), // current uid == uid in matcher
                    ));

                    if !process_callback_ret(
                        invoke_callback_opt(uid, ctx, set, inv, ser),
                        |_| Ok(()),
                        |e: &Error| {
                            proc.set_failed_info(e.display());
                            Ok(())
                        },
                    )? {
                        proc.undo(set)?;
                    }
                    proc.reset();
                }
            }
            Err(e) => {
                if !e.is_failure() {
                    return Err(e);
                } else {
                    proc.set_failed_info(e.to_string());
                }
            }
        }
    }
    Ok(vec![])
}
