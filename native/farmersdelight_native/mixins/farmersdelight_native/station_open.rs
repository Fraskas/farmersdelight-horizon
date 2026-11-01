// remote_stations::Game::remote_station_open HEAD: the data slots of an open
// CookingPotScreen are the pot menu's (ContainerSetData goes to
// `remote_station_data`).
fn farmersdelight_station_open(game: &Game<'_>, container: i32) -> core::ops::ControlFlow<bool> {
    if game.fd_pot_screen() && game.remote_menu().0 == container {
        return core::ops::ControlFlow::Break(true);
    }
    core::ops::ControlFlow::Continue(())
}
