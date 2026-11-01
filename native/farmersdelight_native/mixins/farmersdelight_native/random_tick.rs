// The random_tick hook point (API 33): the mod's crops grow their own way
// (`RiceBlock`, `RicePaniclesBlock`, `BuddingTomatoBlock`, `TomatoBlock`) in
// vanilla's random ticks, instead of their template's.
fn farmersdelight_random_tick(
    level: &mut GameLevel<'_>,
    ticks: &mut Ticks,
    pos: BlockPos,
    state: BlockStateId,
    handled: &mut bool,
) {
    if !*handled && level.fd_random_tick(ticks, pos, state) {
        *handled = true;
    }
}