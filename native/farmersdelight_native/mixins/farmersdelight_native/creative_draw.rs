// Game::draw_item HEAD: the first item the creative screen draws (its first
// tab icon) comes after the panel and the tab buttons; NeoForge's page
// buttons and page number are drawn there, under the stacks and tooltips.
fn farmersdelight_creative_draw(game: &Game<'_>, batch: &mut HudBatch) {
    game.fd_creative_pages_draw(batch);
}
