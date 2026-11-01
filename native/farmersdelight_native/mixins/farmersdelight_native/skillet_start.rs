// remote::Game::remote_start_held_use HEAD: the window's own
// `SkilletItem.use`, starting the use as the server will. It never FAILs
// (it starts cooking or passes), so the hook answers true.
fn farmersdelight_skillet_start(game: &mut Game<'_>, held: &ItemStack) -> core::ops::ControlFlow<bool> {
    let Some(start) = game.fd_skillet_predict_use(held) else {
        return core::ops::ControlFlow::Continue(());
    };
    if start && game.using.is_none() {
        game.using = Some((game.held_slot(), 0));
    }
    core::ops::ControlFlow::Break(true)
}
