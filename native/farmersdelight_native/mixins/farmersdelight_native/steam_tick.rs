// particles::ParticleEngine::tick_particle HEAD: a steam particle runs its
// own `SteamParticle.tick`.
fn farmersdelight_steam_tick(p: &mut Particle, level: &ParticleLevel) -> core::ops::ControlFlow<()> {
    if farmersdelight_steam::is_steam(p) {
        farmersdelight_steam::tick(p, level);
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(())
}
