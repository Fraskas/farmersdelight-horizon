// creative::item_group_key HEAD: Farmer's Delight's tab is titled
// `itemGroup.farmersdelight`.
fn farmersdelight_group_key(id: &str) -> core::ops::ControlFlow<Option<&'static str>> {
    if id == farmersdelight_pages::TAB {
        return core::ops::ControlFlow::Break(Some("itemGroup.farmersdelight"));
    }
    core::ops::ControlFlow::Continue(())
}
