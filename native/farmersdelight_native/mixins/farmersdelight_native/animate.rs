// animate_tick::block_animate_tick HEAD: a Farmer's Delight block's own
// `animateTick` (the stove's flames, smoke and crackle) for the block
// `ClientLevel.animateTick` picked.
fn farmersdelight_animate(
    engine: &mut ParticleEngine,
    ctx: &AnimateContext,
    state: BlockStateId,
    pos: [i32; 3],
) {
    farmersdelight_animate::stove(engine, ctx, state, pos);
}
