// remote::Game::remote_block_reacts HEAD: the client's own run of a Farmer's
// Delight block's `useItemOn` (and the cutting board's ToolCarvingEvent),
// as the mod runs it on the client: SUCCESS swings the arm, CONSUME ends the
// click without, and either keeps the client from sending the hand's item use.
fn farmersdelight_reacts(
    game: &mut Game<'_>,
    hit: &super::Hit,
    held: Option<&ItemStack>,
) -> core::ops::ControlFlow<(InteractionResult, DryUse)> {
    let sneaking = game.keys.contains(&KeyCode::ShiftLeft);
    match game.fd_board_predict(hit.pos, held, sneaking) {
        Some(true) => core::ops::ControlFlow::Break((InteractionResult::Success, DryUse::default())),
        Some(false) => core::ops::ControlFlow::Break((InteractionResult::Consume, DryUse::default())),
        None => core::ops::ControlFlow::Continue(()),
    }
}
