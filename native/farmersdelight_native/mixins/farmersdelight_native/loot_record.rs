// spill::Pending::record_saved HEAD: a broken skillet's item, read from its
// block entity before the block's loot is rolled (`CopySkilletFunction`).
fn farmersdelight_loot_record(pos: [i32; 3], tag: &horizon_world::nbt::Tag) {
    crate::game::farmersdelight::skillet_loot_record(pos, tag);
}
