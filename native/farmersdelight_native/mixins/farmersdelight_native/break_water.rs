// World::set HEAD: the window's predicted break of a Farmer's Delight block
// holding water writes the water, not air (as vanilla's client does).
fn farmersdelight_break_water(
    world: &mut World,
    x: i32,
    y: i32,
    z: i32,
    state: BlockStateId,
) -> core::ops::ControlFlow<crate::Result<()>> {
    match farmersdelight_ropes::break_water(world, [x, y, z], state) {
        Some(done) => core::ops::ControlFlow::Break(done),
        None => core::ops::ControlFlow::Continue(()),
    }
}