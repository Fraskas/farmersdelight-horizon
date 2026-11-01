// remote_stations::Game::remote_station_data HEAD: CookingPotMenu's data
// slots (the cook time and its total, for the progress arrow).
fn farmersdelight_station_data(game: &mut Game<'_>, id: usize, value: i16) -> core::ops::ControlFlow<()> {
    if game.fd_pot_screen() {
        game.fd_pot_screen_data(id, value);
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(())
}
