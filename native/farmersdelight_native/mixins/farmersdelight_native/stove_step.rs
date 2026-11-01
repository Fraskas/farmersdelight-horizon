// motion::MotionWorld::effects_from_blocks HEAD: `AbstractStoveBlock.stepOn`'s
// `burnEntitySteppingOnStove`, where the game runs MagmaBlock's `stepOn`
// (the block under an entity on the ground, `getOnPosLegacy`): a lit stove
// burns a living entity whose box meets its grill (`GRILLING_AREA`, 3..13 x
// 0..1 x 3..13 sixteenths above the stove) unless it steps carefully
// (sneaks), with `ModDamageTypes.STOVE_BURN` for 1.
fn farmersdelight_stove_step(world: &mut MotionWorld<'_, '_, '_>) {
    if world.player.spectator || !world.player.ground.on_ground || world.player.shift_key_down {
        return;
    }
    let Ok(on) = world.legacy_on_position() else {
        return;
    };
    let lit = world.assets.registry.state(world.block(on)).is_some_and(|s| {
        s.name == "farmersdelight:stove"
            && s.properties.get("lit").is_some_and(|l| &**l == "true")
    });
    if !lit {
        return;
    }
    let (x, y, z) = (f64::from(on.x), f64::from(on.y) + 1.0, f64::from(on.z));
    let min = horizon_core::geometry::Vec3::new(x + 3.0 / 16.0, y, z + 3.0 / 16.0);
    let max = horizon_core::geometry::Vec3::new(x + 13.0 / 16.0, y + 1.0 / 16.0, z + 13.0 / 16.0);
    if !world.player.bounds().intersects_corners(min, max) {
        return;
    }
    // Frost walker's `damage_immunity` covers #burn_from_stepping, which the
    // stove's damage type joins.
    if world.player.burn_from_stepping_immune {
        return;
    }
    let _ = world.environment_hurt("farmersdelight:stove_burn", 1.0);
}
