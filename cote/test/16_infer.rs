use cote::{aopt::value::AnyValue, *};
use std::{any::TypeId, fmt::Debug};

#[derive(Debug, PartialEq, Eq, CoteOpt)]
#[infer(val = i32, action = Action::App)]
#[infer(init = Some(ValInitializer::new_value(42i32)))]
pub struct Speed(i32);

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    assert_eq!(
        TypeId::of::<<Speed as Infer>::Val>(),
        TypeId::of::<i32>(),
        "same type"
    );
    assert_eq!(<Speed as Infer>::infer_act(), Action::App);
    assert_eq!(<Speed as Infer>::infer_force(), false);
    assert_eq!(<Speed as Infer>::infer_ctor(), ctor_default_name());
    assert_eq!(<Speed as Infer>::infer_index(), None);
    assert_eq!(<Speed as Infer>::infer_style(), vec![Style::Argument]);
    assert_eq!(<Speed as Infer>::infer_ignore_name(), false);
    assert_eq!(<Speed as Infer>::infer_ignore_index(), true);
    assert_eq!(<Speed as Infer>::infer_ignore_alias(), false);
    assert!(<Speed as Infer>::infer_validator().is_none());
    assert_eq!(<Speed as Infer>::infer_type_id(), TypeId::of::<i32>());
    check_initializer(<Speed as Infer>::infer_initializer(), 42)?;
    Ok(())
}

fn check_initializer<T: PartialEq + Debug + ErasedTy>(
    init: Option<ValInitializer>,
    val: T,
) -> color_eyre::Result<()> {
    assert!(init.is_some());
    if let Some(mut init) = init {
        let mut any_value = AnyValue::new();

        // put the value into AnyValue
        init.invoke(&mut any_value)?;
        assert_eq!(any_value.val::<T>()?, &val);
    }
    Ok(())
}
