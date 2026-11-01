// native_hooks::Game::native_tick (a hook point): the cutting boards' stored
// stacks are published from the world's block entities, and the window draws
// them as `CuttingBoardRenderer` does.
fn farmersdelight_tick(game: &mut crate::game::Game<'_>) {
    game.fd_board_tick();
}
