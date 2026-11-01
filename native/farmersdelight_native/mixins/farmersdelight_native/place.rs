// block_items::Game::apply_item_block_state HEAD: a Farmer's Delight block
// just placed gets the state its `getStateForPlacement` gives (facing the
// player, waterlogged in water...), from the click noted before placement.
fn farmersdelight_place(
    game: &mut Game<'_>,
    pos: [i32; 3],
    state: BlockStateId,
    stack: &ItemStack,
) -> core::ops::ControlFlow<Result<BlockStateId>> {
    match game.fd_placement(pos, state, stack) {
        Some(placed) => core::ops::ControlFlow::Break(placed),
        None => core::ops::ControlFlow::Continue(()),
    }
}
