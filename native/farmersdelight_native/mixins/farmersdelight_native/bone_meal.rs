// growth::Game::use_bone_meal HEAD: bone meal on the mod's crops (their
// `BonemealableBlock`: rice and its panicles, budding tomatoes, tomatoes,
// wild rice).
fn farmersdelight_bone_meal(game: &mut Game<'_>, stack: &ItemStack) -> core::ops::ControlFlow<Result<bool>> {
    match game.fd_bone_meal(stack) {
        Some(used) => core::ops::ControlFlow::Break(Ok(used)),
        None => core::ops::ControlFlow::Continue(()),
    }
}
