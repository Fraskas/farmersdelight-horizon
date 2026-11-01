// lan_stations::Game::lan_station_click HEAD: a click on a player's cooking
// pot menu (`handleContainerClick`).
fn farmersdelight_pot_click(
    game: &mut Game<'_>,
    host: &mut LanHost,
    conn: u32,
    p: &play::ContainerClick,
) -> core::ops::ControlFlow<bool> {
    if game.fd_pot_menu_click(host, conn, p) {
        return core::ops::ControlFlow::Break(true);
    }
    core::ops::ControlFlow::Continue(())
}
