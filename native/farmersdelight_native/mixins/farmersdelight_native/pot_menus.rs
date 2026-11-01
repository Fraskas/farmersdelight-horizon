// lan_stations::Game::lan_stations_tick HEAD: the players' cooking pot menus
// each tick (opened, `stillValid`, `broadcastChanges`).
fn farmersdelight_pot_menus(game: &mut Game<'_>, host: &mut LanHost) {
    game.fd_pot_menus_tick(host);
}
