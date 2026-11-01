// machines::Game::draw_machine_screen HEAD: CookingPotScreen's background and
// labels.
fn farmersdelight_draw_screen(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    ox: f32,
    oy: f32,
) -> core::ops::ControlFlow<bool> {
    if game.fd_pot_draw(batch, ox, oy) {
        return core::ops::ControlFlow::Break(true);
    }
    core::ops::ControlFlow::Continue(())
}
