// signs::sign_kind HEAD: the canvas signs (`CanvasSignBlock`,
// `HangingCanvasSignBlock` and their wall forms) are signs, the hanging ones
// by the mod's own names (`hanging_canvas_sign`, not `..._hanging_sign`).
fn farmersdelight_sign_kind(name: &str) -> core::ops::ControlFlow<Option<bool>> {
    match name.strip_prefix("farmersdelight:") {
        Some(path) if path.contains("canvas") && path.ends_with("sign") => {
            core::ops::ControlFlow::Break(Some(path.contains("hanging")))
        }
        _ => core::ops::ControlFlow::Continue(()),
    }
}