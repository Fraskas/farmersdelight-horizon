// block_entity_render::classify HEAD: the canvas signs' text placement
// (`CanvasSignRenderer`, `HangingCanvasSignRenderer`): hanging or standing,
// on a wall by `facing`, else by `rotation`.
fn farmersdelight_sign_classify(
    name: &str,
    prop: &dyn Fn(&str) -> Option<String>,
) -> core::ops::ControlFlow<Kind> {
    let Some(path) = name.strip_prefix("farmersdelight:") else {
        return core::ops::ControlFlow::Continue(());
    };
    if !(path.contains("canvas") && path.ends_with("sign")) {
        return core::ops::ControlFlow::Continue(());
    }
    let wall = path.contains("wall");
    let angle = if wall {
        y_rot(intern(&prop("facing").unwrap_or_default()))
    } else {
        prop("rotation").and_then(|r| r.parse::<f32>().ok()).unwrap_or(0.0) * 22.5
    };
    core::ops::ControlFlow::Break(Kind::Sign {
        hanging: path.contains("hanging"),
        wall,
        angle,
    })
}