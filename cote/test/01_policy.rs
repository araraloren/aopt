use cote::*;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let widget = Widget::into_policy();

    assert!(check_t::<FwdPolicy<'_, ASet, ASer>>(&widget));

    let windows = Windows::into_policy();

    assert!(check_t::<DelayPolicy<'_, ASet, ASer>>(&windows));

    let button = Button::into_policy();

    assert!(check_t::<PrePolicy<'_, ASet, ASer>>(&button));

    let line_editor = LineEditor::into_policy();

    assert!(check_t::<NullPolicy<'_, ASet, ASer>>(&line_editor));

    Ok(())
}

fn check_t<T: 'static>(w: &dyn Any) -> bool {
    w.is::<T>()
}
