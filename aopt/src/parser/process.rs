use tracing::trace;

use crate::ctx::Ctx;
use crate::ctx::InnerCtx;
use crate::ctx::Invoker;
use crate::opt::Opt;
use crate::parser::CtxSaver;
use crate::prelude::ServicesExt;
use crate::proc::Match;
use crate::proc::NOAProcess;
use crate::proc::OptProcess;
use crate::proc::Process;
use crate::set::SetOpt;
use crate::Error;
use crate::Uid;

pub struct ProcessCtx<'a, Set, Ser> {
    pub idx: usize,

    pub tot: usize,

    pub ctx: &'a mut Ctx,

    pub set: &'a mut Set,

    pub inv: &'a mut Invoker<Set, Ser>,

    pub ser: &'a mut Ser,
}

pub fn invoke_callback_opt<Set, Ser>(
    uid: Uid,
    ctx: &Ctx,
    set: &mut Set,
    inv: &mut Invoker<Set, Ser>,
    ser: &mut Ser,
) -> Result<Option<()>, Error>
where
    SetOpt<Set>: Opt,
    Ser: ServicesExt + 'static,
    Set: crate::set::Set + 'static,
{
    // Take the service, invoke the handler of option.
    // Catch the result of handler, so we can register it back to Services.
    match inv.has(uid) {
        true => {
            trace!("Invoke callback of Opt{{{uid}}} with {:?}", ctx);
            inv.invoke(set, ser, &ctx)
        }
        false => {
            trace!("Invoke default of Opt{{{uid}}} with {:?}", ctx);
            inv.invoke_default(set, ser, &ctx)
        }
    }
}

pub fn process_opt<'a, Set, Ser>(
    ProcessCtx {
        idx,
        tot,
        ctx,
        set,
        inv,
        ser,
    }: ProcessCtx<'a, Set, Ser>,
    proc: &mut OptProcess<Set>,
    invoke: bool,
) -> Result<Vec<CtxSaver>, Error>
where
    SetOpt<Set>: Opt,
    Ser: ServicesExt + 'static,
    Set: crate::set::Set + 'static,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys();
    let mut savers = vec![];

    for uid in keys {
        match proc.process(uid, set) {
            Ok(index) => {
                if let Some(index) = index {
                    let mat = proc.mat(index).unwrap(); // always true

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
                }
            }
        }
    }
    if proc.is_mat() && invoke {
        for saver in savers {
            let uid = saver.uid;

            ctx.set_inner_ctx(Some(saver.ctx));
            // undo the process if option callback return None
            if invoke_callback_opt(uid, ctx, set, inv, ser)?.is_none() {
                proc.undo(set)?;
                break;
            }
        }
        Ok(vec![])
    } else {
        (!proc.is_mat()).then(|| proc.undo(set));

        Ok(savers)
    }
}

pub fn process_non_opt<'a, Set, Ser>(
    ProcessCtx {
        idx,
        tot,
        ctx,
        set,
        inv,
        ser,
    }: ProcessCtx<'a, Set, Ser>,
    proc: &mut NOAProcess<Set>,
) -> Result<Vec<CtxSaver>, Error>
where
    SetOpt<Set>: Opt,
    Ser: ServicesExt + 'static,
    Set: crate::set::Set + 'static,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys().to_vec();

    for uid in keys {
        match proc.process(uid, set) {
            Ok(index) => {
                if let Some(index) = index {
                    let mat = proc.mat(index).unwrap(); // always true

                    ctx.set_inner_ctx(Some(
                        InnerCtx::default()
                            .with_idx(idx)
                            .with_total(tot)
                            .with_style(mat.style())
                            .with_name(mat.name().cloned())
                            .with_arg(mat.clone_arg())
                            .with_uid(uid), // current uid == uid in matcher
                    ));

                    let ret = match inv.has(uid) {
                        true => {
                            // callback in Invoker
                            trace!("Invoke callback of NOA{{{uid}}} with {:?}", &ctx);
                            inv.invoke(set, ser, &ctx)
                        }
                        false => {
                            // call `invoke_default` if callback not exist
                            trace!("Invoke default of NOA{{{uid}}} with {:?}", &ctx);
                            inv.invoke_default(set, ser, &ctx)
                        }
                    };
                    let ret = ret?;

                    // return None means NOA not match
                    if ret.is_none() {
                        proc.undo(set)?;
                    }
                    proc.reset();
                }
            }
            Err(e) => {
                if !e.is_failure() {
                    return Err(e);
                }
            }
        }
    }
    Ok(vec![])
}
