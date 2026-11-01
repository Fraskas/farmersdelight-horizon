// remote::Game::held_until_released HEAD: a skillet in use counts its ticks
// on the window until it lets go or the server completes the cooking.
fn farmersdelight_skillet_held(stack: &ItemStack) -> core::ops::ControlFlow<bool> {
    if &*stack.item == "farmersdelight:skillet" {
        return core::ops::ControlFlow::Break(true);
    }
    core::ops::ControlFlow::Continue(())
}
