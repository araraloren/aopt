use tracing::trace;

use crate::ctx::Ctx;
use crate::opt::Creator;
use crate::opt::Opt;
use crate::parser::CtxSaver;
use crate::proc::Match;
use crate::proc::NOAProcess;
use crate::proc::OptProcess;
use crate::proc::Process;
use crate::ser::InvokeService;
use crate::ser::Services;
use crate::Error;
use crate::Uid;

pub fn invoke_callback_opt<Set>(
    saver: CtxSaver,
    set: &mut Set,
    ser: &mut Services,
) -> Result<Option<()>, Error>
where
    <Set::Ctor as Creator>::Opt: Opt,
    Set: crate::set::Set + 'static,
{
    let uid = saver.uid;
    // Take the service, invoke the handler of option.
    // Catch the result of handler, so we can register it back to Services.
    let mut inv_ser = ser.take::<InvokeService<Set>>()?;
    let ret = match inv_ser.has(uid) {
        true => {
            trace!("Invoke callback of Opt{{{uid}}} with {:?}", saver.ctx);
            inv_ser.invoke(set, ser, &saver.ctx)
        }
        false => {
            trace!("Invoke default of Opt{{{uid}}} with {:?}", saver.ctx);
            inv_ser.invoke_default(set, ser, &saver.ctx)
        }
    };

    ser.register(inv_ser);
    ret
}

pub fn process_opt<Set>(
    ctx: &Ctx,
    set: &mut Set,
    ser: &mut Services,
    proc: &mut OptProcess<Set>,
    invoke: bool,
) -> Result<Vec<CtxSaver>, Error>
where
    <Set::Ctor as Creator>::Opt: Opt,
    Set: crate::set::Set + 'static,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys().to_vec();
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
                        ctx: ctx
                            .clone()
                            .with_uid(uid) // current uid == uid in matcher
                            .with_name(mat.name().cloned())
                            .with_prefix(mat.prefix().cloned())
                            .with_style(mat.style())
                            .with_arg(mat.clone_arg())
                            .with_disable(mat.disable()),
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
            // undo the process if option callback return None
            if invoke_callback_opt(saver, set, ser)?.is_none() {
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

pub fn process_non_opt<Set>(
    ctx: &Ctx,
    set: &mut Set,
    ser: &mut Services,
    proc: &mut NOAProcess<Set>,
) -> Result<Vec<CtxSaver>, Error>
where
    <Set::Ctor as Creator>::Opt: Opt,
    Set: crate::set::Set + 'static,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys().to_vec();

    for uid in keys {
        match proc.process(uid, set) {
            Ok(index) => {
                if let Some(index) = index {
                    let mat = proc.mat(index).unwrap(); // always true

                    // save the context
                    let ctx = ctx
                        .clone()
                        .with_style(mat.style())
                        .with_name(mat.name().cloned())
                        .with_arg(mat.clone_arg())
                        .with_uid(uid); // current uid == uid in matcher
                    let mut inv_ser = ser.take::<InvokeService<Set>>()?;
                    let ret = match inv_ser.has(uid) {
                        true => {
                            // callback in InvokeService
                            trace!("Invoke callback of NOA{{{uid}}} with {:?}", &ctx);
                            inv_ser.invoke(set, ser, &ctx)
                        }
                        false => {
                            // call `invoke_default` if callback not exist
                            trace!("Invoke default of NOA{{{uid}}} with {:?}", &ctx);
                            inv_ser.invoke_default(set, ser, &ctx)
                        }
                    };

                    ser.register(inv_ser);
                    let ret = ret?;

                    // rteurn None means NOA not match
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
