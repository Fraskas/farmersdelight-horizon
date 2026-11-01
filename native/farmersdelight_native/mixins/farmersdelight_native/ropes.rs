// Blocks that follow their neighbours (`updateShape`): every block write
// (`World::set`) beside a rope fence, or that makes one, and every write
// above rice or below rice panicles, queues its position with the world it
// was written in; the game's tick gives the rope fences around it their
// connections again, and the rice its `supporting` and its panicles.
use std::sync::{Mutex, RwLock};

/// The rope fence's block states and its gate's, sorted (set by the game
/// once its tables are loaded).
pub static FENCES: RwLock<Vec<u32>> = RwLock::new(Vec::new());
pub static GATES: RwLock<Vec<u32>> = RwLock::new(Vec::new());
/// The rice's states and the rice panicles', sorted.
pub static RICE: RwLock<Vec<u32>> = RwLock::new(Vec::new());
pub static PANICLES: RwLock<Vec<u32>> = RwLock::new(Vec::new());
/// The tatami's and the tatami mats' states: a write beside one may break
/// a mat or unpair a tatami.
pub static TATAMI: RwLock<Vec<u32>> = RwLock::new(Vec::new());
/// Positions written beside a rope fence or a rice crop, with the address
/// of the world written (the server's world and the window's are both here).
pub static DIRTY: Mutex<Vec<(usize, [i32; 3])>> = Mutex::new(Vec::new());

fn has(list: &RwLock<Vec<u32>>, state: u32) -> bool {
    list.read().is_ok_and(|l| l.binary_search(&state).is_ok())
}

/// `World::set` of `state` at `p`, before the write.
pub fn changed(world: &super::World, p: [i32; 3], state: horizon_world::registry::BlockStateId) {
    let Ok(fences) = FENCES.read() else {
        return;
    };
    if fences.is_empty() {
        return;
    }
    let becomes = fences.binary_search(&state.0).is_ok() || has(&GATES, state.0);
    let near = [[0, -1], [1, 0], [0, 1], [-1, 0]]
        .iter()
        .any(|d| fences.binary_search(&world.block(p[0] + d[0], p[1], p[2] + d[1]).0).is_ok());
    let crop = has(&RICE, world.block(p[0], p[1] - 1, p[2]).0)
        || has(&PANICLES, world.block(p[0], p[1] + 1, p[2]).0);
    let tatami = [[0, 1, 0], [0, -1, 0], [1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1]]
        .iter()
        .any(|d| has(&TATAMI, world.block(p[0] + d[0], p[1] + d[1], p[2] + d[2]).0));
    if (becomes || near || crop || tatami)
        && let Ok(mut dirty) = DIRTY.lock()
        && dirty.len() < 8192
    {
        dirty.push((world as *const super::World as usize, p));
    }
}

/// The window's predicted break of a block holding water (`MultiPlayerGameMode
/// .destroyBlock` leaves `fluidState.createLegacyBlock()`): its position and
/// the water state its air write becomes.
pub static BREAK_WATER: Mutex<Option<([i32; 3], u32)>> = Mutex::new(None);

/// `World::set`'s HEAD: the predicted break's air is that block's water.
pub fn break_water(
    world: &mut super::World,
    p: [i32; 3],
    state: horizon_world::registry::BlockStateId,
) -> Option<crate::Result<()>> {
    if state != world.air {
        return None;
    }
    let water = BREAK_WATER
        .lock()
        .ok()?
        .take_if(|(at, _)| *at == p)
        .map(|(_, water)| water)?;
    Some(world.set(p[0], p[1], p[2], horizon_world::registry::BlockStateId(water)))
}