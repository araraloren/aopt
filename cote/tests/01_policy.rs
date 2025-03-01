use cote::prelude::*;
use std::any::Any;

// The default policy is `fwd`(FwdPolicy)
#[derive(Debug, Cote)]
pub struct Widget;

// Configure the policy with built-in `delay`, `pre` or `fwd`
#[derive(Debug, Cote)]
#[cote(policy = delay)]
pub struct Windows;

// Using `pre`(`PrePolicy`) if the struct has sub commands
#[derive(Debug, Cote)]
pub struct Button {
    #[sub()]
    #[allow(unused)]
    wid: Option<Widget>,

    #[sub()]
    #[allow(unused)]
    win: Option<Windows>,
}

// `NullPolicy` is a example policy can used in attribute `cote`
#[derive(Debug, Cote)]
#[cote(policy = NullPolicy)]
pub struct LineEditor;

#[test]
fn policy() {
    assert!(policy_impl().is_ok());
}

fn policy_impl() -> Result<(), Box<dyn std::error::Error>> {
    let widget = Widget::into_policy();

    assert!(check_t::<FwdPolicy<'_, CoteSet, CoteSer>>(&widget));

    let windows = Windows::into_policy();

    assert!(check_t::<DelayPolicy<'_, CoteSet, CoteSer>>(&windows));

    let button = Button::into_policy();

    assert!(check_t::<PrePolicy<'_, CoteSet, CoteSer>>(&button));

    let line_editor = LineEditor::into_policy();

    assert!(check_t::<NullPolicy<'_, CoteSet, CoteSer>>(&line_editor));

    Ok(())
}

fn check_t<T: 'static>(w: &dyn Any) -> bool {
    w.is::<T>()
}
