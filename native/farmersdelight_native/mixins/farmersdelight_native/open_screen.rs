// remote::Game::remote_open_screen HEAD: ClientboundOpenScreenPacket for
// `farmersdelight:cooking_pot` opens CookingPotScreen. Its nine slots are kept
// as the open container's (as a dispenser menu's are), under the key no block
// has; the screen's block is the clicked one.
fn farmersdelight_open_screen(game: &mut Game<'_>, p: &play::OpenScreen) -> core::ops::ControlFlow<()> {
    use core::ops::ControlFlow::{Break, Continue};
    let menu = game
        .remote
        .as_ref()
        .and_then(|r| r.builtin.get("minecraft:menu")?.name(p.menu).map(str::to_owned))
        .unwrap_or_default();
    if menu != crate::game::farmersdelight::POT_MENU {
        return Continue(());
    }
    game.recipe_ui.reset_screen();
    let target = game.remote.as_ref().and_then(|r| r.use_target);
    game.fd_pot_screen_opened(target);
    let title = menu_title(&p.title);
    if let Some(r) = game.remote.as_mut() {
        r.menu = Some((p.container, menu));
        r.menu_title = Some(title);
    }
    game.craft_size = 2;
    let key = (i32::MIN, 0, 0);
    game.world
        .block_entities
        .insert(key, BlockEntity::Chest(vec![None; 9]));
    game.container = Some(key);
    game.screen = Screen::Inventory;
    game.capture(false);
    Break(())
}
