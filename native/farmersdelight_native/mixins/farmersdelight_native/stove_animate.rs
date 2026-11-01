// A module of animate_tick.rs: `StoveBlock.animateTick` of Farmer's Delight,
// for a stove `ClientLevel.animateTick` picked. A lit stove crackles one
// time in ten, and smokes and flames at its front at a random spot.
use super::{AnimateContext, T, add, sound};
use crate::particles::ParticleEngine;
use horizon_core::random::RandomSource;
use horizon_world::registry::BlockStateId;

pub(super) fn stove(engine: &mut ParticleEngine, ctx: &AnimateContext, state: BlockStateId, pos: [i32; 3]) {
    let Some(s) = ctx.registry.state(state) else {
        return;
    };
    if &*s.name != "farmersdelight:stove" || s.properties.get("lit").is_none_or(|v| &**v != "true") {
        return;
    }
    let x = f64::from(pos[0]) + 0.5;
    let y = f64::from(pos[1]);
    let z = f64::from(pos[2]) + 0.5;
    if engine.random().next_int_bound(10).unwrap_or(1) == 0 {
        sound(engine, "farmersdelight:block.stove.crackle", [x, y, z], 1.0, 1.0);
    }
    let (step_x, step_z) = match s.properties.get("facing").map(|v| &**v) {
        Some("north") => (0.0, -1.0),
        Some("south") => (0.0, 1.0),
        Some("west") => (-1.0, 0.0),
        _ => (1.0, 0.0),
    };
    let x_axis = step_x != 0.0;
    let random = engine.random();
    let horizontal = random.next_double() * 0.6 - 0.3;
    let x_offset = if x_axis { step_x * 0.52 } else { horizontal };
    let y_offset = random.next_double() * 6.0 / 16.0;
    let z_offset = if x_axis { horizontal } else { step_z * 0.52 };
    let at = [x + x_offset, y + y_offset, z + z_offset];
    add(engine, T::Smoke, at, [0.0; 3]);
    add(engine, T::Flame, at, [0.0; 3]);
}
