use std::ffi::OsStr;
use std::ffi::OsString;

use super::CtxSaver;
use crate::ctx::Ctx;
use crate::opt::Opt;
use crate::proc::Match;
use crate::proc::NOAProcess;
use crate::proc::OptProcess;
use crate::proc::Process;
use crate::ser::InvokeService;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::ser::ServicesExt;
use crate::Arc;
use crate::Error;
use crate::Str;
use crate::Uid;

pub fn invoke_callback_opt<Set>(
    saver: CtxSaver,
    set: &mut Set,
    ser: &mut Services,
    inv_ser: &mut InvokeService<Set>,
) -> Result<(), Error>
where
    Set::Opt: Opt,
    Set: crate::set::Set + 'static,
{
    let uid = saver.uid;
    let has_callback = inv_ser.has(uid);

    if has_callback {
        // callback in InvokeService
        inv_ser.invoke(uid, set, ser, &saver.ctx)?;
    } else {
        inv_ser.invoke_default(uid, set, ser, &saver.ctx)?;
    }

    Ok(())
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
    let mut context_savers = vec![];

    for uid in keys {
        if let Ok(Some(index)) = proc.process(uid, set) {
            let mat = proc.mat(index).unwrap(); // always true

            // save the context
            context_savers.push(CtxSaver {
                uid,
                ctx: {
                    let mut ctx = ctx.clone();

                    // .set_idx(idx) set when process option
                    // .set_len(len) set before process options
                    // .set_args(args) set before process options
                    ctx.opt_mut()?
                        .set_uid(uid) // current uid == uid in matcher
                        .set_name(mat.name().clone())
                        .set_pre(mat.pre().cloned())
                        .set_sty(mat.sty())
                        .set_arg(mat.arg().cloned())
                        .set_dsb(mat.dsb());
                    ctx
                },
            });
        }
    }
    if proc.is_mat() && invoke {
        for saver in context_savers {
            invoke_callback_opt(saver, set, ser, inv_ser)?;
        }
        Ok(vec![])
    } else {
        (!proc.is_mat()).then(|| proc.undo(set));

        Ok(context_savers)
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
            let mut matched = true;
            let mat = proc.mat(index).unwrap(); // always true

            // save the context
            let ctx = {
                let mut ctx = ctx.clone();
                // .set_idx(idx) set when process option
                // .set_len(len) set before process options
                // .set_args(args) set before process options
                ctx.noa_mut()?.set_sty(mat.sty()).set_uid(uid); // current uid == uid in matcher
                ctx
            };

            let ret;
            let has_callback = inv_ser.has(uid);

            if has_callback {
                // callback in InvokeService
                ret = inv_ser.invoke(uid, set, ser, &ctx)?;
                matched = ret.is_some();
            } else {
                ret = inv_ser.invoke_default(uid, set, ser, &ctx)?;
            }

            // rteurn None means NOA not match
            if !matched {
                proc.undo(set)?;
            } else {
                //set.get_mut(uid).unwrap().val_act(ret, ser, &ctx)?;
            }
            proc.reset();
        }
    }
    Ok(vec![])
}
