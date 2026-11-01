// lan::containers::Hub::damage_event HEAD: the clients' damage type registry
// is the synchronized vanilla one, which has no `farmersdelight:stove_burn`.
// Its damage event goes out as `minecraft:hot_floor`, a type with the same
// client effects (`DamageEffects.BURNING`: the hurt flash and the on-fire
// hurt sound); the death message is the server's, with the stove's own text.
fn farmersdelight_damage_event(
    hub: &Hub,
    id: i32,
    kind: &str,
    cause: Option<i32>,
    direct: Option<i32>,
    at: [f64; 3],
) -> core::ops::ControlFlow<()> {
    if kind == "farmersdelight:stove_burn" {
        hub.damage_event(id, "minecraft:hot_floor", cause, direct, at);
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(())
}
