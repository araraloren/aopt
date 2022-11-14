use std::fmt::Debug;
use std::marker::PhantomData;

use super::process_non_opt;
use super::process_opt;
use super::Guess;
use super::GuessNOACfg;
use super::GuessOptCfg;
use super::NOAGuess;
use super::OptGuess;
use super::Policy;
use super::UserStyle;
use crate::args::ArgParser;
use crate::args::Args;
use crate::astr;
use crate::ctx::Ctx;
use crate::ext::ServicesExt;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::prelude::SetExt;
use crate::proc::Process;
use crate::ser::InvokeService;
use crate::ser::Services;
use crate::set::Pre;
use crate::set::Set;
use crate::Arc;
use crate::Error;

/// Forward process the option before any
/// NOA([`Cmd`](crate::opt::Style::Cmd), [`Pos`](crate::opt::Style::Pos) and [`Main`](crate::opt::Style::Main)).
///
/// You can get the value of any option in the handler of NOA.
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Arc;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut policy = AForward::default();
/// let mut set = policy.default_set();
/// let mut ser = policy.default_ser();
///
/// ser.ser_usrval_mut()?
///     .insert(ser::Value::new(vec!["foo", "bar"]));
///
/// let filter_id = set.add_opt("--filter=b/")?.run()?;
/// let pos_id = set
///     .add_opt("pos=p@*")?
///     .set_initiator(ValInitiator::empty::<String>())
///     .run()?;
/// ser.ser_invoke_mut()?
///     .register_ser(
///         pos_id,
///         move |_: Uid,
///                 _: &mut ASet,
///                 ser: &mut Services,
///                 filter: ser::Value<Vec<&str>>,
///                 mut value: ctx::Value<String>| {
///             let do_filter = bool::val(filter_id, ser)?;
///             let valid = if *do_filter {
///                 !filter.iter().any(|&v| v == value.as_str())
///             } else {
///                 true
///             };
///
///             Ok(valid.then(|| value.take()))
///         },
///     )
///     .with_default();
///
/// let args = Args::new(["set", "42", "foo", "bar"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// let values = String::vals(pos_id, &ser)?;
///
/// assert_eq!(values[0], "set");
/// assert_eq!(values[1], "42");
///
/// let args = Args::new(["--/filter", "set", "42", "foo", "bar"].into_iter());
///
/// policy.parse(Arc::new(args), &mut ser, &mut set)?;
///
/// let values = String::vals(pos_id, &ser)?;
///
/// assert_eq!(values[0], "set");
/// assert_eq!(values[1], "42");
/// assert_eq!(values[2], "foo");
/// assert_eq!(values[3], "bar");
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Forward<S> {
    strict: bool,

    marker_s: PhantomData<S>,
}

impl<S> Default for Forward<S> {
    fn default() -> Self {
        Self {
            strict: true,
            marker_s: PhantomData::default(),
        }
    }
}

impl<S> Forward<S>
where
    S::Opt: Opt,
    S: Set + OptParser + Debug + 'static,
{
    pub fn new(strict: bool) -> Self {
        Self {
            strict,
            ..Default::default()
        }
    }

    /// In strict mode, if an argument looks like an option (it matched any option prefix),
    /// then it must matched.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    pub fn set_strict(&mut self, strict: bool) -> &mut Self {
        self.strict = strict;
        self
    }

    pub fn get_strict(&self) -> bool {
        self.strict
    }
}

