// The update_shape hook point (API 32): the mod's blocks answer a
// neighbour's change as their Java `updateShape` does (rope fence sides,
// tatami pairs and mats, rice under its panicles).
fn farmersdelight_update_shape(
    level: &GameLevel<'_>,
    pos: BlockPos,
    state: BlockStateId,
    direction: Direction,
    neighbour: BlockStateId,
    new: &mut Option<BlockStateId>,
) {
    if new.is_none() {
        *new = level.fd_update_shape(pos, state, direction, neighbour);
    }
}