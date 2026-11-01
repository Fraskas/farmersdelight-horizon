// spill::Pending::attach HEAD: `CopySkilletFunction`, the skillet's drop is
// the item it was placed from (its enchantments and wear).
fn farmersdelight_loot_attach(pos: [i32; 3], stack: &mut ItemStack) {
    crate::game::farmersdelight::skillet_loot_attach(pos, stack);
}
