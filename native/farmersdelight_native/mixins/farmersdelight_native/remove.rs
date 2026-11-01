// spill::GameLevel::remove_saved_entity HEAD: a Farmer's Delight block
// entity goes with its block, and its block's onRemove drops what it held
// with `Containers.dropItemStack`: the cutting board's stored stack
// (`CuttingBoardBlock.onRemove`), each of the stove's items
// (`AbstractStoveBlock.onRemove`, `ItemUtils.dropItems`).
fn farmersdelight_remove(
    level: &mut GameLevel<'_>,
    pos: BlockPos,
    old: BlockStateId,
    side_effects: bool,
) -> core::ops::ControlFlow<bool> {
    use core::ops::ControlFlow::{Break, Continue};
    use horizon_world::nbt::Tag;
    let name = level.facts.get(old).name.clone();
    if !matches!(
        &*name,
        "farmersdelight:cutting_board"
            | "farmersdelight:stove"
            | "farmersdelight:cooking_pot"
            | "farmersdelight:skillet"
    ) {
        return Continue(());
    }
    let at = |t: &Tag| {
        t.get("x") == Some(&Tag::Int(pos.x))
            && t.get("y") == Some(&Tag::Int(pos.y))
            && t.get("z") == Some(&Tag::Int(pos.z))
    };
    let Some(chunk) = level.world.chunks.get_mut(&(pos.x >> 4, pos.z >> 4)) else {
        return Break(false);
    };
    let chunk = std::sync::Arc::make_mut(chunk);
    let Some(saved) = chunk.saved.as_mut() else {
        return Break(false);
    };
    let saved = std::sync::Arc::make_mut(saved);
    let Some(index) = saved.block_entities.iter().position(at) else {
        return Break(false);
    };
    let tag = saved.block_entities.remove(index);
    if side_effects {
        if &*name == "farmersdelight:skillet" {
            // SkilletBlock.onRemove: Containers.dropItemStack of its food.
            level.drop_item_stack(pos, crate::game::farmersdelight::skillet_food(&tag));
        } else if &*name == "farmersdelight:cutting_board" {
            level.drop_item_stack(pos, crate::game::farmersdelight::board_stack(&tag));
        } else if &*name == "farmersdelight:cooking_pot" {
            // CookingPotBlock.onRemove: Containers.dropContents of all but the
            // meal, then getUsedRecipesAndPopExperience at the block's center.
            let (drops, xp) = crate::game::farmersdelight::pot_drops(&tag);
            for stack in drops {
                level.drop_item_stack(pos, Some(stack));
            }
            if xp > 0 {
                let center = [
                    f64::from(pos.x) + 0.5,
                    f64::from(pos.y) + 0.5,
                    f64::from(pos.z) + 0.5,
                ];
                level.award_orbs(center, xp);
            }
        } else {
            for stack in crate::game::farmersdelight::stove_items(&tag) {
                level.drop_item_stack(pos, stack);
            }
        }
    }
    Break(true)
}
