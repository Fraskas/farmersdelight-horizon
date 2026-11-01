// lan_stations::Game::lan_station_closed HEAD: a player's cooking pot menu
// closes with the window it names, or with any other menu that opens (None,
// and the stations close too).
fn farmersdelight_pot_closed(
    game: &mut Game<'_>,
    host: &mut LanHost,
    conn: u32,
    window: Option<i32>,
) -> core::ops::ControlFlow<bool> {
    if game.fd_pot_menu_closed(host, conn, window) && window.is_some() {
        return core::ops::ControlFlow::Break(true);
    }
    core::ops::ControlFlow::Continue(())
}
