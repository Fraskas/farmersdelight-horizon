// The destroy_block hook point (API 14): `playerWillDestroy` of the mod's
// blocks (a creative player breaking a full tatami mat's foot takes its
// head too, without its drop).
fn farmersdelight_destroy_block(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if !*canceled {
        game.fd_player_will_destroy(pos);
    }
}