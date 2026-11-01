// Game::release_used_item HEAD: `SkilletItem.releaseUsing`, the food back.
fn farmersdelight_skillet_release(game: &mut Game<'_>, stack: &ItemStack) {
    game.fd_skillet_released(stack);
}
