//! Tests for the [`keyed::Column`] widget.
//!
//! [`keyed::Column`]: iced_widget::keyed::Column
use iced_test::Simulator;
use iced_widget::{Component, Renderer, Widget, button, component, keyed_column, text};

/// A button that counts how many times it has been pressed in its own state.
struct Counter(&'static str);

impl<'a> Component<'a, ()> for Counter {
    type State = u32;
    type Event = ();

    fn update(&self, count: &mut u32, _event: (), _renderer: &Renderer) -> Option<()> {
        *count += 1;
        None
    }

    fn view(&self, count: &u32) -> impl Widget<()> + 'a {
        button(text(format!("{} {count}", self.0))).on_press(())
    }
}

fn view(keys: &[&'static str]) -> impl Widget<()> + 'static {
    keyed_column(keys.iter().map(|&key| (key, component(Counter(key)))))
}

/// Presses the counter of `key` once, rebuilds the column with `after` and
/// checks that the state stayed with `key`.
fn assert_state_follows_key(before: &[&'static str], after: &[&'static str], key: &str) {
    let mut simulator = Simulator::new(view(before));

    let _ = simulator.click(format!("{key} 0").as_str()).unwrap();
    assert!(simulator.find(format!("{key} 1").as_str()).is_ok());

    let mut simulator = simulator.rebuild(view(after));

    for other in after {
        let expected = if *other == key { 1 } else { 0 };

        assert!(
            simulator
                .find(format!("{other} {expected}").as_str())
                .is_ok(),
            "{other} should have a count of {expected} after rebuilding with {after:?}"
        );
    }
}

#[test]
fn state_is_kept_when_pushing() {
    assert_state_follows_key(&["a"], &["a", "b"], "a");
    assert_state_follows_key(&["a", "b"], &["a", "b", "c"], "b");
}

#[test]
fn state_is_kept_when_inserting_in_the_middle() {
    assert_state_follows_key(&["a", "c"], &["a", "b", "c"], "c");
    assert_state_follows_key(&["a", "c"], &["a", "b", "c"], "a");
}

#[test]
fn state_is_kept_when_inserting_at_the_start() {
    assert_state_follows_key(&["b"], &["a", "b"], "b");
}
