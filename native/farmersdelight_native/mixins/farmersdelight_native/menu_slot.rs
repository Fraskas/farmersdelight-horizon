// remote::Game::menu_slot HEAD: CookingPotMenu's slots are the pot's nine
// (the open container's), the inventory's 27, then the hotbar.
fn farmersdelight_menu_slot(menu: &str, slot: i32) -> core::ops::ControlFlow<Option<usize>> {
    use core::ops::ControlFlow::{Break, Continue};
    if menu != crate::game::farmersdelight::POT_MENU {
        return Continue(());
    }
    Break(match usize::try_from(slot) {
        Ok(s @ 0..9) => Some(300 + s),
        Ok(s @ 9..36) => Some(s),
        Ok(s @ 36..45) => Some(s - 36),
        _ => None,
    })
}
