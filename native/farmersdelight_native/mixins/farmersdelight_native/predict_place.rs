// remote::Game::remote_can_place HEAD: the client's prediction of placing a
// Farmer's Delight block, run as the game runs it (BlockItem.place on a
// disposable level), with the placed state the block's getStateForPlacement
// gives: the window never draws the block in its default state first.
fn farmersdelight_can_place(
    game: &mut Game<'_>,
    hit: &super::Hit,
    stack: &ItemStack,
) -> core::ops::ControlFlow<Option<DryPlace>> {
    use core::ops::ControlFlow::{Break, Continue};
    if !stack.item.starts_with("farmersdelight:") || !game.may_build() {
        return Continue(());
    }
    // SkilletItem.place: only when sneaking; otherwise the skillet is used.
    if &*stack.item == "farmersdelight:skillet" && !game.keys.contains(&KeyCode::ShiftLeft) {
        return Break(None);
    }
    // RiceBlock, WildRiceBlock: only into water; BuddingBushBlock: on farmland.
    if !game.fd_crop_predict_place(hit.pos, hit.face, &stack.item) {
        return Break(None);
    }
    // RopeItem.updatePlacementContext: rope clicked into rope goes below it.
    if &*stack.item == "farmersdelight:rope" {
        let sneaking = game.keys.contains(&KeyCode::ShiftLeft);
        match game.fd_rope_target(hit.pos, hit.face, sneaking) {
            Some(Some((p, face))) => {
                let at = horizon_core::position::BlockPos::new(p[0], p[1], p[2]);
                return Break(game.fd_rope_state(p, face).map(|s| (at, vec![(at, s)])));
            }
            Some(None) => return Break(None),
            None => {}
        }
    }
    let ctx = horizon_world::level::PlaceContext {
        clicked_pos: horizon_core::position::BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]),
        face: horizon_core::geometry::Direction::ALL[face_direction(hit.face) as usize],
        hit: Vec3::new(hit.point[0], hit.point[1], hit.point[2]),
        inside: hit.face == [0; 3],
        yaw: game.yaw,
        pitch: game.pitch,
        sneaking: game.keys.contains(&KeyCode::ShiftLeft),
        item: stack.item.to_string(),
        can_use_game_master_blocks: game.can_use_game_master_blocks(),
    };
    let placed: Option<DryPlace> = game.with_level(|b, level, _| {
        let mut level = DryLevel::new(level);
        match b.place_block(
            &mut level,
            &mut horizon_world::level::Ticks::default(),
            &ctx,
        ) {
            horizon_world::level::PlaceOutcome::Placed { pos, .. } => {
                Some((pos, level.changed.into_iter().collect()))
            }
            horizon_world::level::PlaceOutcome::Failed => None,
        }
    });
    let Some((pos, mut changes)) = placed else {
        return Break(None);
    };
    let mut placed_state = None;
    for (p, state) in changes.iter_mut() {
        if *p == pos {
            *state = game.fd_predict_placement([p.x, p.y, p.z], *state, hit.face);
            placed_state = Some(*state);
        }
    }
    // The rope fences beside it take their new connections with it.
    if let Some(state) = placed_state {
        for (q, next) in game.fd_rope_predict([pos.x, pos.y, pos.z], state) {
            changes.push((horizon_core::position::BlockPos::new(q[0], q[1], q[2]), next));
        }
        // setPlacedBy's second block: the full tatami mat's head, a tatami's pair.
        let sneaking = game.keys.contains(&KeyCode::ShiftLeft);
        for (q, next) in game.fd_placed_by([pos.x, pos.y, pos.z], state, sneaking) {
            changes.push((horizon_core::position::BlockPos::new(q[0], q[1], q[2]), next));
        }
    }
    Break(Some((pos, changes)))
}
