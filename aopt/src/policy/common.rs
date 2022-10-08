use super::CtxSaver;
use crate::ctx::Ctx;
use crate::opt::Opt;
use crate::proc::NOAProcess;
use crate::proc::OptProcess;
use crate::proc::Process;
use crate::ser::InvokeService;
use crate::ser::Services;
use crate::ser::ServicesExt;
use crate::ser::ValueService;
use crate::Error;
use crate::Str;
use crate::Uid;

pub fn invoke_callback_opt<Set, Value>(
    saver: CtxSaver,
    set: &mut Set,
    ser: &mut Services,
    inv_ser: &mut InvokeService<Set, Value>,
) -> Result<(), Error>
where
    Set::Opt: Opt,
    Value: From<Str> + 'static,
    Set: crate::set::Set + 'static,
{
    let uid = saver.uid;
    let ret;
    let has_callback_is = inv_ser.has(uid);
    let has_callback_opt = set.get(uid).unwrap().has_callback();

    if has_callback_is {
        // callback in InvokeService
        ret = inv_ser.invoke(uid, set, ser, saver.ctx.clone())?;
    } else if has_callback_opt {
        // callback in OptCallback
        ret = set
            .get_mut(uid)
            .unwrap()
            .invoke(ser, &saver.ctx)?
            .map(|v| Value::from(v));
    } else {
        // default value
        ret = saver.ctx.arg().map(|v| Value::from(v));
    }
    // save the value to ValueService
    if let Some(ret) = ret {
        ser.ser_mut::<ValueService<Value>>()?.ins(uid, ret);
    }
    Ok(())
}

pub fn process_opt<Set, Value>(
    ctx: &Ctx,
    set: &mut Set,
    ser: &mut Services,
    proc: &mut OptProcess<Set>,
    inv_ser: &mut InvokeService<Set, Value>,
    invoke: bool,
) -> Result<Vec<CtxSaver>, Error>
where
    Set::Opt: Opt,
    Value: From<Str> + 'static,
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
                ctx: ctx
                    .clone()
                    // .with_idx(idx) set when process option
                    // .with_len(len) set before process options
                    // .with_args(args) set before process options
                    .with_uid(uid) // current uid == uid in matcher
                    .with_name(mat.name())
                    .with_pre(mat.pre())
                    .with_sty(mat.sty())
                    .with_arg(mat.arg())
                    .with_dsb(mat.dsb()),
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

pub fn process_non_opt<Set, Value>(
    ctx: &Ctx,
    set: &mut Set,
    ser: &mut Services,
    proc: &mut NOAProcess<Set>,
    inv_ser: &mut InvokeService<Set, Value>,
) -> Result<Vec<CtxSaver>, Error>
where
    Set::Opt: Opt,
    Value: From<Str> + 'static,
    Set: crate::set::Set + 'static,
{
    // copy the uid of option, avoid borrow the set
    let keys: Vec<Uid> = set.keys().to_vec();

    for uid in keys {
        if let Ok(Some(index)) = proc.process(uid, set) {
            let mut matched = true;
            let mat = proc.mat(index).unwrap(); // always true

            // save the context
            let ctx = ctx
                .clone()
                // .with_idx(idx) set when process option
                // .with_len(len) set before process options
                // .with_args(args) set before process options
                .with_name(mat.get_name())
                .with_pre(mat.pre())
                .with_sty(mat.sty())
                .with_uid(uid) // current uid == uid in matcher
                .with_arg(mat.arg())
                .with_dsb(mat.dsb());

            let ret;
            let has_callback_is = inv_ser.has(uid);
            let has_callback_opt = set.get(uid).unwrap().has_callback();

            if has_callback_is {
                // callback in InvokeService
                ret = inv_ser.invoke(uid, set, ser, ctx.clone())?;
                matched = ret.is_some();
            } else if has_callback_opt {
                // callback in OptCallback
                ret = set
                    .get_mut(uid)
                    .unwrap()
                    .invoke(ser, &ctx)?
                    .map(|v| Value::from(v));
                matched = ret.is_some();
            } else {
                // default value
                ret = ctx.arg_of().map(|v| Value::from(v.clone()));
            }
            // save the value to ValueService
            if let Some(ret) = ret {
                ser.ser_mut::<ValueService<Value>>()?.ins(uid, ret);
            }
            // reset the process if any callback return None
            if !matched {
                proc.undo(set)?;
            }
            proc.reset();
        }
    }
    Ok(vec![])
}
