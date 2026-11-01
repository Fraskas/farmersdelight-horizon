// level_particles::Game::growth_particles HEAD: the mod's crops that grow with
// bone meal (`RiceBlock`, `RicePaniclesBlock`, `BuddingTomatoBlock`,
// `WildRiceBlock`: `BonemealableBlock.Type.GROWER`) spark inside the block
// (`ParticleUtils.spawnParticleInBlock`), as vanilla's growers do.
fn farmersdelight_growth_particles(game: &mut Game<'_>, pos: BlockPos, count: i32) -> core::ops::ControlFlow<()> {
    let state = game.world.block(pos.x, pos.y, pos.z);
    let grower = matches!(
        &*game.res.facts().get(state).name,
        "farmersdelight:rice"
            | "farmersdelight:rice_panicles"
            | "farmersdelight:budding_tomatoes"
            | "farmersdelight:wild_rice"
    );
    if !grower {
        return core::ops::ControlFlow::Continue(());
    }
    let at = [pos.x, pos.y, pos.z];
    let top = game.block_shape_top(at);
    game.spawn_particles(at, count, 0.5, top, true, ParticleType::HappyVillager);
    core::ops::ControlFlow::Break(())
}