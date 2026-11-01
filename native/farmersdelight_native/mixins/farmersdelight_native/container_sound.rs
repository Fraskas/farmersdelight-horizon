// storage::Game::container_edge_sound HEAD: a cabinet's own open and close
// sounds (a basket makes none).
fn farmersdelight_container_sound(
    game: &mut Game<'_>,
    key: (i32, i32, i32),
    open: bool,
) -> core::ops::ControlFlow<()> {
    if game.fd_container_edge_sound(key, open) {
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(())
}
