// Game::click_creative HEAD: a left click on a page button turns the page.
fn farmersdelight_creative_click(game: &mut Game<'_>, right: bool) -> core::ops::ControlFlow<()> {
    if !right && game.fd_creative_pages_click() {
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(())
}
