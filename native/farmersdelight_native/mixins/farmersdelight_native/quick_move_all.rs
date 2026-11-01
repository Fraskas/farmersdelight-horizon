// remote::Game::remote_quick_move_all HEAD: a Shift double click on
// CookingPotScreen quick-moves each matching slot of the clicked side, every
// one predicted with the pot menu's own `doClick` and sent.
fn farmersdelight_quick_move_all(
    game: &mut Game<'_>,
    slot: usize,
    moved: &ItemStack,
) -> core::ops::ControlFlow<()> {
    use core::ops::ControlFlow::{Break, Continue};
    if !game.fd_pot_screen() {
        return Continue(());
    }
    let (container, menu) = game.remote_menu();
    let Some(clicked) = Game::menu_slot_index(&menu, slot) else {
        return Break(());
    };
    let state = game.remote.as_ref().map_or(0, |r| r.state_id);
    for (slot, changed, carried) in game.fd_pot_quick_move_all(clicked, moved) {
        let changed = changed
            .iter()
            .map(|(i, s)| (*i, game.to_network(s).map(|s| Hashed::of(&s))))
            .collect();
        let carried = game.to_network(&carried).map(|s| Hashed::of(&s));
        game.send(S::ContainerClick(play::ContainerClick {
            container,
            state,
            slot,
            button: 0,
            input: input_ordinal(Input::QuickMove),
            changed,
            carried,
        }));
    }
    Break(())
}
