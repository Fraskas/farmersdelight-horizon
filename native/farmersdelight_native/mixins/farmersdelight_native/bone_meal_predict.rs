// remote::Game::remote_bone_meal_succeeds HEAD: the window's bone meal on
// the mod's crops succeeds where their `isValidBonemealTarget` says.
fn farmersdelight_bone_meal_predict(game: &mut Game<'_>, hit: &super::Hit) -> core::ops::ControlFlow<bool> {
    match game.fd_bone_meal_predict(hit.pos) {
        Some(ok) => core::ops::ControlFlow::Break(ok),
        None => core::ops::ControlFlow::Continue(()),
    }
}
