// advancements::Game::player_hit HEAD (a hit that landed): the skillet's
// attack sound (`SkilletItem.SkilletEvents.playSkilletAttackSound`).
fn farmersdelight_skillet_hit(game: &mut Game<'_>, kind: &str, direct: &Option<EntityView>) {
    game.fd_skillet_hit(kind, direct.is_some());
}
