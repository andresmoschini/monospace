fn main() {
    let mut a = state_standalone::App::default();
    a.apply(state_standalone::Action::Increment);
    a.apply(state_standalone::Action::ToggleModal);
    println!("{a:?}");
    assert_eq!(state_standalone::action_for_key('+'), Some(state_standalone::Action::Increment));
    println!("state.rs compiles and runs with ZERO dependencies");
}
