// stations::Game::draw_slot_tooltip HEAD: CookingPotScreen draws the hovered
// slot's tooltip itself (`renderMealDisplayTooltip`: the meal's name in its
// rarity's colour and, with a container, "Served on"), then the heat icon's
// (`renderHeatIndicatorTooltip`).
fn farmersdelight_slot_tooltip(game: &mut Game<'_>, batch: &mut HudBatch) -> core::ops::ControlFlow<()> {
    use core::ops::ControlFlow::{Break, Continue};
    use horizon_client::text::Text;
    if !game.fd_pot_screen() {
        return Continue(());
    }
    let (mx, my) = game.cursor;
    if game.carried.is_none()
        && let Some(slot) = game.slot_at(batch, mx, my)
        && let Some(stack) = game.slot_view(slot)
    {
        if slot == 306 {
            let mut lines = vec![(
                Text::literal(game.pack.names.display_item(&stack.item)),
                game.rarity_color(&stack),
            )];
            if let Some(container) = game.fd_pot_screen_container() {
                let name = game.pack.names.display_item(&container);
                let line = game
                    .translate("container.farmersdelight.cooking_pot.served_on", "Served on: %s")
                    .replacen("%s", &name, 1);
                lines.push((Text::literal(line), rgba(170, 170, 170, 255)));
            }
            game.draw_tooltip_lines(batch, &lines, mx, my);
        } else {
            game.draw_stack_tooltip(batch, &stack, mx, my);
        }
    }
    let origin = game.inventory_origin(batch);
    if game.fd_pot_over_heat_icon(origin) {
        let key = game.fd_pot_heat_key();
        let fallback = if key.ends_with("not_heated") {
            "Needs heat from below"
        } else {
            "Heated"
        };
        let line = Text::literal(game.translate(key, fallback));
        game.draw_tooltip_lines(batch, &[(line, u32::MAX)], mx, my);
    }
    Break(())
}
