// block_entity_anims::Game::container_openers_changed HEAD: a cabinet's
// openers counter (its `open` state); a basket has none.
fn farmersdelight_container_openers(
    game: &mut Game<'_>,
    key: (i32, i32, i32),
    open: bool,
) -> core::ops::ControlFlow<bool> {
    match game.fd_container_openers(key, open) {
        Some(edge) => core::ops::ControlFlow::Break(edge),
        None => core::ops::ControlFlow::Continue(()),
    }
}
