// potions::Game::tick_drinking HEAD (the first per-item step of an item in
// use, after its release checks): a skillet's `onUseTick` and finish.
fn farmersdelight_skillet_tick(
    game: &mut Game<'_>,
    slot: usize,
    ticks: u32,
    stack: &ItemStack,
) -> core::ops::ControlFlow<bool> {
    if game.fd_skillet_use_tick(slot, ticks, stack) {
        return core::ops::ControlFlow::Break(true);
    }
    core::ops::ControlFlow::Continue(())
}
