// machines::Game::machine_slot_position HEAD: CookingPotMenu's layout.
fn farmersdelight_slot_position(game: &Game<'_>, i: usize) -> core::ops::ControlFlow<Option<(f32, f32)>> {
    if game.fd_pot_screen() {
        return core::ops::ControlFlow::Break(game.fd_pot_slot_position(i));
    }
    core::ops::ControlFlow::Continue(())
}
