// The tooltip hook point (API 12): the mod's `appendHoverText` lines
// (`ConsumableItem`'s effects, the drinks' lines, `PlaceableItem`,
// `DogFoodItem`, `HorseFeedItem`, the skillet) under the item's name.
fn farmersdelight_tooltip(game: &Game<'_>, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    game.fd_tooltip(stack, lines);
}