// containers::default_name_key HEAD: the cabinet's and basket's default
// names (`TextUtils.container("cabinet")`, `("basket")`).
fn farmersdelight_container_name(block: &str) -> core::ops::ControlFlow<Option<&'static str>> {
    if block.starts_with("farmersdelight:") && block.ends_with("_cabinet") {
        return core::ops::ControlFlow::Break(Some("container.farmersdelight.cabinet"));
    }
    if matches!(block, "farmersdelight:wooden_basket" | "farmersdelight:bamboo_basket") {
        return core::ops::ControlFlow::Break(Some("container.farmersdelight.basket"));
    }
    core::ops::ControlFlow::Continue(())
}
