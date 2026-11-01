// signs::wood_type HEAD: a canvas sign's "wood" is its canvas
// (`CanvasSign.getBackgroundColor`): the sign editor draws
// `gui/[hanging_]signs/canvas[_color]`, as the mod's edit screens do.
fn farmersdelight_wood_type(name: &str) -> core::ops::ControlFlow<&str> {
    const CANVASES: [&str; 17] = [
        "canvas", "canvas_white", "canvas_orange", "canvas_magenta", "canvas_light_blue",
        "canvas_yellow", "canvas_lime", "canvas_pink", "canvas_gray", "canvas_light_gray",
        "canvas_cyan", "canvas_purple", "canvas_blue", "canvas_brown", "canvas_green",
        "canvas_red", "canvas_black",
    ];
    let Some(path) = name.strip_prefix("farmersdelight:") else {
        return core::ops::ControlFlow::Continue(());
    };
    let Some(color) = ["wall_hanging_canvas_sign", "hanging_canvas_sign", "canvas_wall_sign", "canvas_sign"]
        .iter()
        .find_map(|k| path.strip_suffix(k))
        .map(|c| c.trim_end_matches('_'))
    else {
        return core::ops::ControlFlow::Continue(());
    };
    let wanted = if color.is_empty() { "canvas".to_string() } else { format!("canvas_{color}") };
    match CANVASES.iter().find(|c| **c == wanted) {
        Some(c) => core::ops::ControlFlow::Break(c),
        None => core::ops::ControlFlow::Continue(()),
    }
}