// creative::Creative::tab_position HEAD: a tab not on the current page has
// no button (drawn, hovered or clicked) anywhere on the screen.
fn farmersdelight_tab_position(tab: &Tab) -> core::ops::ControlFlow<(f32, f32)> {
    if !farmersdelight_pages::visible(&tab.id) {
        return core::ops::ControlFlow::Break((-100_000.0, -100_000.0));
    }
    core::ops::ControlFlow::Continue(())
}
