// remote::Game::remote_menu_ops HEAD: clicks on CookingPotScreen are
// predicted with the pot menu's own `doClick` (its meal and result slots take
// nothing), then each goes out as a ContainerClick with the slots it changed
// and the carried stack after it.
fn farmersdelight_menu_ops(
    game: &mut Game<'_>,
    ops: &[(Option<usize>, i32, Input)],
) -> core::ops::ControlFlow<bool> {
    use core::ops::ControlFlow::{Break, Continue};
    if !game.fd_pot_screen() {
        return Continue(());
    }
    let (container, menu) = game.remote_menu();
    let mut clicks = Vec::with_capacity(ops.len());
    for &(slot, button, input) in ops {
        let slot = match slot {
            None => -999,
            Some(internal) => match Game::menu_slot_index(&menu, internal) {
                Some(s) => s as i16,
                None => return Break(false),
            },
        };
        clicks.push((slot, button, input_ordinal(input)));
    }
    let (predicted, threw) = game.fd_pot_predict(&clicks);
    let state = game.remote.as_ref().map_or(0, |r| r.state_id);
    for (&(slot, button, input), (changed, carried)) in clicks.iter().zip(predicted) {
        let changed = changed
            .iter()
            .map(|(i, s)| (*i, game.to_network(s).map(|s| Hashed::of(&s))))
            .collect();
        let carried = game.to_network(&carried).map(|s| Hashed::of(&s));
        game.send(S::ContainerClick(play::ContainerClick {
            container,
            state,
            slot,
            button: button as i8,
            input,
            changed,
            carried,
        }));
    }
    // LivingEntity.drop(.., Prediction.PREDICTED): the client swings.
    if threw {
        game.swing_hand(Hand::Main, SwingAnimation::DEFAULT);
    }
    Break(true)
}
