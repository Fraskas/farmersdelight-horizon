// World::set HEAD: a block written beside a rope fence (or one becoming a
// rope fence) queues the rope fences' `updateShape`.
fn farmersdelight_rope_changed(world: &World, x: i32, y: i32, z: i32, state: BlockStateId) {
    farmersdelight_ropes::changed(world, [x, y, z], state);
}
