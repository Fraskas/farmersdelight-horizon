// machines::Game::screen_height HEAD: CookingPotScreen's imageHeight (166).
fn farmersdelight_screen_height(game: &Game<'_>) -> core::ops::ControlFlow<f32> {
    if game.fd_pot_screen() {
        return core::ops::ControlFlow::Break(166.0);
    }
    core::ops::ControlFlow::Continue(())
}
