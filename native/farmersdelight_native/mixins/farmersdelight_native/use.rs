// Game::use_hand HEAD (pinned): one hand's turn of a right click. A Farmer's
// Delight block reacts first, as its Java block does: the cutting board's
// sneak carving (`ToolCarvingEvent`, a RightClickBlock event) and its
// `useItemOn`. The click's placement context is noted for the FD block items.
fn farmersdelight_use(
    game: &mut crate::game::Game<'_>,
    block_hit: Option<&Hit>,
) -> core::ops::ControlFlow<Result<bool>> {
    let sneaking = game.keys.contains(&KeyCode::ShiftLeft);
    let hit = block_hit.map(|h| (h.pos, h.face, h.point));
    match game.fd_use_hand(hit, sneaking) {
        Some(done) => core::ops::ControlFlow::Break(Ok(done)),
        None => core::ops::ControlFlow::Continue(()),
    }
}