impl<S> Policy for Forward<S>
where
    S::Opt: Opt,
    S: Set + OptParser + Pre + Debug + 'static,
{
    type Ret = bool;

    type Set = S;

    type Error = Error;

    fn parse(
        &mut self,
        args: Arc<Args>,
        ser: &mut Services,
        set: &mut Self::Set,
    ) -> Result<Option<Self::Ret>, Self::Error> {
        let keys = set.keys().to_vec();

        for id in keys {
            set.opt_mut(id)?.init(ser)?;
        }
        ser.ser_check()?.pre_check(set)?;

        // take the invoke service, avoid borrow the ser
        let mut is = ser.take::<InvokeService<S>>()?;
        let stys = [
            UserStyle::EqualWithValue,
            UserStyle::Argument,
            UserStyle::Boolean,
            UserStyle::CombinedOption,
            UserStyle::EmbeddedValue,
        ];
        let args_len = args.len();
        let mut noa_args = Args::default();
        let mut iter = args.iter().enumerate();
        let mut opt_ctx = Ctx::default();

        opt_ctx.set_args(args.clone()).set_total(args_len);

        while let Some((idx, (opt, arg))) = iter.next() {
            let mut matched = false;
            let mut consume = false;
            let arg = arg.map(|v| Arc::new(v.clone()));

            if let Ok(clopt) = opt.parse(set.prefix()) {
                for style in stys.iter() {
                    if let Some(mut proc) = OptGuess::new()
                        .guess(style, GuessOptCfg::new(idx, args_len, arg.clone(), &clopt))?
                    {
                        opt_ctx.set_idx(idx);
                        process_opt::<S>(&opt_ctx, set, ser, &mut proc, &mut is, true)?;
                        if proc.is_mat() {
                            matched = true;
                        }
                        if proc.consume() {
                            consume = true;
                        }
                        if matched {
                            break;
                        }
                    }
                }
                if !matched && self.get_strict() {
                    let default_str = astr("");

                    return Err(Error::sp_invalid_option_name(format!(
                        "{}{}",
                        clopt.prefix().unwrap_or(&default_str),
                        clopt.name().unwrap_or(&default_str)
                    )));
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

        ser.ser_check()?.opt_check(set)?;

        let noa_args = Arc::new(noa_args);
        let noa_len = noa_args.len();
        let mut noa_ctx = Ctx::default();

        noa_ctx.set_args(noa_args.clone()).set_total(noa_args.len());

        // when style is pos, noa index is [1..=len]
        if noa_args.len() > 0 {
            if let Some(mut proc) = NOAGuess::new().guess(
                &UserStyle::Cmd,
                GuessNOACfg::new(noa_args.clone(), 1, noa_len),
            )? {
                noa_ctx.set_idx(1);
                process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is)?;
            }

            ser.ser_check()?.cmd_check(set)?;

            for idx in 0..noa_len {
                if let Some(mut proc) = NOAGuess::new().guess(
                    &UserStyle::Pos,
                    GuessNOACfg::new(noa_args.clone(), idx + 1, noa_len),
                )? {
                    noa_ctx.set_idx(idx + 1);
                    process_non_opt::<S>(&noa_ctx, set, ser, &mut proc, &mut is)?;
                }
            }
        } else {
            ser.ser_check()?.cmd_check(set)?;
        }
        ser.ser_check()?.pos_check(set)?;

        let main_args = noa_args;
        let mut main_ctx = noa_ctx;

        main_ctx.set_idx(0);
        if let Some(mut proc) =
            NOAGuess::new().guess(&UserStyle::Main, GuessNOACfg::new(main_args, 0, noa_len))?
        {
            process_non_opt::<S>(&main_ctx, set, ser, &mut proc, &mut is)?;
        }

        ser.ser_check()?.post_check(set)?;
        ser.register(is);

        Ok(Some(true))
    }
}

#[cfg(test)]
mod test {

    use std::ops::Deref;

    use crate::prelude::*;
    use crate::Arc;
    use crate::Error;
    use crate::RawVal;

    #[test]
    fn testing_1() {
        assert!(testing_1_main().is_ok());
    }

    fn testing_1_main() -> Result<(), Error> {
        fn check_opt_val<T: std::fmt::Debug + PartialEq + 'static>(
            ser: &mut ASer,
            opt: &AOpt,
            uid: Uid,
            name: &str,
            prefix: Option<&str>,
            vals: Option<Vec<T>>,
            optional: bool,
            action: &ValAction,
            assoc: &ValAssoc,
            index: Option<&Index>,
            alias: Option<Vec<(&str, &str)>>,
            deactivate: bool,
        ) -> Result<(), Error> {
            let opt_uid = opt.uid();

            assert_eq!(opt_uid, uid);
            assert_eq!(opt.name(), name);
            assert_eq!(opt.prefix().map(|v| v.as_str()), prefix);
            assert_eq!(opt.optional(), optional, "optional not equal: {}", optional);
            assert_eq!(opt.action(), action);
            assert_eq!(opt.assoc(), assoc);
            assert_eq!(opt.idx(), index, "option index not equal: {:?}", index);
            assert_eq!(
                opt.is_deactivate(),
                deactivate,
                "deactivate style not matched!"
            );
            if let Some(opt_vals) = T::vals(opt_uid, ser).ok() {
                if let Some(vals) = vals {
                    assert_eq!(opt_vals.len(), vals.len());
                    for (l, r) in opt_vals.iter().zip(vals.iter()) {
                        assert_eq!(l, r, "option value not equal: {:?} != {:?}", l, r);
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
                    for (prefix, name) in alias {
                        assert!(
                            opt_alias.iter().any(|(p, n)| p == prefix && n == name),
                            "alias => {:?} <--> {}, {}",
                            &opt_alias,
                            prefix,
                            name,
                        );
                    }
                }
            } else {
                assert!(alias.is_none());
            }
            Ok(())
        }

        fn string_collection_validator(vals: Vec<&'static str>) -> ValValidator {
            ValValidator::new(
                move |_: &str,
                      val: Option<&RawVal>,
                      _: bool,
                      _: (usize, usize)|
                      -> Result<bool, Error> {
                    Ok(val
                        .map(|v| v.to_str())
                        .flatten()
                        .map(|v| vals.contains(&v))
                        .unwrap_or_default())
                },
            )
        }

        let mut policy = AForward::default();
        let mut set = policy.default_set();
        let mut ser = policy.default_ser();
        let args = Args::new(
            [
                "--copt",
                "--iopt=63",
                "--/dopt",
                "set", // 1
                "--iopt",
                "-42",
                "+eopt",
                "-/fopt",
                "8",       // 2
                "16",      // 3
                "average", // 4
                "--jopt",
                "2",
                "--iopt-alias1",
                "0",
                "--nopt=8.99",
                "--hopt",
                "48",
                "--alias-k=4",
                "-l2.79",
                "--nopt",
                "3.12",
            ]
            .into_iter(),
        );

        set.add_prefix("+");

        // 5
        set.add_opt("--aopt=b")?.run()?;
        set.add_opt("--bopt=b/")?.run()?;
        set.add_opt("--copt=b!")?.run()?;
        set.add_opt("--dopt=b!/")?.run()?;
        set.add_opt("--eopt=b")?.add_alias("+eopt").run()?;
        set.add_opt("--fopt=b/")?.add_alias("-fopt").run()?;

        // 8
        set.add_opt("--gopt=i")?.run()?;
        set.add_opt("--hopt=i!")?.run()?;
        ser.ser_invoke_mut()?
            .register_ser(
                set.add_opt("--iopt=i")?.add_alias("--iopt-alias1").run()?,
                |set: &mut ASet, ser: &mut ASer, val: ctx::Value<i64>| {
                    assert_eq!(
                        i64::val(set["--hopt"].uid(), ser).ok(),
                        None,
                        "Option can set in any order, not access it in option"
                    );
                    Ok(Some(val.deref() + 21))
                },
            )
            .with_default();

        // 10
        set.add_opt("--jopt=u")?.set_optional(false).run()?;
        set.add_opt("--kopt=u")?
            .set_action(ValAction::Set)
            .add_alias("--alias-k")
            .run()?;

        // 13
        set.add_opt("--lopt=f!")?.add_alias("-l").run()?;
        set.add_opt("--mopt=f")?.set_value(1.02f64).run()?;
        set.add_opt("--nopt=f")?.set_action(ValAction::Set).run()?;

        let apos_uid = set
            .add_opt("apos=p@1")?
            .set_initiator(ValInitiator::empty::<String>())
            .run()?;
        let bpos_uid = set
            .add_opt("bpos=p@[2,3]")?
            .set_assoc(ValAssoc::Uint)
            .run()?;
        let cpos_uid = set
            .add_opt("cpos=p@4..5")?
            .set_validator(string_collection_validator(vec!["average", "plus"]))
            .run()?;

        ser.ser_invoke_mut::<ASet>()?
            .register_ser(
                set.add_opt("main=m")?.run()?,
                move |set: &mut ASet, ser: &mut Services, idx: ctx::Index| {
                    let copt = &set["--copt"];
                    let dopt = &set["dopt"];
                    let bpos = &set["bpos"];
                    let cpos = &set[cpos_uid];

                    assert_eq!(idx.deref(), &0);
                    check_opt_val(
                        ser,
                        cpos,
                        cpos_uid,
                        "cpos",
                        None,
                        Some(vec![2.31]),
                        true,
                        &ValAction::App,
                        &ValAssoc::Noa,
                        Some(&Index::Range(4, 5)),
                        None,
                        false,
                    )?;
                    check_opt_val::<u64>(
                        ser,
                        bpos,
                        bpos_uid,
                        "bpos",
                        None,
                        Some(vec![32, 64]),
                        true,
                        &ValAction::App,
                        &ValAssoc::Uint,
                        Some(&Index::list(vec![2, 3])),
                        None,
                        false,
                    )?;
                    check_opt_val(
                        ser,
                        copt,
                        2,
                        "copt",
                        Some("--"),
                        Some(vec![true]),
                        false,
                        &ValAction::Set,
                        &ValAssoc::Bool,
                        None,
                        None,
                        false,
                    )?;
                    check_opt_val(
                        ser,
                        dopt,
                        3,
                        "dopt",
                        Some("--"),
                        Some(vec![false]),
                        false,
                        &ValAction::Set,
                        &ValAssoc::Bool,
                        None,
                        None,
                        true,
                    )?;
                    Ok(Some(true))
                },
            )
            .with_default();
        ser.ser_invoke_mut()?
            .register_ser(
                cpos_uid,
                |set: &mut ASet, ser: &mut ASer, val: ctx::Value<String>, idx: ctx::Index| {
                    let lopt = &set["--lopt"];
                    let mopt = &set["--mopt"];
                    let nopt = &set["--nopt"];

                    check_opt_val(
                        ser,
                        nopt,
                        13,
                        "nopt",
                        Some("--"),
                        Some(vec![3.12]),
                        true,
                        &ValAction::Set,
                        &ValAssoc::Flt,
                        None,
                        None,
                        false,
                    )?;
                    check_opt_val::<f64>(
                        ser,
                        mopt,
                        12,
                        "mopt",
                        Some("--"),
                        Some(vec![1.02]),
                        true,
                        &ValAction::App,
                        &ValAssoc::Flt,
                        None,
                        None,
                        false,
                    )?;
                    check_opt_val::<f64>(
                        ser,
                        lopt,
                        11,
                        "lopt",
                        Some("--"),
                        Some(vec![2.79]),
                        false,
                        &ValAction::App,
                        &ValAssoc::Flt,
                        None,
                        Some(vec![("-", "l")]),
                        false,
                    )?;
                    assert!(idx.deref() == &4);

                    let mut sum = 0.0;

                    for uid in [lopt, mopt, nopt].iter().map(|v| v.uid()) {
                        sum += f64::val(uid, ser)?;
                    }

                    match val.deref().as_str() {
                        "average" => Ok(Some(sum / 3.0)),
                        "plus" => Ok(Some(sum)),
                        _ => Ok(None),
                    }
                },
            )
            .with_default();
        ser.ser_invoke_mut()?
            .register_ser(
                bpos_uid,
                |set: &mut ASet, ser: &mut ASer, val: ctx::Value<u64>, idx: ctx::Index| {
                    let jopt = &set["--jopt"];
                    let kopt = &set["--kopt"];

                    check_opt_val::<u64>(
                        ser,
                        jopt,
                        9,
                        "jopt",
                        Some("--"),
                        Some(vec![2]),
                        false,
                        &ValAction::App,
                        &ValAssoc::Uint,
                        None,
                        None,
                        false,
                    )?;
                    check_opt_val::<u64>(
                        ser,
                        kopt,
                        10,
                        "kopt",
                        Some("--"),
                        Some(vec![4]),
                        true,
                        &ValAction::Set,
                        &ValAssoc::Uint,
                        None,
                        None,
                        false,
                    )?;
                    assert!(idx.deref() == &2 || idx.deref() == &3);
                    Ok(Some(val.deref() * u64::val(set["--alias-k"].uid(), ser)?))
                },
            )
            .with_default();
        ser.ser_invoke_mut()?
            .register_ser(
                apos_uid,
                move |set: &mut ASet,
                      ser: &mut ASer,
                      uid: ctx::Uid,
                      name: ctx::Name,
                      mut value: ctx::Value<String>| {
                    let aopt = &set[0];
                    let bopt = &set["--bopt"];
                    let apos = &set[*uid.deref()];
                    let eopt = &set["+eopt"];
                    let fopt = &set["--fopt=b"];
                    let gopt = &set["--gopt"];
                    let hopt = &set["--hopt"];
                    let iopt = &set["--iopt"];

                    assert_eq!(name.deref(), "set");
                    check_opt_val::<i64>(
                        ser,
                        iopt,
                        8,
                        "iopt",
                        Some("--"),
                        Some(vec![84, -21, 21]),
                        true,
                        &ValAction::App,
                        &ValAssoc::Int,
                        None,
                        Some(vec![("--", "iopt-alias1")]),
                        false,
                    )?;
                    check_opt_val::<i64>(
                        ser,
                        hopt,
                        7,
                        "hopt",
                        Some("--"),
                        Some(vec![48]),
                        false,
                        &ValAction::App,
                        &ValAssoc::Int,
                        None,
                        None,
                        false,
                    )?;
                    check_opt_val::<i64>(
                        ser,
                        gopt,
                        6,
                        "gopt",
                        Some("--"),
                        None,
                        true,
                        &ValAction::App,
                        &ValAssoc::Int,
                        None,
                        None,
                        false,
                    )?;

                    check_opt_val(
                        ser,
                        fopt,
                        5,
                        "fopt",
                        Some("--"),
                        Some(vec![false]),
                        true,
                        &ValAction::Set,
                        &ValAssoc::Bool,
                        None,
                        Some(vec![("-", "fopt")]),
                        true,
                    )?;
                    check_opt_val(
                        ser,
                        eopt,
                        4,
                        "eopt",
                        Some("--"),
                        Some(vec![true]),
                        true,
                        &ValAction::Set,
                        &ValAssoc::Bool,
                        None,
                        Some(vec![("+", "eopt")]),
                        false,
                    )?;
                    check_opt_val(
                        ser,
                        bopt,
                        1,
                        "bopt",
                        Some("--"),
                        Some(vec![true]),
                        true,
                        &ValAction::Set,
                        &ValAssoc::Bool,
                        None,
                        None,
                        true,
                    )?;
                    check_opt_val(
                        ser,
                        aopt,
                        0,
                        "aopt",
                        Some("--"),
                        Some(vec![false]),
                        true,
                        &ValAction::Set,
                        &ValAssoc::Bool,
                        None,
                        None,
                        false,
                    )?;
                    check_opt_val::<String>(
                        ser,
                        apos,
                        apos_uid,
                        "apos",
                        None,
                        None,
                        true,
                        &ValAction::App,
                        &ValAssoc::Noa,
                        Some(&Index::forward(1)),
                        None,
                        false,
                    )?;
                    Ok(Some(value.take()))
                },
            )
            .with_default();
        policy.parse(Arc::new(args), &mut ser, &mut set)?;
        Ok(())
    }
}
