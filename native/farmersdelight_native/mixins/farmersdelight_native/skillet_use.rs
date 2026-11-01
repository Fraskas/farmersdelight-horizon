// Game::use_tool HEAD: the skillet in the acting hand (`SkilletItem.place`
// only when sneaking, `SkilletItem.use` near heat), and the crops' items
// where their block can't go (rice out of water, tomato seeds off farmland).
fn farmersdelight_skillet_use(
    game: &mut Game<'_>,
    stack: &ItemStack,
    on_block: bool,
) -> core::ops::ControlFlow<Result<horizon_client::first_person::UseResult>> {
    if let Some(result) = game.fd_crop_item_use(stack, on_block) {
        return core::ops::ControlFlow::Break(Ok(result));
    }
    // RopeItem.updatePlacementContext: rope clicked into rope extends it.
    if let Some(result) = game.fd_rope_item_use(stack, on_block) {
        return core::ops::ControlFlow::Break(Ok(result));
    }
    match game.fd_skillet_item_use(stack, on_block) {
        Some(result) => core::ops::ControlFlow::Break(Ok(result)),
        None => core::ops::ControlFlow::Continue(()),
    }
}
