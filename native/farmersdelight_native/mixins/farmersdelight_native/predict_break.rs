// remote::Game::predict_break HEAD: a Farmer's Delight block holding water
// (rice, wild rice, the waterlogged ones) leaves its water in the window's
// prediction of the break.
fn farmersdelight_predict_break(game: &mut Game<'_>, pos: [i32; 3], state: BlockStateId) {
    game.fd_predict_break(pos, state);
}