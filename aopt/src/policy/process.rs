use crate::ctx::Ctx;
use crate::opt::Opt;
use crate::policy::CtxSaver;
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
    inv_ser: &mut InvokeService<Set>,
) -> Result<Option<()>, Error>
where
    Set::Opt: Opt,
    Set: crate::set::Set + 'static,
{
    let uid = saver.uid;
    Ok(match inv_ser.has(uid) {
        true => {
            // callback in InvokeService
            inv_ser.invoke(uid, set, ser, &saver.ctx)?
        }
        false => {
            // call `invoke_default` if callback not exist
            inv_ser.invoke_default(uid, set, ser, &saver.ctx)?
        }
    })
}

pub fn process_opt<Set>(
    ctx: &Ctx,
    set: &mut Set,
    ser: &mut Services,
    proc: &mut OptProcess<Set>,
    inv_ser: &mut InvokeService<Set>,
    invoke: bool,
) -> Result<Vec<CtxSaver>, Error>
where
    Set::Opt: Opt,
    Set: crate::set::Set + 'static,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys().to_vec();
    let mut savers = vec![];

    for uid in keys {
        if let Ok(Some(index)) = proc.process(uid, set) {
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
    if proc.is_mat() && invoke {
        for saver in savers {
            // undo the process if option callback return None
            if invoke_callback_opt(saver, set, ser, inv_ser)?.is_none() {
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
    inv_ser: &mut InvokeService<Set>,
) -> Result<Vec<CtxSaver>, Error>
where
    Set::Opt: Opt,
    Set: crate::set::Set + 'static,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys().to_vec();

    for uid in keys {
        if let Ok(Some(index)) = proc.process(uid, set) {
            let mat = proc.mat(index).unwrap(); // always true

            // save the context
            let ctx = ctx
                .clone()
                .with_style(mat.style())
                .with_name(mat.name().cloned())
                .with_uid(uid); // current uid == uid in matcher
            let ret = match inv_ser.has(uid) {
                true => {
                    // callback in InvokeService
                    inv_ser.invoke(uid, set, ser, &ctx)?
                }
                false => {
                    // call `invoke_default` if callback not exist
                    inv_ser.invoke_default(uid, set, ser, &ctx)?
                }
            };

            // rteurn None means NOA not match
            if ret.is_none() {
                proc.undo(set)?;
            }
            proc.reset();
        }
    }
    Ok(vec![])
}
