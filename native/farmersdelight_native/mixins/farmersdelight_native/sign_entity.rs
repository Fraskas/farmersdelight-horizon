// lan::chunks::block_entity_type HEAD: the canvas signs' block entities are
// vanilla's sign and hanging sign (`ModBlockEntityTypes.CANVAS_SIGN` and
// `HANGING_CANVAS_SIGN` serialize as them), so the window gets their text.
fn farmersdelight_sign_entity(name: &str) -> core::ops::ControlFlow<Option<&'static str>> {
    match name.strip_prefix("farmersdelight:") {
        Some(path) if path.contains("canvas") && path.ends_with("sign") => {
            core::ops::ControlFlow::Break(Some(if path.contains("hanging") {
                "minecraft:hanging_sign"
            } else {
                "minecraft:sign"
            }))
        }
        _ => core::ops::ControlFlow::Continue(()),
    }
}