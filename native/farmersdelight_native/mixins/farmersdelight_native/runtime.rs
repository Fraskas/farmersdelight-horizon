// Farmer's Delight mechanics for Horizon, ported from the mod's own source
// (vectorwing/FarmersDelight 1.21.1-1.3.4, MIT).
//
// Injected as `crate::game::farmersdelight` (a module of `game.rs`) on the
// pinned commit.
//
// - Placement: each block's `getStateForPlacement`.
// - The cutting board: `CuttingBoardBlock.useItemOn` and its `ToolCarvingEvent`,
//   `CuttingBoardBlockEntity` (kept as the chunk's block entity in the mod's own
//   format: `Inventory` and `IsItemCarved`) and `CuttingBoardRenderer`, drawn
//   with client item displays posed exactly as the renderer poses its models.
use super::text_displays::{RemoteTextDisplay, Shows};
use super::{Game, Hand, spill};
use horizon_assets::item_model::DisplayContext;
use horizon_client::items::ItemStack;
use horizon_client::mobs::Who;
use horizon_client::particles::{ParticleOptions, ParticleType};
use horizon_client::save;
use horizon_client::text::Text;
use horizon_core::random::RandomSource;
use horizon_protocol::entity_data::Value;
use horizon_world::nbt::Tag;
use horizon_world::registry::BlockStateId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{LazyLock, Mutex};

const BOARD: &str = "farmersdelight:cutting_board";
/// `ItemStackHandler`'s slot limit.
const SLOT_LIMIT: u32 = 64;
/// `Configuration.CUTTING_BOARD_FORTUNE_BONUS`'s default.
const FORTUNE_BONUS: f32 = 0.1;
/// `ItemDisplayContext.FIXED`, the context the renderer draws with.
const FIXED_CONTEXT: i8 = 8;

// ------------------------------------------------------------ placement

/// What `BlockPlaceContext` reads of a click, noted before the block is
/// placed (the placement replaces the water it may stand in).
struct Click {
    tick: u64,
    face: [i32; 3],
    water: Vec<([i32; 3], bool)>,
    sneaking: bool,
}

/// A direction's unit step.
fn dir_step(d: &str) -> [i32; 3] {
    match d {
        "north" => [0, 0, -1],
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        "up" => [0, 1, 0],
        _ => [0, -1, 0],
    }
}
fn add3(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
static CLICK: LazyLock<Mutex<Option<Click>>> = LazyLock::new(|| Mutex::new(None));

const HORIZONTAL_2D: [&str; 4] = ["south", "west", "north", "east"];

fn opposite(d: &str) -> &'static str {
    match d {
        "north" => "south",
        "south" => "north",
        "east" => "west",
        "west" => "east",
        "up" => "down",
        _ => "up",
    }
}
fn face_name(face: [i32; 3]) -> &'static str {
    match face {
        [0, 1, 0] => "up",
        [0, -1, 0] => "down",
        [0, 0, -1] => "north",
        [0, 0, 1] => "south",
        [-1, 0, 0] => "west",
        _ => "east",
    }
}
/// `Direction.fromYRot`.
fn horizontal_direction(yaw: f32) -> &'static str {
    HORIZONTAL_2D[((f64::from(yaw) / 90.0 + 0.5).floor() as i64 & 3) as usize]
}
/// `Direction.orderedByNearest(entity)[0]`.
fn nearest_looking_direction(yaw: f32, pitch: f32) -> &'static str {
    let f = pitch.to_radians();
    let g = -yaw.to_radians();
    let (h, i, j, k) = (f.sin(), f.cos(), g.sin(), g.cos());
    let (east, up, south) = (j > 0.0, h < 0.0, k > 0.0);
    let l = if east { j } else { -j };
    let m = if up { -h } else { h };
    let n = if south { k } else { -k };
    let o = l * i;
    let p = n * i;
    let d1 = if east { "east" } else { "west" };
    let d2 = if up { "up" } else { "down" };
    let d3 = if south { "south" } else { "north" };
    if l > n {
        if m > o { d2 } else { d1 }
    } else if m > p {
        d2
    } else {
        d3
    }
}

// ------------------------------------------------------ pies and feasts

/// `PieBlock.pieSlice` of each pie (`ModBlocks`).
fn pie_slice(name: &str) -> Option<&'static str> {
    Some(match name {
        "farmersdelight:apple_pie" => "farmersdelight:apple_pie_slice",
        "farmersdelight:sweet_berry_cheesecake" => "farmersdelight:sweet_berry_cheesecake_slice",
        "farmersdelight:chocolate_pie" => "farmersdelight:chocolate_pie_slice",
        "farmersdelight:pumpkin_pie" => "farmersdelight:pumpkin_pie_slice",
        _ => return None,
    })
}
/// `PieBlock.getMaxBites`.
const PIE_BITES: i32 = 4;

/// A `FeastBlock` as `ModBlocks` makes it.
struct Feast {
    /// `servingItem`; the rice roll medley serves by its servings left.
    serving: &'static str,
    /// The serving's `craftRemainder` (`bowlFoodItem`): what the player
    /// must hold to take one.
    container: Option<&'static str>,
    leftovers: bool,
    particles: bool,
}
fn feast(name: &str) -> Option<Feast> {
    let bowl = Some("minecraft:bowl");
    let (serving, container, leftovers, particles) = match name {
        "farmersdelight:roast_chicken_block" => ("farmersdelight:roast_chicken", bowl, true, true),
        "farmersdelight:stuffed_pumpkin_block" => ("farmersdelight:stuffed_pumpkin", bowl, false, true),
        "farmersdelight:honey_glazed_ham_block" => ("farmersdelight:honey_glazed_ham", bowl, true, true),
        "farmersdelight:shepherds_pie_block" => ("farmersdelight:shepherds_pie", bowl, true, true),
        "farmersdelight:gleaming_salad_block" => ("farmersdelight:gleaming_salad", bowl, true, true),
        "farmersdelight:rice_roll_medley_block" => ("farmersdelight:salmon_roll", None, true, false),
        _ => return None,
    };
    Some(Feast {
        serving,
        container,
        leftovers,
        particles,
    })
}
/// `RiceRollMedleyBlock.riceRollServings`, by servings left minus one.
const RICE_ROLLS: [&str; 8] = [
    "farmersdelight:cod_roll",
    "farmersdelight:cod_roll",
    "farmersdelight:salmon_roll",
    "farmersdelight:salmon_roll",
    "farmersdelight:salmon_roll",
    "farmersdelight:kelp_roll_slice",
    "farmersdelight:kelp_roll_slice",
    "farmersdelight:kelp_roll_slice",
];
/// `Direction.getStepX/Z` of a horizontal direction.
fn step(d: &str) -> [f64; 2] {
    match d {
        "north" => [0.0, -1.0],
        "south" => [0.0, 1.0],
        "west" => [-1.0, 0.0],
        _ => [1.0, 0.0],
    }
}

// ---------------------------------------------------------------- stove

const STOVE: &str = "farmersdelight:stove";
/// `StoveBlockEntity.getInventorySlotCount`.
const STOVE_SLOTS: usize = 6;
/// `StoveBlockEntity.getStoveItemOffset`.
const STOVE_OFFSETS: [[f32; 2]; STOVE_SLOTS] = [
    [0.3, 0.2],
    [0.0, 0.2],
    [-0.3, 0.2],
    [0.3, -0.2],
    [0.0, -0.2],
    [-0.3, -0.2],
];
/// `RecipeType.CAMPFIRE_COOKING`, what the stove cooks.
const CAMPFIRE_COOKING: &str = "minecraft:campfire_cooking";

/// What a stove holds (`AbstractStoveBlockEntity`): one item per slot and
/// each slot's cooking progress and total time.
#[derive(Clone, Default, PartialEq)]
struct Stove {
    items: [Option<ItemStack>; STOVE_SLOTS],
    progress: [i32; STOVE_SLOTS],
    total: [i32; STOVE_SLOTS],
}
impl Stove {
    fn is_empty(&self) -> bool {
        self.items.iter().all(Option::is_none)
    }
}
fn int_array(tag: &Tag, key: &str) -> [i32; STOVE_SLOTS] {
    let mut out = [0; STOVE_SLOTS];
    if let Some(Tag::IntArray(v)) = tag.get(key) {
        for (o, x) in out.iter_mut().zip(v) {
            *o = *x;
        }
    }
    out
}
/// The stove's items in a saved tag (`Inventory`, an `ItemStackHandler`).
pub(super) fn stove_items(tag: &Tag) -> [Option<ItemStack>; STOVE_SLOTS] {
    let mut items: [Option<ItemStack>; STOVE_SLOTS] = Default::default();
    let inventory = tag.get("Inventory").unwrap_or(tag);
    if let Some(Tag::List(list)) = inventory.get("Items") {
        for t in list {
            if let Some(Tag::Int(slot)) = t.get("Slot")
                && let Some(cell) = items.get_mut(*slot as usize)
            {
                *cell = save::item_from_nbt(t);
            }
        }
    }
    items
}
fn stove_from_tag(tag: &Tag) -> Stove {
    Stove {
        items: stove_items(tag),
        progress: int_array(tag, "CookingTimes"),
        total: int_array(tag, "CookingTotalTimes"),
    }
}
/// `AbstractStoveBlockEntity.saveAdditional`.
fn stove_tag(pos: [i32; 3], stove: &Stove) -> Tag {
    let items: Vec<Tag> = stove
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let mut item = save::item_to_nbt(s.as_ref()?);
            if let Tag::Compound(map) = &mut item {
                map.insert("Slot".into(), Tag::Int(i as i32));
            }
            Some(item)
        })
        .collect();
    Tag::compound([
        ("id", Tag::String(STOVE.into())),
        ("x", Tag::Int(pos[0])),
        ("y", Tag::Int(pos[1])),
        ("z", Tag::Int(pos[2])),
        (
            "Inventory",
            Tag::compound([
                ("Items", Tag::List(items)),
                ("Size", Tag::Int(STOVE_SLOTS as i32)),
            ]),
        ),
        ("CookingTimes", Tag::IntArray(stove.progress.to_vec())),
        ("CookingTotalTimes", Tag::IntArray(stove.total.to_vec())),
    ])
}
/// `v` turned by the quaternion `q` (x, y, z, w).
fn rotate_vec(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let p = [v[0], v[1], v[2], 0.0];
    let conj = [-q[0], -q[1], -q[2], q[3]];
    let r = mul(mul(q, p), conj);
    [r[0], r[1], r[2]]
}
/// `Direction.getClockWise` of a horizontal direction.
fn clockwise(d: &str) -> &'static str {
    match d {
        "north" => "east",
        "east" => "south",
        "south" => "west",
        _ => "north",
    }
}
/// The stoves each side knows, as the boards are published (`VIEWS`).
static STOVE_VIEWS: LazyLock<Mutex<HashMap<bool, HashMap<Key, [Option<ItemStack>; STOVE_SLOTS]>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
/// The authoritative side's stoves holding something: they tick.
static ACTIVE_STOVES: LazyLock<Mutex<std::collections::HashSet<Key>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

// ---------------------------------------------------------- cooking pot

const POT: &str = "farmersdelight:cooking_pot";
/// `CookingPotBlockEntity`'s slots: six ingredients, then these.
const MEAL_SLOT: usize = 6;
const CONTAINER_SLOT: usize = 7;
const OUTPUT_SLOT: usize = 8;
const POT_SLOTS: usize = 9;

/// The `craftRemainder` of Farmer's Delight's items (`ModItems`:
/// `bowlFoodItem`, `drinkItem`, and the two set by hand).
fn fd_remainder(item: &str) -> Option<&'static str> {
    let path = item.strip_prefix("farmersdelight:")?;
    Some(match path {
        "milk_bottle" | "hot_cocoa" | "apple_cider" | "melon_juice" | "glow_berry_custard" => {
            "minecraft:glass_bottle"
        }
        "fruit_salad" | "mixed_salad" | "nether_salad" | "cooked_rice" | "bone_broth"
        | "beef_stew" | "chicken_soup" | "vegetable_soup" | "fish_stew" | "fried_rice"
        | "pumpkin_soup" | "baked_cod_stew" | "noodle_soup" | "onion_soup" | "bacon_and_eggs"
        | "pasta_with_meatballs" | "pasta_with_mutton_chop" | "mushroom_rice"
        | "roasted_mutton_chops" | "vegetable_noodles" | "steak_and_potatoes" | "ratatouille"
        | "squid_ink_pasta" | "grilled_salmon" | "roast_chicken" | "stuffed_pumpkin"
        | "honey_glazed_ham" | "shepherds_pie" | "gleaming_salad" | "dog_food"
        | "tomato_sauce" => "minecraft:bowl",
        _ => return None,
    })
}
/// `CookingPotBlockEntity.INGREDIENT_REMAINDER_OVERRIDES`.
fn remainder_override(item: &str) -> Option<&'static str> {
    Some(match item {
        "minecraft:powder_snow_bucket"
        | "minecraft:axolotl_bucket"
        | "minecraft:cod_bucket"
        | "minecraft:pufferfish_bucket"
        | "minecraft:salmon_bucket"
        | "minecraft:tropical_fish_bucket" => "minecraft:bucket",
        "minecraft:suspicious_stew"
        | "minecraft:mushroom_stew"
        | "minecraft:rabbit_stew"
        | "minecraft:beetroot_soup" => "minecraft:bowl",
        "minecraft:potion"
        | "minecraft:splash_potion"
        | "minecraft:lingering_potion"
        | "minecraft:experience_bottle" => "minecraft:glass_bottle",
        _ => return None,
    })
}

/// `CookingPotRecipe`.
#[derive(Clone)]
struct PotRecipe {
    id: String,
    inputs: Vec<Ingredient>,
    result: String,
    count: u32,
    /// `getOutputContainer`: the recipe's container, else the result's
    /// crafting remainder.
    container: Option<String>,
    experience: f32,
    time: i32,
}
fn load_pot_recipes() -> Vec<PotRecipe> {
    let dir = Game::options_path()
        .with_file_name("horizon-mods")
        .join("farmersdelight")
        .join("data/farmersdelight/recipe/cooking");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    files.sort();
    let mut recipes = Vec::new();
    for path in files {
        let Some(v) = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        else {
            continue;
        };
        if v["type"] != "farmersdelight:cooking" {
            continue;
        }
        let inputs: Vec<Ingredient> = v["ingredients"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|i| {
                let mut ing = Ingredient::default();
                read_ingredient(i, &mut ing);
                ing
            })
            .collect();
        let Some(result) = v["result"]["id"].as_str().map(String::from) else {
            continue;
        };
        let container = v["container"]["id"]
            .as_str()
            .or_else(|| v["container"]["item"].as_str())
            .map(String::from)
            .or_else(|| fd_remainder(&result).map(String::from));
        let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        recipes.push(PotRecipe {
            id: format!("farmersdelight:cooking/{stem}"),
            inputs,
            count: v["result"]["count"].as_u64().unwrap_or(1) as u32,
            result,
            container,
            experience: v["experience"].as_f64().unwrap_or(0.0) as f32,
            time: v["cookingtime"].as_i64().unwrap_or(200) as i32,
        });
    }
    recipes
}
static POT_RECIPES: LazyLock<Vec<PotRecipe>> = LazyLock::new(load_pot_recipes);

/// What a cooking pot holds (`CookingPotBlockEntity`).
#[derive(Clone, Default)]
struct Pot {
    items: [Option<ItemStack>; POT_SLOTS],
    cook_time: i32,
    cook_time_total: i32,
    /// `mealContainerStack`.
    container: Option<ItemStack>,
    custom_name: Option<String>,
    /// `usedRecipeTracker`.
    used: Vec<(String, i32)>,
}
fn pot_items(tag: &Tag) -> [Option<ItemStack>; POT_SLOTS] {
    let mut items: [Option<ItemStack>; POT_SLOTS] = Default::default();
    if let Some(Tag::List(list)) = tag.get("Inventory").and_then(|i| i.get("Items")) {
        for t in list {
            if let Some(Tag::Int(slot)) = t.get("Slot")
                && let Some(cell) = items.get_mut(*slot as usize)
            {
                *cell = save::item_from_nbt(t);
            }
        }
    }
    items
}
fn pot_from_tag(tag: &Tag) -> Pot {
    let int = |k: &str| match tag.get(k) {
        Some(Tag::Int(v)) => *v,
        _ => 0,
    };
    let mut used = Vec::new();
    if let Some(Tag::Compound(map)) = tag.get("RecipesUsed") {
        for (k, v) in map.iter() {
            if let Tag::Int(n) = v {
                used.push((k.to_string_lossy().to_string(), *n));
            }
        }
    }
    Pot {
        items: pot_items(tag),
        cook_time: int("CookTime"),
        cook_time_total: int("CookTimeTotal"),
        container: tag.get("Container").and_then(save::item_from_nbt),
        custom_name: match tag.get("CustomName") {
            Some(Tag::String(s)) => Some(s.to_string_lossy().to_string()),
            _ => None,
        },
        used,
    }
}
/// `CookingPotBlockEntity.saveAdditional`.
fn pot_tag(pos: [i32; 3], pot: &Pot) -> Tag {
    let items: Vec<Tag> = pot
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let mut item = save::item_to_nbt(s.as_ref()?);
            if let Tag::Compound(map) = &mut item {
                map.insert("Slot".into(), Tag::Int(i as i32));
            }
            Some(item)
        })
        .collect();
    let used = Tag::compound(pot.used.iter().map(|(k, v)| (k.as_str(), Tag::Int(*v))));
    let mut tag = Tag::compound([
        ("id", Tag::String(POT.into())),
        ("x", Tag::Int(pos[0])),
        ("y", Tag::Int(pos[1])),
        ("z", Tag::Int(pos[2])),
        ("CookTime", Tag::Int(pot.cook_time)),
        ("CookTimeTotal", Tag::Int(pot.cook_time_total)),
        (
            "Container",
            pot.container
                .as_ref()
                .map_or_else(|| Tag::compound([] as [(&str, Tag); 0]), save::item_to_nbt),
        ),
        (
            "Inventory",
            Tag::compound([
                ("Items", Tag::List(items)),
                ("Size", Tag::Int(POT_SLOTS as i32)),
            ]),
        ),
        ("RecipesUsed", used),
    ]);
    if let (Some(name), Tag::Compound(map)) = (&pot.custom_name, &mut tag) {
        map.insert("CustomName".into(), Tag::String(name.as_str().into()));
    }
    tag
}
/// The items a removed pot drops (`getDroppableInventory`: all but the
/// meal) and the experience of the recipes it cooked.
pub(super) fn pot_drops(tag: &Tag) -> (Vec<ItemStack>, u32) {
    let pot = pot_from_tag(tag);
    let drops = pot
        .items
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != MEAL_SLOT)
        .filter_map(|(_, s)| s.clone())
        .collect();
    (drops, pot_xp(&pot.used))
}
/// `getUsedRecipesAndPopExperience`'s experience: per recipe, floor(count *
/// experience), and the fraction's chance of one more (`createExperience`).
fn pot_xp(used: &[(String, i32)]) -> u32 {
    let mut xp = 0u32;
    for (id, count) in used {
        if let Some(r) = POT_RECIPES.iter().find(|r| r.id == *id) {
            let total = *count as f32 * r.experience;
            let mut whole = total.floor() as u32;
            let fraction = total - total.floor();
            if fraction != 0.0 && rand_f64() < f64::from(fraction) {
                whole += 1;
            }
            xp += whole;
        }
    }
    xp
}
/// `Math.random()`.
fn rand_f64() -> f64 {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64),
    );
    (h.finish() >> 11) as f64 / (1u64 << 53) as f64
}
/// The pots whose block entity the authoritative side ticks.
static ACTIVE_POTS: LazyLock<Mutex<std::collections::HashSet<Key>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));
/// Each side's pots with a meal in them (for the boiling sound's choice).
static POT_MEALS: LazyLock<Mutex<HashMap<bool, std::collections::HashSet<Key>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// ------------------------------------------------------- cutting board

/// What a cutting board holds (`CuttingBoardBlockEntity`).
#[derive(Clone, Default)]
struct Board {
    stack: Option<ItemStack>,
    carved: bool,
}

/// The stored stack of a cutting board's saved tag (`Inventory`, an
/// `ItemStackHandler` of one slot).
pub(super) fn board_stack(tag: &Tag) -> Option<ItemStack> {
    let Some(Tag::List(items)) = tag.get("Inventory").and_then(|i| i.get("Items")) else {
        return None;
    };
    items
        .iter()
        .find(|t| matches!(t.get("Slot"), Some(Tag::Int(0))))
        .and_then(save::item_from_nbt)
}
fn board_from_tag(tag: &Tag) -> Board {
    Board {
        stack: board_stack(tag),
        carved: matches!(tag.get("IsItemCarved"), Some(Tag::Byte(b)) if *b != 0),
    }
}
/// `CuttingBoardBlockEntity.saveAdditional`.
fn board_tag(pos: [i32; 3], board: &Board) -> Tag {
    let mut items = Vec::new();
    if let Some(stack) = &board.stack {
        let mut item = save::item_to_nbt(stack);
        if let Tag::Compound(map) = &mut item {
            map.insert("Slot".into(), Tag::Int(0));
        }
        items.push(item);
    }
    Tag::compound([
        ("id", Tag::String(BOARD.into())),
        ("x", Tag::Int(pos[0])),
        ("y", Tag::Int(pos[1])),
        ("z", Tag::Int(pos[2])),
        (
            "Inventory",
            Tag::compound([("Items", Tag::List(items)), ("Size", Tag::Int(1))]),
        ),
        ("IsItemCarved", Tag::Byte(board.carved as i8)),
    ])
}

/// One possible result of a cutting recipe (`ChanceResult`).
#[derive(Clone)]
struct Chance {
    item: String,
    count: u32,
    chance: f32,
}
/// An ingredient: any of these items or tags (`Ingredient`; Farmer's
/// Delight's `item_ability` ingredient is the tools' tag here).
#[derive(Clone, Default)]
struct Ingredient {
    items: Vec<String>,
    tags: Vec<String>,
}
/// `CuttingBoardRecipe`.
#[derive(Clone)]
struct CuttingRecipe {
    input: Ingredient,
    tool: Ingredient,
    results: Vec<Chance>,
    sound: Option<String>,
}

/// `ItemAbility`s as the tools' tags hold them (`canPerformAction`).
fn ability_tags(action: &str) -> &'static [&'static str] {
    match action.split('_').next().unwrap_or("") {
        "knife" => &["c:tools/knife", "farmersdelight:tools/knives"],
        "axe" => &["minecraft:axes"],
        "shovel" => &["minecraft:shovels"],
        "pickaxe" => &["minecraft:pickaxes"],
        "hoe" => &["minecraft:hoes"],
        "sword" => &["minecraft:swords"],
        "shears" => &["c:tools/shear"],
        _ => &[],
    }
}
fn read_ingredient(v: &serde_json::Value, out: &mut Ingredient) {
    match v {
        serde_json::Value::Array(list) => list.iter().for_each(|e| read_ingredient(e, out)),
        serde_json::Value::Object(o) => {
            if let Some(item) = o.get("item").and_then(|i| i.as_str()) {
                out.items.push(item.to_owned());
            }
            if let Some(tag) = o.get("tag").and_then(|t| t.as_str()) {
                out.tags.push(tag.to_owned());
            }
            if o.get("type").and_then(|t| t.as_str()) == Some("farmersdelight:item_ability")
                && let Some(action) = o.get("action").and_then(|a| a.as_str())
            {
                out.tags.extend(ability_tags(action).iter().map(|t| t.to_string()));
            }
        }
        serde_json::Value::String(item) => out.items.push(item.clone()),
        _ => {}
    }
}
fn load_recipes() -> Vec<CuttingRecipe> {
    let dir = Game::options_path()
        .with_file_name("horizon-mods")
        .join("farmersdelight")
        .join("data/farmersdelight/recipe/cutting");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    // The recipe manager's order: by id.
    files.sort();
    let mut recipes = Vec::new();
    for path in files {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            continue;
        };
        if v["type"] != "farmersdelight:cutting" {
            continue;
        }
        let mut input = Ingredient::default();
        read_ingredient(&v["ingredients"], &mut input);
        let mut tool = Ingredient::default();
        read_ingredient(&v["tool"], &mut tool);
        let results: Vec<Chance> = v["result"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|r| {
                Some(Chance {
                    item: r["item"]["id"].as_str()?.to_owned(),
                    count: r["item"]["count"].as_u64().unwrap_or(1) as u32,
                    chance: r["chance"].as_f64().unwrap_or(1.0) as f32,
                })
            })
            .collect();
        let sound = v["sound"]["sound_id"].as_str().map(String::from);
        recipes.push(CuttingRecipe {
            input,
            tool,
            results,
            sound,
        });
    }
    recipes
}
static RECIPES: LazyLock<Vec<CuttingRecipe>> = LazyLock::new(load_recipes);

/// Java's `java.util.Random`, for the renderer's model offsets.
struct JavaRandom(i64);
impl JavaRandom {
    fn new(seed: i64) -> Self {
        JavaRandom((seed ^ 0x5DEE_CE66D) & ((1 << 48) - 1))
    }
    fn next_float(&mut self) -> f32 {
        self.0 = (self.0.wrapping_mul(0x5DEE_CE66D).wrapping_add(0xB)) & ((1 << 48) - 1);
        (self.0 >> 24) as i32 as f32 / (1 << 24) as f32
    }
}

/// A quaternion (x, y, z, w) of `degrees` about an axis.
fn axis_rotation(axis: [f32; 3], degrees: f32) -> [f32; 4] {
    let (s, c) = (degrees.to_radians() / 2.0).sin_cos();
    [axis[0] * s, axis[1] * s, axis[2] * s, c]
}
fn mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}
/// `Direction.toYRot`.
fn y_rot(d: &str) -> f32 {
    match d {
        "south" => 0.0,
        "west" => 90.0,
        "north" => 180.0,
        _ => 270.0,
    }
}
fn counter_clockwise(d: &str) -> [f64; 2] {
    match d {
        "north" => [-1.0, 0.0],
        "west" => [0.0, 1.0],
        "south" => [1.0, 0.0],
        _ => [0.0, -1.0],
    }
}

/// A cutting board as the window draws it.
#[derive(Clone, PartialEq)]
struct View {
    stack: ItemStack,
    carved: bool,
    facing: String,
}
type Key = (i32, i32, i32, &'static str);
/// What the window draws on a block: a board's contents, or a stove's
/// items with the stove's facing.
enum Want {
    Board(View),
    Stove([Option<ItemStack>; STOVE_SLOTS], String),
    Skillet(ItemStack, String),
}
impl Want {
    /// What the models depend on: the same signature, the same models.
    fn signature(&self) -> String {
        match self {
            Want::Board(v) => format!(
                "board|{}|{}|{}|{}|{:?}",
                v.stack.item, v.stack.count, v.stack.damage, v.carved, v.facing
            ),
            Want::Stove(items, facing) => {
                let mut s = format!("stove|{facing}");
                for item in items {
                    match item {
                        Some(i) => s.push_str(&format!("|{}:{}", i.item, i.damage)),
                        None => s.push_str("|-"),
                    }
                }
                s
            }
            Want::Skillet(stack, facing) => format!(
                "skillet|{facing}|{}:{}:{}",
                stack.item, stack.damage, stack.count
            ),
        }
    }
}
/// The boards each side knows, published by its `Game` (false: the
/// server's, true: the window's own world), by dimension and position.
static VIEWS: LazyLock<Mutex<HashMap<bool, HashMap<Key, View>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
/// The window's item displays per board: their entity ids and what they show.
static DISPLAYS: LazyLock<Mutex<HashMap<Key, (Vec<i32>, String)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_DISPLAY: AtomicI32 = AtomicI32::new(-2_000_000);

impl Game<'_> {
    fn fd_props(&self, state: BlockStateId) -> HashMap<String, String> {
        self.pack
            .assets
            .registry
            .state(state)
            .map(|s| {
                s.properties
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
    fn fd_name(&self, pos: [i32; 3]) -> String {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        self.res.facts().get(state).name.to_string()
    }
    /// A water source, or a block waterlogged with one (`Fluids.WATER`).
    fn fd_water_source(&self, pos: [i32; 3]) -> bool {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let Some(s) = self.pack.assets.registry.state(state) else {
            return false;
        };
        (&*s.name == "minecraft:water" && s.properties.get("level").is_some_and(|l| &**l == "0"))
            || s.properties.get("waterlogged").is_some_and(|w| &**w == "true")
    }
    fn fd_item_in_tag(&self, tag: &str, item: &str) -> bool {
        self.res
            .items()
            .item_tags
            .get(tag)
            .is_some_and(|t| t.contains(item))
    }
    fn fd_ingredient_test(&self, ingredient: &Ingredient, stack: &ItemStack) -> bool {
        ingredient.items.iter().any(|i| *i == *stack.item)
            || ingredient
                .tags
                .iter()
                .any(|t| self.fd_item_in_tag(t, &stack.item))
    }
    fn fd_main(&self) -> Option<ItemStack> {
        self.inventory.get(self.selected).and_then(Clone::clone)
    }
    fn fd_set_main(&mut self, stack: Option<ItemStack>) {
        if let Some(slot) = self.inventory.get_mut(self.selected) {
            *slot = stack.filter(|s| s.count > 0);
        }
    }
    fn fd_max_stack(&self, stack: &ItemStack) -> u32 {
        self.res.items().get(&stack.item).map_or(64, |i| i.max_stack)
    }
    /// `Player.displayClientMessage(text, true)`: the acting player's action
    /// bar. Single player runs on an integrated server, whose players are
    /// its guests: the acting guest gets the packet at the end of its turn,
    /// as the game's own action-bar lines do (`sign_click_actions_disabled`).
    fn fd_action_bar(&mut self, text: Text) {
        if self.lan_acting.is_some() {
            self.guest_packets
                .push(horizon_protocol::packets::play::Clientbound::SystemChat(
                    horizon_protocol::packets::play::SystemChat {
                        content: horizon_client::lan::data::component_nbt(&text.component()),
                        overlay: true,
                    },
                ));
            return;
        }
        if self.options.dedicated.is_none() {
            self.titles.set_overlay(text, false);
        }
    }
    fn fd_play(&mut self, pos: [i32; 3], sound: &str, volume: f32, pitch: f32) {
        let center = pos.map(|v| f64::from(v) + 0.5);
        self.audio.play(sound, Some(center), volume, pitch);
    }

    /// `Game::use_hand`'s HEAD: Some when a Farmer's Delight block took the
    /// hand's turn (true: the click ends there).
    pub(super) fn fd_use_hand(
        &mut self,
        hit: Option<([i32; 3], [i32; 3], [f64; 3])>,
        sneaking: bool,
    ) -> Option<bool> {
        if self.player.is_none() || self.spectator() || !self.native_on("farmersdelight") {
            return None;
        }
        let (pos, face, _point) = hit?;
        // BlockPlaceContext of the block item this click may place.
        let target = [pos[0] + face[0], pos[1] + face[1], pos[2] + face[2]];
        let water = vec![
            (pos, self.fd_water_source(pos)),
            (target, self.fd_water_source(target)),
        ];
        if let Ok(mut click) = CLICK.lock() {
            *click = Some(Click {
                tick: self.ticks,
                face,
                water,
                sneaking,
            });
        }
        let name = self.fd_name(pos);
        // ServerPlayerGameMode.useItemOn: isSecondaryUseActive with something
        // in either hand keeps the block from being used.
        let hands_full = self.fd_main().is_some() || self.offhand.is_some();
        if let Some(slice) = pie_slice(&name) {
            if sneaking && hands_full {
                return None;
            }
            return self.fd_pie_use(pos, slice);
        }
        if let Some(feast) = feast(&name) {
            if sneaking && hands_full {
                return None;
            }
            return self.fd_feast_use(pos, &feast);
        }
        if name == STOVE {
            if sneaking && hands_full {
                return None;
            }
            return self.fd_stove_use(pos);
        }
        if name == POT {
            if sneaking && hands_full {
                return None;
            }
            return self.fd_pot_use(pos, sneaking);
        }
        if name == SKILLET {
            if sneaking && hands_full {
                return None;
            }
            return self.fd_skillet_use(pos);
        }
        if is_cabinet(&name) || is_basket(&name) {
            if sneaking && hands_full {
                return None;
            }
            return self.fd_container_use(pos, is_cabinet(&name));
        }
        if name == TOMATOES {
            if sneaking && hands_full {
                return None;
            }
            return self.fd_tomato_pick(pos);
        }
        if name != BOARD {
            return None;
        }
        let main = self.fd_main();
        // ToolCarvingEvent: a RightClickBlock event, before useItemOn, for
        // each hand, always with the main hand's stack.
        if sneaking && let Some(tool) = &main {
            if self.fd_board_carve(pos, tool) {
                self.swing_arm();
                return Some(true);
            }
        }
        // isSecondaryUseActive with something in either hand: the block is
        // not used.
        if sneaking && (main.is_some() || self.offhand.is_some()) {
            return None;
        }
        // useItemOn reads the main hand on the main hand's turn and never
        // passes, so the off hand never gets one.
        if self.hand != Hand::Main {
            return None;
        }
        let swing = self.fd_board_use(pos, main);
        if swing {
            self.swing_arm();
        }
        Some(true)
    }

    fn fd_int_prop(&self, state: BlockStateId, prop: &str) -> i32 {
        self.fd_props(state)
            .get(prop)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }
    /// `serverLevel.sendParticles(new BlockParticleOption(BLOCK, state), x +
    /// 0.5, y + `dy`, z + 0.5, 3, 0.1, 0.1, 0.1, 0.001)`.
    fn fd_block_particles(&mut self, pos: [i32; 3], state: BlockStateId, dy: f64) {
        let Some((sprite, tint)) = self.particle_sprite(state, pos) else {
            return;
        };
        let options = ParticleOptions::block(ParticleType::Block, sprite, tint);
        self.particles.send_particles(
            &options,
            [
                f64::from(pos[0]) + 0.5,
                f64::from(pos[1]) + dy,
                f64::from(pos[2]) + 0.5,
            ],
            [0.1, 0.1, 0.1],
            0.001,
            3,
        );
    }
    fn fd_is_knife(&self, item: &str) -> bool {
        // ItemUtils.isKnife: CommonTags.Items.TOOLS_KNIFE.
        self.fd_item_in_tag("c:tools/knife", item)
    }
    /// `Player.canEat(false)`.
    fn fd_can_eat(&self) -> bool {
        self.creative || self.player.as_ref().is_some_and(|p| p.health.food.food < 20)
    }

    /// A pie's turn of a hand (`PieBlock.useItemOn`, then `useWithoutItem` on
    /// the main hand's): true when it took the click.
    fn fd_pie_use(&mut self, pos: [i32; 3], slice: &str) -> Option<bool> {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let held = self.held().cloned();
        // useItemOn: a knife in this hand cuts a slice off.
        if let Some(knife) = held.filter(|s| self.fd_is_knife(&s.item)) {
            self.fd_pie_cut(pos, state, slice, &knife);
            self.swing_arm();
            return Some(true);
        }
        // PASS_TO_DEFAULT_BLOCK_INTERACTION: the main hand's turn goes on to
        // useWithoutItem, which eats a bite.
        if self.hand != Hand::Main || !self.fd_can_eat() {
            return None;
        }
        self.fd_pie_bite(pos, state, slice);
        self.swing_arm();
        Some(true)
    }
    /// The pie loses a bite, or goes when it was the last.
    fn fd_pie_take(&mut self, pos: [i32; 3], state: BlockStateId) {
        let bites = self.fd_int_prop(state, "bites");
        let next = if bites < PIE_BITES - 1 {
            let next = (bites + 1).to_string();
            self.with_props(state, &[("bites", next.as_str())])
        } else {
            Some(BlockStateId(0))
        };
        if let Some(next) = next {
            let _ = self.level_set(pos, next);
        }
    }
    /// `PieBlock.consumeBite`: the slice's food and effects, a bite less.
    fn fd_pie_bite(&mut self, pos: [i32; 3], state: BlockStateId, slice: &str) {
        if let Some(info) = self.res.items().get(slice).cloned() {
            if let Some((nutrition, saturation)) = info.food
                && let Some(p) = self.player.as_mut()
            {
                // FoodData.eat(sliceFood).
                let food = &mut p.health.food;
                food.food = (food.food + nutrition as i32).clamp(0, 20);
                food.saturation = (food.saturation + saturation).clamp(0.0, food.food as f32);
            }
            if let Some(consumable) = &info.consumable {
                for effect in &consumable.effects {
                    self.apply_consume_effect(effect);
                }
            }
        }
        self.fd_pie_take(pos, state);
        self.fd_play(pos, "minecraft:entity.generic.eat", 0.8, 0.8);
        self.fd_block_particles(pos, state, 0.3);
    }
    /// `PieBlock.cutSlice`: a bite less, and its slice thrown toward the
    /// player.
    fn fd_pie_cut(&mut self, pos: [i32; 3], state: BlockStateId, slice: &str, knife: &ItemStack) {
        self.fd_pie_take(pos, state);
        // player.getDirection().getOpposite().
        let [sx, sz] = step(opposite(horizontal_direction(self.yaw)));
        self.spawn_item(
            ItemStack::new(slice, 1),
            [
                f64::from(pos[0]) + 0.5,
                f64::from(pos[1]) + 0.3,
                f64::from(pos[2]) + 0.5,
            ],
            [sx * 0.15, 0.05, sz * 0.15],
            0,
        );
        self.fd_play(pos, "farmersdelight:block.food.slice", 0.8, 0.8);
        self.fd_block_particles(pos, state, 0.3);
        self.stats.add("used", &knife.item, 1);
    }

    /// `FeastBlock.useItemOn` (`takeServing`) with this hand's stack: true
    /// when it took the click.
    fn fd_feast_use(&mut self, pos: [i32; 3], feast: &Feast) -> Option<bool> {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let servings = self.fd_int_prop(state, "servings");
        if servings == 0 {
            // Leftovers: the plate breaks off with its drops.
            self.fd_play(pos, "minecraft:block.wood.break", 0.8, 0.8);
            let _ = self.destroy_block_dropping(pos);
            self.swing_arm();
            return Some(true);
        }
        let serving = if feast.serving == "farmersdelight:salmon_roll" && feast.container.is_none() {
            RICE_ROLLS[(servings - 1).clamp(0, 7) as usize]
        } else {
            feast.serving
        };
        let held = self.held().cloned();
        if let Some(container) = feast.container
            && !held.as_ref().is_some_and(|h| &*h.item == container)
        {
            // feast.use_container, with the container's name; the block
            // passes to its default interaction, which does nothing.
            let path = container.strip_prefix("minecraft:").unwrap_or(container);
            self.fd_action_bar(Text::translate(
                "block.farmersdelight.feast.use_container",
                &[Text::translate(&format!("item.minecraft.{path}"), &[])],
            ));
            return None;
        }
        let left = (servings - 1).to_string();
        if let Some(next) = self.with_props(state, &[("servings", left.as_str())]) {
            let _ = self.level_set(pos, next);
        }
        if let Some(h) = &held {
            self.stats.add("used", &h.item, 1);
        }
        // heldStack.shrink(1) (consume_held keeps a creative player's).
        if feast.container.is_some() {
            self.consume_held();
        }
        // Inventory.add, else Player.drop(serving, false).
        if let Some(rest) = self.give(ItemStack::new(serving, 1))
            && let Some(eye) = self.eye_position()
        {
            self.throw_from_eyes(rest, eye, false);
        }
        if servings - 1 == 0 && !feast.leftovers {
            let _ = self.destroy_block_dropping(pos);
        }
        self.fd_play(pos, "farmersdelight:block.food.take_portion", 1.0, 1.0);
        if feast.particles {
            self.fd_block_particles(pos, state, 0.5);
        }
        self.swing_arm();
        Some(true)
    }

    fn fd_stove(&self, pos: [i32; 3]) -> Stove {
        spill::saved_entity(&self.world, pos)
            .map(|t| stove_from_tag(&t))
            .unwrap_or_default()
    }
    /// The block entity's `setChanged` and `sendBlockUpdated`: saved with
    /// the chunk, sent to the players, drawn by the window.
    fn fd_put_stove(&mut self, pos: [i32; 3], stove: &Stove) {
        spill::put_saved_entity(&mut self.world, pos, stove_tag(pos, stove));
        self.mark_block_entity_updated((pos[0], pos[1], pos[2]));
        self.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
        self.fd_publish_stove(pos, &stove.items);
        if !stove.is_empty()
            && let Ok(mut active) = ACTIVE_STOVES.lock()
        {
            active.insert((pos[0], pos[1], pos[2], self.dim.id()));
        }
    }
    fn fd_publish_stove(&mut self, pos: [i32; 3], items: &[Option<ItemStack>; STOVE_SLOTS]) {
        let key = (pos[0], pos[1], pos[2], self.dim.id());
        let side = self.native_window();
        if let Ok(mut views) = STOVE_VIEWS.lock() {
            let map = views.entry(side).or_default();
            if items.iter().all(Option::is_none) {
                map.remove(&key);
            } else {
                map.insert(key, items.clone());
            }
        }
    }
    fn fd_lit(&self, pos: [i32; 3]) -> bool {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        self.fd_props(state).get("lit").is_some_and(|v| v == "true")
    }
    /// `AbstractStoveBlock.ignite` / `extinguish`: the lit state, and a
    /// block_change game event.
    fn fd_set_lit(&mut self, pos: [i32; 3], lit: bool) {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let value = if lit { "true" } else { "false" };
        if let Some(next) = self.with_props(state, &[("lit", value)]) {
            let _ = self.level_set(pos, next);
        }
        self.post_block_game_event("minecraft:block_change", pos, Some(Who::Player), None);
    }
    /// `AbstractStoveBlock.isStoveTopCovered`: the block above reaches into
    /// the grilling area, `Block.box(3, 0, 3, 13, 1, 13)` of the block above.
    fn fd_stove_covered(&self, pos: [i32; 3]) -> bool {
        let above = [pos[0], pos[1] + 1, pos[2]];
        let (x, y, z) = (f64::from(above[0]), f64::from(above[1]), f64::from(above[2]));
        let area = [
            x + 3.0 / 16.0,
            y,
            z + 3.0 / 16.0,
            x + 13.0 / 16.0,
            y + 1.0 / 16.0,
            z + 13.0 / 16.0,
        ];
        self.outline_boxes(above)
            .iter()
            .any(|b| (0..3).all(|a| b[a] < area[a + 3] && area[a] < b[a + 3]))
    }
    /// `Containers.dropItemStack` at the block's integer position.
    fn fd_drop_stack(&mut self, pos: [i32; 3], stack: ItemStack) {
        let at = horizon_core::position::BlockPos::new(pos[0], pos[1], pos[2]);
        for piece in horizon_world::level::removal::drop_item_stack(&mut self.world.random, at, stack.count)
        {
            self.spawn_item(
                ItemStack {
                    count: piece.count,
                    ..stack.clone()
                },
                piece.pos,
                piece.motion,
                0,
            );
        }
    }

    /// `AbstractStoveBlock.useItemOn` with this hand's stack: true when it
    /// took the click.
    fn fd_stove_use(&mut self, pos: [i32; 3]) -> Option<bool> {
        let held = self.held().cloned();
        if self.fd_lit(pos) {
            // tryToExtinguish (sidedSuccess: CONSUME on the server, so the
            // server swings no arm; the client's own prediction does).
            if let Some(h) = &held {
                if self.fd_item_in_tag("minecraft:shovels", &h.item) {
                    // LevelEvent.SOUND_EXTINGUISH_FIRE.
                    let r = &mut self.world.random;
                    let pitch = 2.6 + (r.next_float() - r.next_float()) * 0.8;
                    self.fd_play(pos, "minecraft:block.fire.extinguish", 0.5, pitch);
                    self.fd_set_lit(pos, false);
                    self.damage_held_tool();
                    return Some(true);
                }
                if self.fd_item_in_tag("c:buckets/water", &h.item) || &*h.item == "minecraft:water_bucket" {
                    self.fd_play(pos, "minecraft:entity.generic.extinguish_fire", 1.0, 1.0);
                    self.fd_set_lit(pos, false);
                    if !self.creative {
                        *self.held_mut() = Some(ItemStack::new("minecraft:bucket", 1));
                    }
                    return Some(true);
                }
            }
        } else if let Some(h) = &held {
            // tryToIgnite.
            if &*h.item == "minecraft:flint_and_steel" {
                let pitch = self.world.random.next_float() * 0.4 + 0.8;
                self.fd_play(pos, "minecraft:item.flintandsteel.use", 1.0, pitch);
                self.fd_set_lit(pos, true);
                self.damage_held_tool();
                self.swing_arm();
                return Some(true);
            }
            if &*h.item == "minecraft:fire_charge" {
                let r = &mut self.world.random;
                let pitch = (r.next_float() - r.next_float()) * 0.2 + 1.0;
                self.fd_play(pos, "minecraft:item.firecharge.use", 1.0, pitch);
                self.fd_set_lit(pos, true);
                self.consume_held();
                self.swing_arm();
                return Some(true);
            }
        }
        // tryToPlaceFoodItem.
        if self.fd_stove_covered(pos) {
            return None;
        }
        let held = held?;
        let time = self.recipes().cook(CAMPFIRE_COOKING, &held).map(|r| r.time as i32)?;
        let mut stove = self.fd_stove(pos);
        // placeFood: the next empty slot, or CONSUME when there is none.
        let Some(slot) = stove.items.iter().position(Option::is_none) else {
            return Some(true);
        };
        stove.items[slot] = Some(ItemStack {
            count: 1,
            ..held.clone()
        });
        stove.total[slot] = time;
        stove.progress[slot] = 0;
        self.fd_put_stove(pos, &stove);
        self.post_block_game_event("minecraft:block_change", pos, Some(Who::Player), None);
        // foodStackToPlace.split(1), on a copy for a creative player.
        self.consume_held();
        self.fd_play(pos, "minecraft:block.lantern.place", 0.5, 1.0);
        self.swing_arm();
        Some(true)
    }

    /// `AbstractStoveBlockEntity.serverTick` for every stove holding food.
    fn fd_stove_tick(&mut self) {
        let dim = self.dim.id();
        let keys: Vec<Key> = match ACTIVE_STOVES.lock() {
            Ok(active) => active.iter().filter(|k| k.3 == dim).copied().collect(),
            Err(_) => return,
        };
        for key in keys {
            let pos = [key.0, key.1, key.2];
            if !self.world.loaded(pos[0], pos[2]) {
                continue;
            }
            let mut stove = self.fd_stove(pos);
            if self.fd_name(pos) != STOVE || stove.is_empty() {
                if let Ok(mut active) = ACTIVE_STOVES.lock() {
                    active.remove(&key);
                }
                continue;
            }
            // shouldDropItems: dropAllItems.
            if self.fd_stove_covered(pos) {
                for slot in stove.items.iter_mut() {
                    if let Some(stack) = slot.take() {
                        self.fd_drop_stack(pos, stack);
                    }
                }
                self.fd_put_stove(pos, &stove);
                self.post_block_game_event("minecraft:block_change", pos, None, None);
                continue;
            }
            let mut cooked = false;
            if self.fd_lit(pos) {
                // cookAndOutputItems.
                for i in 0..STOVE_SLOTS {
                    let Some(item) = stove.items[i].clone() else {
                        continue;
                    };
                    stove.progress[i] += 1;
                    if stove.progress[i] < stove.total[i] {
                        continue;
                    }
                    let result = self
                        .res
                        .recipes()
                        .cook(CAMPFIRE_COOKING, &item)
                        .map_or_else(|| item.clone(), |r| r.result.clone());
                    let r = &mut self.world.random;
                    let vx = r.next_gaussian() * f64::from(0.01f32);
                    let vz = r.next_gaussian() * f64::from(0.01f32);
                    self.spawn_item(
                        result,
                        [
                            f64::from(pos[0]) + 0.5,
                            f64::from(pos[1]) + 1.0,
                            f64::from(pos[2]) + 0.5,
                        ],
                        [vx, f64::from(0.1f32), vz],
                        0,
                    );
                    stove.items[i] = None;
                    cooked = true;
                }
            } else {
                // coolItems.
                for i in 0..STOVE_SLOTS {
                    if stove.progress[i] > 0 {
                        stove.progress[i] = (stove.progress[i] - 2).clamp(0, stove.total[i]);
                    }
                }
            }
            if cooked {
                self.fd_put_stove(pos, &stove);
                self.post_block_game_event("minecraft:block_change", pos, None, None);
            } else {
                // setChanged: the timers are saved with the chunk only.
                spill::put_saved_entity(&mut self.world, pos, stove_tag(pos, &stove));
                self.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
            }
        }
    }

    /// `StoveBlockEntity.particleTick` on the window: a lit stove's food
    /// smokes, each item one tick in five, three smoke particles.
    fn fd_stove_particles(&mut self, key: Key, items: &[Option<ItemStack>; STOVE_SLOTS]) {
        let pos = [key.0, key.1, key.2];
        if !self.fd_lit(pos) {
            return;
        }
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let facing = self.fd_props(state).remove("facing").unwrap_or_else(|| "north".into());
        let [sx, sz] = step(&facing);
        let [cx, cz] = step(clockwise(&facing));
        // Direction.get2DDataValue: south 0, west 1, north 2, east 3.
        let odd = matches!(facing.as_str(), "west" | "east");
        for (i, item) in items.iter().enumerate() {
            if item.is_none() || self.particles.random().next_float() >= 0.2 {
                continue;
            }
            let [mut ox, mut oy] = STOVE_OFFSETS[i].map(f64::from);
            if odd {
                std::mem::swap(&mut ox, &mut oy);
            }
            let x = (f64::from(pos[0]) + 0.5) - (sx * ox) + (cx * ox);
            let y = f64::from(pos[1]) + 1.0;
            let z = (f64::from(pos[2]) + 0.5) - (sz * oy) + (cz * oy);
            for _ in 0..3 {
                self.particles.add(
                    &ParticleOptions::simple(ParticleType::Smoke),
                    [x, y, z],
                    [0.0, 5.0e-4, 0.0],
                );
            }
        }
    }
    /// `DefaultStoveRenderer`: each item flat over the stove, turned to its
    /// front, at its slot's offset, three eighths of its size.
    fn fd_stove_poses(
        &self,
        items: &[Option<ItemStack>; STOVE_SLOTS],
        facing: &str,
    ) -> Vec<(ItemStack, [f32; 3], [f32; 4], f32)> {
        let f = -y_rot(opposite(facing));
        let q = mul(
            axis_rotation([0.0, 1.0, 0.0], f),
            axis_rotation([1.0, 0.0, 0.0], 90.0),
        );
        items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let [ox, oy] = STOVE_OFFSETS[i];
                let o = rotate_vec(q, [ox, oy, 0.0]);
                Some((item.clone()?, [0.5 + o[0], 1.02 + o[1], 0.5 + o[2]], q, 0.375))
            })
            .collect()
    }

    fn fd_pot(&self, pos: [i32; 3]) -> Pot {
        spill::saved_entity(&self.world, pos)
            .map(|t| pot_from_tag(&t))
            .unwrap_or_default()
    }
    /// `inventoryChanged`: saved, sent to the players, ticking.
    fn fd_put_pot(&mut self, pos: [i32; 3], pot: &Pot, update: bool) {
        spill::put_saved_entity(&mut self.world, pos, pot_tag(pos, pot));
        self.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
        let key = (pos[0], pos[1], pos[2], self.dim.id());
        if let Ok(mut active) = ACTIVE_POTS.lock() {
            active.insert(key);
        }
        if update {
            self.mark_block_entity_updated((pos[0], pos[1], pos[2]));
            self.fd_publish_pot(key, pot.items[MEAL_SLOT].is_some());
        }
    }
    fn fd_publish_pot(&mut self, key: Key, meal: bool) {
        let side = self.native_window();
        if let Ok(mut meals) = POT_MEALS.lock() {
            let set = meals.entry(side).or_default();
            if meal {
                set.insert(key);
            } else {
                set.remove(&key);
            }
        }
    }
    /// The item's crafting remainder (`Item.getCraftingRemainingItem`).
    fn fd_crafting_remainder(&self, item: &str) -> Option<String> {
        fd_remainder(item)
            .map(String::from)
            .or_else(|| self.res.items().get(item).and_then(|i| i.remainder.clone()))
    }
    /// A block in one of the heat tags: `#farmersdelight:heat_sources` (with
    /// `#tray_heat_sources`: lava, campfires, fire) or `heat_conductors`.
    fn fd_heat_source(&self, pos: [i32; 3]) -> Option<bool> {
        let name = self.fd_name(pos);
        let source = matches!(
            name.as_str(),
            "minecraft:magma_block"
                | "minecraft:lava_cauldron"
                | "farmersdelight:stove"
                | "minecraft:lava"
                | "minecraft:campfire"
                | "minecraft:soul_campfire"
                | "minecraft:fire"
                | "minecraft:soul_fire"
        );
        if !source {
            return None;
        }
        // A source with a `lit` property heats only lit.
        let state = self.world.block(pos[0], pos[1], pos[2]);
        Some(self.fd_props(state).get("lit").is_none_or(|v| v == "true"))
    }
    /// `HeatableBlockEntity.isHeated`: a heat source below, or one under a
    /// heat conductor (a hopper) below.
    fn fd_pot_heated(&self, pos: [i32; 3]) -> bool {
        let below = [pos[0], pos[1] - 1, pos[2]];
        if let Some(lit) = self.fd_heat_source(below) {
            return lit;
        }
        if matches!(self.fd_name(below).as_str(), "minecraft:hopper" | "create:chute") {
            return self.fd_heat_source([pos[0], pos[1] - 2, pos[2]]).unwrap_or(false);
        }
        false
    }
    /// `CookingPotRecipe.matches`: as many ingredients as filled slots, each
    /// slot matched to a different ingredient (`RecipeMatcher.findMatches`).
    fn fd_pot_recipe(&self, items: &[Option<ItemStack>; POT_SLOTS]) -> Option<PotRecipe> {
        let inputs: Vec<&ItemStack> = items[..MEAL_SLOT].iter().flatten().collect();
        if inputs.is_empty() {
            return None;
        }
        POT_RECIPES
            .iter()
            .find(|r| {
                if r.inputs.len() != inputs.len() {
                    return false;
                }
                let fits: Vec<Vec<usize>> = inputs
                    .iter()
                    .map(|s| {
                        (0..r.inputs.len())
                            .filter(|&j| self.fd_ingredient_test(&r.inputs[j], s))
                            .collect()
                    })
                    .collect();
                fn assign(fits: &[Vec<usize>], i: usize, used: &mut Vec<bool>) -> bool {
                    if i == fits.len() {
                        return true;
                    }
                    for &j in &fits[i] {
                        if !used[j] {
                            used[j] = true;
                            if assign(fits, i + 1, used) {
                                return true;
                            }
                            used[j] = false;
                        }
                    }
                    false
                }
                assign(&fits, 0, &mut vec![false; r.inputs.len()])
            })
            .cloned()
    }
    /// `canCook`: the result fits the meal slot.
    fn fd_pot_can_cook(&self, pot: &Pot, recipe: &PotRecipe) -> bool {
        let Some(meal) = &pot.items[MEAL_SLOT] else {
            return true;
        };
        if *meal.item != *recipe.result || meal.extra.is_some() {
            return false;
        }
        let max = self.fd_max_stack(meal);
        meal.count + recipe.count <= max.max(64) || meal.count + recipe.count <= max
    }
    /// `ejectIngredientRemainder`: out of the pot's counter-clockwise side.
    fn fd_pot_eject(&mut self, pos: [i32; 3], facing: &str, item: &str) {
        let [dx, dz] = counter_clockwise(facing);
        self.spawn_item(
            ItemStack::new(item, 1),
            [
                f64::from(pos[0]) + 0.5 + dx * 0.25,
                f64::from(pos[1]) + 0.7,
                f64::from(pos[2]) + 0.5 + dz * 0.25,
            ],
            [dx * f64::from(0.08f32), f64::from(0.25f32), dz * f64::from(0.08f32)],
            0,
        );
    }
    /// `isContainerValid`.
    fn fd_pot_container_valid(pot: &Pot, stack: Option<&ItemStack>) -> bool {
        match (stack, &pot.container) {
            (Some(s), Some(c)) => *s.item == *c.item,
            _ => false,
        }
    }

    /// `CookingPotBlock.useItemOn` with this hand's stack: the click always
    /// ends here.
    fn fd_pot_use(&mut self, pos: [i32; 3], sneaking: bool) -> Option<bool> {
        let held = self.held().cloned();
        // A pot placed before this port made block entities gets one now.
        if spill::saved_entity(&self.world, pos).is_none() {
            self.fd_put_pot(pos, &Pot::default(), false);
        }
        let state = self.world.block(pos[0], pos[1], pos[2]);
        if held.is_none() && sneaking {
            // The handle, or back to standing on what is below.
            let support = self.fd_props(state).remove("support").unwrap_or_default();
            let below = self.fd_name([pos[0], pos[1] - 1, pos[2]]);
            let next = if support == "handle" {
                if self.fd_tray_heat_source(&below) { "tray" } else { "none" }
            } else {
                "handle"
            };
            if let Some(s) = self.with_props(state, &[("support", next)]) {
                let _ = self.level_set(pos, s);
            }
            self.fd_play(pos, "minecraft:block.lantern.place", 0.7, 1.0);
            self.swing_arm();
            return Some(true);
        }
        let mut pot = self.fd_pot(pos);
        // useHeldItemOnMeal: a valid container takes a serving.
        if Self::fd_pot_container_valid(&pot, held.as_ref())
            && let Some(meal) = pot.items[MEAL_SLOT].as_mut()
        {
            let serving = ItemStack {
                count: 1,
                ..meal.clone()
            };
            meal.count -= 1;
            if meal.count == 0 {
                pot.items[MEAL_SLOT] = None;
            }
            // container.shrink(1), creative or not.
            if let Some(h) = self.held_mut().as_mut() {
                h.count -= 1;
            }
            if self.held().is_some_and(|h| h.count == 0) {
                *self.held_mut() = None;
            }
            self.fd_put_pot(pos, &pot, true);
            if let Some(rest) = self.give(serving)
                && let Some(eye) = self.eye_position()
            {
                self.throw_from_eyes(rest, eye, false);
            }
            self.fd_play(pos, "farmersdelight:block.food.take_portion", 1.0, 1.0);
        } else {
            self.fd_pot_open_menu(pos);
        }
        self.swing_arm();
        Some(true)
    }
    /// `player.openMenu(cookingPot, pos)`: the acting player's menu opens
    /// with the game's own menus, where the server has the players.
    fn fd_pot_open_menu(&mut self, pos: [i32; 3]) {
        let Some(conn) = self.lan_acting else {
            return;
        };
        if let Ok(mut opens) = POT_OPENS.lock() {
            opens.push((conn, pos, self.dim.id()));
        }
    }
    /// NeoForge's `CreativeModeInventoryScreen` page buttons (vanilla `Button`s:
    /// "<" and ">", 20 by 20, 50 above the panel at its two ends) and the
    /// page number (`"%d / %d"`, white, 44 above it, centred), over the panel.
    pub(super) fn fd_creative_pages_draw(&self, batch: &mut horizon_client::hud::HudBatch) {
        use horizon_client::creative::{Creative, farmersdelight_pages as pages};
        // Once per frame, with the screen's widgets: after the panel and the
        // tab buttons, before the stacks and the tooltips.
        if self.screen != super::Screen::Creative || !CREATIVE_PAGES_DUE.swap(false, Ordering::Relaxed) {
            return;
        }
        let (ox, oy) = Self::creative_origin(batch);
        let atlas = &self.gui.atlas;
        for (x, y, step) in creative_page_buttons(ox, oy) {
            let hovered = self.cursor.0 >= x
                && self.cursor.0 < x + 20.0
                && self.cursor.1 >= y
                && self.cursor.1 < y + 20.0;
            let sprite = if hovered {
                "gui/sprites/widget/button_highlighted"
            } else {
                "gui/sprites/widget/button"
            };
            horizon_client::ui::nine_slice(batch, atlas, sprite, x, y, 20.0, 20.0, u32::MAX);
            if hovered {
                horizon_client::widgets::request_cursor(horizon_client::widgets::Cursor::PointingHand);
            }
            // AbstractButton.renderString: centred between 2 and width - 2.
            let label = Text::literal(if step < 0 { "<" } else { ">" });
            let w = atlas.font.width_text(&label);
            batch.text_styled(atlas, &label, x + 10.0 - (w / 2.0).floor(), y + 6.0, u32::MAX, true);
        }
        let page = Text::literal(format!("{} / {}", pages::current() + 1, pages::PAGES));
        let w = atlas.font.width_text(&page);
        let x = ox + (Creative::WIDTH / 2.0).floor() - (w / 2.0).floor();
        batch.text_styled(atlas, &page, x, oy - 44.0, u32::MAX, true);
    }
    /// A click on a page button: the page before or after (none past the ends).
    pub(super) fn fd_creative_pages_click(&mut self) -> bool {
        use horizon_client::creative::farmersdelight_pages as pages;
        let Some(size) = self.renderer.as_ref().map(|r| [r.config.width, r.config.height]) else {
            return false;
        };
        let batch = horizon_client::hud::HudBatch::new(size);
        let (ox, oy) = Self::creative_origin(&batch);
        let (cx, cy) = self.cursor;
        for (x, y, step) in creative_page_buttons(ox, oy) {
            if cx >= x && cx < x + 20.0 && cy >= y && cy < y + 20.0 {
                let page = (pages::current() as i32 + step).clamp(0, pages::PAGES as i32 - 1);
                pages::set(page as usize);
                self.audio.play("minecraft:ui.button.click", None, 0.25, 1.0);
                return true;
            }
        }
        false
    }

    /// `CookingPotBlockEntity.cookingTick` for every pot.
    fn fd_pot_tick(&mut self) {
        let dim = self.dim.id();
        let keys: Vec<Key> = match ACTIVE_POTS.lock() {
            Ok(active) => active.iter().filter(|k| k.3 == dim).copied().collect(),
            Err(_) => return,
        };
        for key in keys {
            let pos = [key.0, key.1, key.2];
            if !self.world.loaded(pos[0], pos[2]) {
                continue;
            }
            if self.fd_name(pos) != POT {
                if let Ok(mut active) = ACTIVE_POTS.lock() {
                    active.remove(&key);
                }
                continue;
            }
            // CookingPotBlock.updateShape from below: the tray follows the
            // heat source under the pot.
            self.fd_support_update(pos);
            let mut pot = self.fd_pot(pos);
            let heated = self.fd_pot_heated(pos);
            let has_input = pot.items[..MEAL_SLOT].iter().any(Option::is_some);
            let mut changed = false;
            let cool = |p: &mut Pot| p.cook_time = (p.cook_time - 2).clamp(0, p.cook_time_total);
            if heated && has_input {
                match self.fd_pot_recipe(&pot.items) {
                    Some(recipe) if self.fd_pot_can_cook(&pot, &recipe) => {
                        changed = self.fd_pot_process(pos, &mut pot, &recipe);
                    }
                    _ => cool(&mut pot),
                }
            } else if pot.cook_time > 0 {
                cool(&mut pot);
            }
            if pot.items[MEAL_SLOT].is_some() {
                let meal_item = pot.items[MEAL_SLOT].as_ref().map(|m| m.item.to_string()).unwrap_or_default();
                // doesMealHaveContainer.
                if pot.container.is_none() && self.fd_crafting_remainder(&meal_item).is_none() {
                    Self::fd_pot_to_output(&mut pot, self.fd_max_stack(&ItemStack::new(&meal_item, 1)));
                    changed = true;
                } else if pot.items[CONTAINER_SLOT].is_some() {
                    let max = self.fd_max_stack(&ItemStack::new(&meal_item, 1));
                    Self::fd_pot_fill_containers(&mut pot, max);
                    changed = true;
                }
            }
            self.fd_put_pot(pos, &pot, changed);
        }
    }
    /// `processCooking`: a meal when the time is up.
    fn fd_pot_process(&mut self, pos: [i32; 3], pot: &mut Pot, recipe: &PotRecipe) -> bool {
        pot.cook_time += 1;
        pot.cook_time_total = recipe.time;
        if pot.cook_time < pot.cook_time_total {
            return false;
        }
        pot.cook_time = 0;
        pot.container = recipe.container.as_deref().map(|c| ItemStack::new(c, 1));
        match &mut pot.items[MEAL_SLOT] {
            Some(meal) => meal.count += recipe.count,
            None => pot.items[MEAL_SLOT] = Some(ItemStack::new(&recipe.result, recipe.count)),
        }
        // setRecipeUsed.
        match pot.used.iter_mut().find(|(id, _)| *id == recipe.id) {
            Some((_, n)) => *n += 1,
            None => pot.used.push((recipe.id.clone(), 1)),
        }
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let facing = self.fd_props(state).remove("facing").unwrap_or_else(|| "north".into());
        for i in 0..MEAL_SLOT {
            let Some(stack) = pot.items[i].clone() else {
                continue;
            };
            if let Some(rest) = self
                .fd_crafting_remainder(&stack.item)
                .or_else(|| remainder_override(&stack.item).map(String::from))
            {
                self.fd_pot_eject(pos, &facing, &rest);
            }
            let left = stack.count - 1;
            pot.items[i] = (left > 0).then(|| ItemStack {
                count: left,
                ..stack
            });
        }
        true
    }
    /// `moveMealToOutput`.
    fn fd_pot_to_output(pot: &mut Pot, max: u32) {
        let Some(meal) = pot.items[MEAL_SLOT].clone() else {
            return;
        };
        let out = pot.items[OUTPUT_SLOT].clone();
        let have = out.as_ref().map_or(0, |o| o.count);
        let n = meal.count.min(max.saturating_sub(have));
        match out {
            None => pot.items[OUTPUT_SLOT] = Some(ItemStack { count: n, ..meal.clone() }),
            Some(o) if *o.item == *meal.item => {
                pot.items[OUTPUT_SLOT] = Some(ItemStack { count: o.count + n, ..o });
            }
            Some(_) => return,
        }
        let left = meal.count - n;
        pot.items[MEAL_SLOT] = (left > 0).then(|| ItemStack { count: left, ..meal });
    }
    /// `useStoredContainersOnMeal`.
    fn fd_pot_fill_containers(pot: &mut Pot, max: u32) {
        let (Some(meal), Some(containers)) =
            (pot.items[MEAL_SLOT].clone(), pot.items[CONTAINER_SLOT].clone())
        else {
            return;
        };
        let out = pot.items[OUTPUT_SLOT].clone();
        let have = out.as_ref().map_or(0, |o| o.count);
        let out_max = out.as_ref().map_or(64, |_| max);
        if !Self::fd_pot_container_valid(pot, Some(&containers)) || have >= out_max {
            return;
        }
        let n = meal.count.min(containers.count).min(max.saturating_sub(have));
        match out {
            None => pot.items[OUTPUT_SLOT] = Some(ItemStack { count: n, ..meal.clone() }),
            Some(o) if *o.item == *meal.item => {
                pot.items[OUTPUT_SLOT] = Some(ItemStack { count: o.count + n, ..o });
            }
            Some(_) => return,
        }
        let meal_left = meal.count - n;
        pot.items[MEAL_SLOT] = (meal_left > 0).then(|| ItemStack { count: meal_left, ..meal });
        let containers_left = containers.count - n;
        pot.items[CONTAINER_SLOT] = (containers_left > 0).then(|| ItemStack {
            count: containers_left,
            ..containers
        });
    }
    /// The window's `CookingPotBlock.animateTick` and `animationTick` for the
    /// heated pots near the player: bubbles every tick one time in five; the
    /// boiling sound (a soup's with a meal in) at `animateTick`'s odds, a
    /// block `ClientLevel.animateTick` picks being boiled one time in ten.
    fn fd_pot_animate(&mut self) {
        let Some(p) = self.player.as_ref().map(|p| p.position) else {
            return;
        };
        let center = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
        let dim = self.dim.id();
        let mut pots: Vec<Key> = Vec::new();
        if let Ok(active) = ACTIVE_POTS.lock() {
            pots.extend(active.iter().filter(|k| k.3 == dim).copied());
        }
        if let Ok(meals) = POT_MEALS.lock() {
            for set in meals.values() {
                pots.extend(set.iter().filter(|k| k.3 == dim).copied());
            }
        }
        pots.sort();
        pots.dedup();
        let meals: std::collections::HashSet<Key> = POT_MEALS
            .lock()
            .map(|m| m.values().flatten().copied().collect())
            .unwrap_or_default();
        for key in pots {
            let pos = [key.0, key.1, key.2];
            if self.fd_name(pos) != POT || !self.fd_pot_heated(pos) {
                continue;
            }
            let (x, y, z) = (f64::from(pos[0]), f64::from(pos[1]), f64::from(pos[2]));
            let r = self.particles.random();
            if r.next_float() < 0.2 {
                let px = x + 0.5 + (r.next_double() * 0.6 - 0.3);
                let pz = z + 0.5 + (r.next_double() * 0.6 - 0.3);
                self.particles.add(
                    &ParticleOptions::simple(ParticleType::BubblePop),
                    [px, y + 0.7, pz],
                    [0.0; 3],
                );
            }
            // Then the steam, one tick in twenty.
            let r = self.particles.random();
            if r.next_float() < 0.05 {
                let sx = x + 0.5 + (r.next_double() * 0.4 - 0.2);
                let sz = z + 0.5 + (r.next_double() * 0.4 - 0.2);
                let motion = if r.next_boolean() { 0.015 } else { 0.005 };
                horizon_client::particles::farmersdelight_steam::add(
                    &mut self.particles,
                    [sx, y + 0.5, sz],
                    [0.0, motion, 0.0],
                );
            }
            // The chance animateTick picks this block this tick: 667 tries
            // each within 16 and 32 blocks, every axis `rand(r) - rand(r)`.
            let pick = |r: f64| -> f64 {
                let axis = |d: i32| -> f64 {
                    let d = f64::from(d.abs());
                    if d >= r { 0.0 } else { (r - d) / (r * r) }
                };
                1.0 - (1.0 - axis(pos[0] - center[0]) * axis(pos[1] - center[1]) * axis(pos[2] - center[2])).powi(667)
            };
            let chance = 1.0 - (1.0 - pick(16.0)) * (1.0 - pick(32.0));
            let r = self.particles.random();
            if r.next_double() < chance && r.next_int_bound(10).unwrap_or(1) == 0 {
                let sound = if meals.contains(&key) {
                    "farmersdelight:block.cooking_pot.boil_soup"
                } else {
                    "farmersdelight:block.cooking_pot.boil"
                };
                let pitch = r.next_float() * 0.2 + 0.9;
                self.audio.play(sound, Some([x + 0.5, y, z + 0.5]), 0.5, pitch);
            }
        }
    }

    fn fd_board(&self, pos: [i32; 3]) -> Board {
        spill::saved_entity(&self.world, pos)
            .map(|t| board_from_tag(&t))
            .unwrap_or_default()
    }
    fn fd_put_board(&mut self, pos: [i32; 3], board: &Board) {
        spill::put_saved_entity(&mut self.world, pos, board_tag(pos, board));
        self.mark_block_entity_updated((pos[0], pos[1], pos[2]));
        self.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
        self.fd_publish(pos, board);
    }
    fn fd_publish(&mut self, pos: [i32; 3], board: &Board) {
        let key = (pos[0], pos[1], pos[2], self.dim.id());
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let facing = self
            .fd_props(state)
            .remove("facing")
            .unwrap_or_else(|| "north".into());
        let side = self.native_window();
        let Ok(mut views) = VIEWS.lock() else {
            return;
        };
        let map = views.entry(side).or_default();
        match &board.stack {
            Some(stack) => {
                map.insert(
                    key,
                    View {
                        stack: stack.clone(),
                        carved: board.carved,
                        facing,
                    },
                );
            }
            None => {
                map.remove(&key);
            }
        }
    }

    /// `CuttingBoardBlock.useItemOn`: whether the arm swings (SUCCESS).
    fn fd_board_use(&mut self, pos: [i32; 3], main: Option<ItemStack>) -> bool {
        let mut board = self.fd_board(pos);
        let Some(held) = main else {
            // An empty hand takes the stored stack back.
            let Some(removed) = board.stack.take() else {
                return false;
            };
            board.carved = false;
            self.fd_put_board(pos, &board);
            if !self.creative {
                // Inventory.add: what doesn't fit is not kept.
                let _ = self.give(removed);
            }
            self.fd_play(pos, "farmersdelight:block.cutting_board.remove_item", 0.25, 0.5);
            return true;
        };
        // canAddItem / addItem: as much of the stack as the slot takes.
        let limit = SLOT_LIMIT.min(self.fd_max_stack(&held));
        let room = match &board.stack {
            _ if board.carved => 0,
            None => limit,
            Some(s) if s.same_item_same_components(&held) => limit.saturating_sub(s.count),
            Some(_) => 0,
        };
        if room > 0 {
            let moved = room.min(held.count);
            match &mut board.stack {
                Some(s) => s.count += moved,
                None => {
                    board.stack = Some(ItemStack {
                        count: moved,
                        ..held.clone()
                    })
                }
            }
            self.fd_put_board(pos, &board);
            if !self.creative {
                let mut rest = held.clone();
                rest.count -= moved;
                self.fd_set_main(Some(rest));
            }
            self.fd_play(pos, "farmersdelight:block.cutting_board.place_item", 1.0, 0.8);
            return true;
        }
        self.fd_board_process(pos, board, held)
    }

    /// `CuttingBoardBlockEntity.processStoredItemUsingTool`.
    fn fd_board_process(&mut self, pos: [i32; 3], mut board: Board, tool: ItemStack) -> bool {
        if board.carved {
            return false;
        }
        let recipe = board.stack.as_ref().and_then(|stored| {
            RECIPES
                .iter()
                .find(|r| {
                    self.fd_ingredient_test(&r.input, stored) && self.fd_ingredient_test(&r.tool, &tool)
                })
                .cloned()
        });
        let (Some(recipe), Some(stored)) = (recipe, board.stack.clone()) else {
            self.fd_action_bar(Text::translate(
                "block.farmersdelight.cutting_board.invalid_tool",
                &[],
            ));
            return false;
        };
        // rollResults: each result's count is rolled item by item.
        let fortune = tool.enchantment_level("minecraft:fortune") as f32;
        let facing = self
            .fd_props(self.world.block(pos[0], pos[1], pos[2]))
            .remove("facing")
            .unwrap_or_else(|| "north".into());
        let [dx, dz] = counter_clockwise(&facing);
        for result in &recipe.results {
            let mut amount = result.count;
            for _ in 0..result.count {
                if self.world.random.next_float() > result.chance + FORTUNE_BONUS * fortune {
                    amount -= 1;
                }
            }
            if amount == 0 {
                continue;
            }
            // ItemUtils.spawnItemEntity: a new ItemEntity, then its motion.
            self.spawn_item(
                ItemStack::new(&result.item, amount),
                [
                    f64::from(pos[0]) + 0.5 + dx * 0.2,
                    f64::from(pos[1]) + 0.2,
                    f64::from(pos[2]) + 0.5 + dz * 0.2,
                ],
                [dx * 0.2, 0.0, dz * 0.2],
                0,
            );
        }
        // hurtAndBreak(1) and the ITEM_USED stat.
        self.damage_held_tool();
        self.stats.add("used", &tool.item, 1);
        // spawnCuttingParticles.
        let particle: ParticleOptions = self.item_particle(&stored);
        self.particles.send_particles(
            &particle,
            [
                f64::from(pos[0]) + 0.5,
                f64::from(pos[1]) + 0.2,
                f64::from(pos[2]) + 0.5,
            ],
            [0.1, 0.1, 0.1],
            0.05,
            5,
        );
        self.fd_board_sound(pos, recipe.sound.as_deref(), &tool, &stored);
        // inventory.extractItem(0, 1).
        let left = stored.count - 1;
        board.stack = (left > 0).then(|| ItemStack {
            count: left,
            ..stored.clone()
        });
        self.fd_put_board(pos, &board);
        let text = if left > 0 {
            Text::translate(
                "block.farmersdelight.cutting_board.remaining_items",
                &[Text::literal(left.to_string())],
            )
        } else {
            Text::literal(String::new())
        };
        self.fd_action_bar(text);
        true
    }

    /// `CuttingBoardBlockEntity.playProcessingSound`.
    fn fd_board_sound(&mut self, pos: [i32; 3], sound: Option<&str>, tool: &ItemStack, item: &ItemStack) {
        if let Some(sound) = sound {
            self.fd_play(pos, sound, 1.0, 1.0);
        } else if self.fd_item_in_tag("c:tools/shear", &tool.item) || &*tool.item == "minecraft:shears" {
            self.fd_play(pos, "minecraft:entity.sheep.shear", 1.0, 1.0);
        } else if self.fd_item_in_tag("c:tools/knife", &tool.item) {
            self.fd_play(pos, "farmersdelight:block.cutting_board.knife_cut", 0.8, 1.0);
        } else if let Some(block) = self.res.items().get(&item.item).and_then(|i| i.block) {
            let sound = self.res.facts().get(block).sounds.break_sound.clone();
            self.fd_play(pos, &sound, 1.0, 0.8);
        } else {
            self.fd_play(pos, "minecraft:block.wood.break", 1.0, 0.8);
        }
    }

    /// `remote::remote_block_reacts`' HEAD: the client's own run of
    /// `CuttingBoardBlock.useItemOn` and `ToolCarvingEvent` against what it
    /// knows of the board. Some(true): SUCCESS (the arm swings), Some(false):
    /// CONSUME, None: not a board, or the block is not used.
    pub(super) fn fd_board_predict(
        &mut self,
        pos: [i32; 3],
        held: Option<&ItemStack>,
        sneaking: bool,
    ) -> Option<bool> {
        if !self.native_on("farmersdelight") {
            return None;
        }
        let name = self.fd_name(pos);
        let hands_full = self.fd_main().is_some() || self.offhand.is_some();
        if pie_slice(&name).is_some() {
            if sneaking && hands_full {
                return None;
            }
            // useItemOn: a knife cuts (SUCCESS); else useWithoutItem on the
            // client: SUCCESS when the player can eat, CONSUME with an empty
            // main hand, else the hand's item is used.
            if held.is_some_and(|s| self.fd_is_knife(&s.item)) || self.fd_can_eat() {
                return Some(true);
            }
            return if self.fd_main().is_none() { Some(false) } else { None };
        }
        if let Some(feast) = feast(&name) {
            if sneaking && hands_full {
                return None;
            }
            let state = self.world.block(pos[0], pos[1], pos[2]);
            let servings = self.fd_int_prop(state, "servings");
            // takeServing on the client: SUCCESS, unless the serving needs a
            // container the hand doesn't hold.
            let has_container = feast
                .container
                .is_none_or(|c| held.is_some_and(|h| &*h.item == c));
            return (servings == 0 || has_container).then_some(true);
        }
        if name == POT {
            // CookingPotBlock.useItemOn is SUCCESS on the client too.
            return (!(sneaking && hands_full)).then_some(true);
        }
        if name == SKILLET {
            // SkilletBlock.useItemOn: CONSUME on the client, whatever the hand.
            return (!(sneaking && hands_full)).then_some(false);
        }
        if is_cabinet(&name) || is_basket(&name) {
            // useWithoutItem: SUCCESS on the client (the arm swings).
            return (!(sneaking && hands_full)).then_some(true);
        }
        if name == TOMATOES {
            // TomatoBlock.useWithoutItem: a ripe vine is picked (SUCCESS).
            let ripe = self.fd_int_prop(self.world.block(pos[0], pos[1], pos[2]), "age") >= 3;
            return (ripe && !(sneaking && hands_full)).then_some(true);
        }
        if name == STOVE {
            if sneaking && hands_full {
                return None;
            }
            // AbstractStoveBlock.useItemOn on the client: lighting and putting
            // out succeed (the arm swings), food that cooks is CONSUME.
            let held = held?;
            let lit = self.fd_lit(pos);
            let douses = self.fd_item_in_tag("minecraft:shovels", &held.item)
                || self.fd_item_in_tag("c:buckets/water", &held.item)
                || &*held.item == "minecraft:water_bucket";
            let lights = matches!(&*held.item, "minecraft:flint_and_steel" | "minecraft:fire_charge");
            if (lit && douses) || (!lit && lights) {
                return Some(true);
            }
            let cooks = self.recipes().cook(CAMPFIRE_COOKING, held).is_some();
            return (cooks && !self.fd_stove_covered(pos)).then_some(false);
        }
        if name != BOARD {
            return None;
        }
        let key = (pos[0], pos[1], pos[2], self.dim.id());
        let view = VIEWS.lock().ok().and_then(|views| {
            [false, true]
                .iter()
                .find_map(|side| views.get(side).and_then(|m| m.get(&key)).cloned())
        });
        let main = self.fd_main();
        if sneaking
            && let Some(tool) = &main
            && view.is_none()
            && self.fd_carvable(tool)
        {
            return Some(true);
        }
        if sneaking && (main.is_some() || self.offhand.is_some()) {
            return None;
        }
        // useItemOn reads the main hand, whichever hand's turn it is.
        let _ = held;
        let Some(main) = main else {
            return Some(false);
        };
        let can_add = match &view {
            None => true,
            Some(v) if v.carved => false,
            Some(v) => {
                v.stack.same_item_same_components(&main)
                    && v.stack.count < SLOT_LIMIT.min(self.fd_max_stack(&main))
            }
        };
        if can_add {
            return Some(false);
        }
        let cuts = view.as_ref().is_some_and(|v| {
            !v.carved
                && RECIPES.iter().any(|r| {
                    self.fd_ingredient_test(&r.input, &v.stack) && self.fd_ingredient_test(&r.tool, &main)
                })
        });
        Some(cuts)
    }
    /// What `carveToolOnBoard` takes: a tiered tool, a trident or shears.
    fn fd_carvable(&self, tool: &ItemStack) -> bool {
        [
            "minecraft:swords",
            "minecraft:axes",
            "minecraft:pickaxes",
            "minecraft:shovels",
            "minecraft:hoes",
            "c:tools/knife",
        ]
        .iter()
        .any(|t| self.fd_item_in_tag(t, &tool.item))
            || matches!(&*tool.item, "minecraft:trident" | "minecraft:shears")
    }

    /// `ToolCarvingEvent.onSneakPlaceTool` and `carveToolOnBoard`: a tiered
    /// tool, a trident or shears goes on an empty board as a carving.
    fn fd_board_carve(&mut self, pos: [i32; 3], tool: &ItemStack) -> bool {
        let tiered = [
            "minecraft:swords",
            "minecraft:axes",
            "minecraft:pickaxes",
            "minecraft:shovels",
            "minecraft:hoes",
            "c:tools/knife",
        ]
        .iter()
        .any(|t| self.fd_item_in_tag(t, &tool.item));
        if !(tiered || matches!(&*tool.item, "minecraft:trident" | "minecraft:shears")) {
            return false;
        }
        let mut board = self.fd_board(pos);
        // addItem must take the whole stack.
        let limit = SLOT_LIMIT.min(self.fd_max_stack(tool));
        if board.carved || board.stack.is_some() || tool.count > limit {
            return false;
        }
        board.stack = Some(tool.clone());
        board.carved = true;
        self.fd_put_board(pos, &board);
        if !self.creative {
            self.fd_set_main(None);
        }
        self.fd_play(pos, "farmersdelight:block.cutting_board.carve_tool", 1.0, 0.8);
        true
    }

    /// `block_items::apply_item_block_state`'s HEAD: the state a Farmer's
    /// Delight block's `getStateForPlacement` gives, set on the block just
    /// placed at `pos`.
    pub(super) fn fd_placement(
        &mut self,
        pos: [i32; 3],
        state: BlockStateId,
        stack: &ItemStack,
    ) -> Option<horizon_client::Result<BlockStateId>> {
        if !self.native_on("farmersdelight")
            || !self.res.facts().get(state).name.starts_with("farmersdelight:")
        {
            return None;
        }
        // The block entity the block makes: the skillet's keeps the item it
        // came from (`SkilletItem.updateCustomBlockEntityTag`); a pot's ticks.
        match &*self.res.facts().get(state).name {
            SKILLET => {
                let skillet = Skillet {
                    skillet: Some(ItemStack {
                        count: 1,
                        ..stack.clone()
                    }),
                    ..Skillet::default()
                };
                self.fd_put_skillet(pos, &skillet, true);
            }
            POT => self.fd_put_pot(pos, &Pot::default(), false),
            // WildRiceBlock.setPlacedBy: the upper half is out of the water.
            WILD_RICE => {
                let top = [pos[0], pos[1] + 1, pos[2]];
                let upper = self.world.block(top[0], top[1], top[2]);
                if self.fd_name(top) == WILD_RICE
                    && let Some(dry) = self.with_props(upper, &[("waterlogged", "false")])
                    && dry != upper
                {
                    let _ = self.level_set(top, dry);
                }
            }
            _ => {}
        }
        let click = CLICK.lock().ok()?.take().filter(|c| self.ticks.saturating_sub(c.tick) <= 1)?;
        let water = click
            .water
            .iter()
            .find(|(p, _)| *p == pos)
            .is_some_and(|(_, w)| *w);
        let new = self.fd_placement_state(pos, state, click.face, water, click.sneaking)?;
        if new != state
            && let Err(e) = self.level_set(pos, new)
        {
            return Some(Err(e));
        }
        for (q, next) in self.fd_placed_by(pos, new, click.sneaking) {
            let _ = self.level_set(q, next);
        }
        Some(Ok(new))
    }
    /// `setPlacedBy` of the blocks that make a second one:
    /// `TatamiMatBlock`'s head beside its foot, and `TatamiBlock`'s pairing
    /// with the tatami it was placed against (unless sneaking).
    pub(super) fn fd_placed_by(
        &self,
        pos: [i32; 3],
        state: BlockStateId,
        sneaking: bool,
    ) -> Vec<([i32; 3], BlockStateId)> {
        let props = self.fd_props(state);
        let facing = props.get("facing").cloned().unwrap_or_default();
        let other = add3(pos, dir_step(&facing));
        match &*self.res.facts().get(state).name {
            FULL_MAT => self
                .with_props(state, &[("part", "head")])
                .map(|head| vec![(other, head)])
                .unwrap_or_default(),
            TATAMI if !sneaking => {
                let there = self.world.block(other[0], other[1], other[2]);
                if self.fd_name(other) == TATAMI
                    && self.fd_props(there).get("paired").map(String::as_str) == Some("false")
                    && let Some(paired) = self.with_props(there, &[("facing", opposite(&facing)), ("paired", "true")])
                {
                    vec![(other, paired)]
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }
    /// `remote::remote_can_place`'s HEAD: the client's prediction of a
    /// placement gets the Farmer's Delight block's own state, so the window
    /// never draws the block in its default state first.
    pub(super) fn fd_predict_placement(
        &self,
        placed: [i32; 3],
        state: BlockStateId,
        face: [i32; 3],
    ) -> BlockStateId {
        if !self.native_on("farmersdelight") {
            return state;
        }
        // The prediction ran on a copy: the real level still holds the water.
        let water = self.fd_water_source(placed);
        let sneaking = self.keys.contains(&winit::keyboard::KeyCode::ShiftLeft);
        self.fd_placement_state(placed, state, face, water, sneaking)
            .unwrap_or(state)
    }
    /// `getStateForPlacement` of the Farmer's Delight block whose state is
    /// `state`, placed at `pos` by a click on `face`.
    fn fd_placement_state(
        &self,
        pos: [i32; 3],
        state: BlockStateId,
        face: [i32; 3],
        water: bool,
        sneaking: bool,
    ) -> Option<BlockStateId> {
        let name = self.res.facts().get(state).name.to_string();
        let block = name.strip_prefix("farmersdelight:")?;
        let wl = if water { "true" } else { "false" };
        let h = horizontal_direction(self.yaw);
        let face = face_name(face);
        let below = self.fd_name([pos[0], pos[1] - 1, pos[2]]);
        let tray = if self.fd_tray_heat_source(&below) { "tray" } else { "none" };
        let props: Vec<(&str, &str)> = match block {
            "cutting_board" => vec![("facing", opposite(h)), ("waterlogged", wl)],
            "stove" => vec![("facing", opposite(h)), ("lit", "true")],
            "wooden_basket" | "bamboo_basket" => vec![
                ("facing", opposite(nearest_looking_direction(self.yaw, self.pitch))),
                ("waterlogged", wl),
            ],
            "cooking_pot" => vec![
                ("facing", opposite(h)),
                ("waterlogged", wl),
                ("support", if face == "down" { "handle" } else { tray }),
            ],
            "skillet" => vec![
                ("facing", h),
                ("waterlogged", wl),
                // SkilletBlock.SUPPORT is a boolean.
                ("support", if tray == "tray" { "true" } else { "false" }),
            ],
            "apple_pie" | "sweet_berry_cheesecake" | "chocolate_pie" | "pumpkin_pie" => {
                vec![("facing", h)]
            }
            "roast_chicken_block"
            | "stuffed_pumpkin_block"
            | "honey_glazed_ham_block"
            | "shepherds_pie_block"
            | "rice_roll_medley_block"
            | "gleaming_salad_block"
            | "half_tatami_mat" => vec![("facing", opposite(h))],
            b if b.ends_with("_cabinet") => vec![("facing", opposite(h))],
            "rice_bale" => vec![("facing", face)],
            // RopeBlock.getStateWithConnections.
            "rope" => self.fd_rope_props(pos, face, water),
            // TatamiMatBlock: facing the player's way, the head beyond it.
            "full_tatami_mat" => vec![("facing", h), ("part", "foot")],
            // TatamiBlock: facing the block clicked, paired with it when that
            // is an unpaired tatami and the player isn't sneaking.
            "tatami" => {
                let toward = opposite(face);
                let clicked = add3(pos, dir_step(toward));
                let state = self.world.block(clicked[0], clicked[1], clicked[2]);
                let pairing = !sneaking
                    && self.fd_name(clicked) == TATAMI
                    && self.fd_props(state).get("paired").map(String::as_str) == Some("false");
                vec![("facing", toward), ("paired", if pairing { "true" } else { "false" })]
            }
            "rope_fence" => {
                let c = |d| if self.fd_rope_connects(pos, d) { "true" } else { "false" };
                vec![
                    ("north", c(horizon_core::geometry::Direction::North)),
                    ("east", c(horizon_core::geometry::Direction::East)),
                    ("south", c(horizon_core::geometry::Direction::South)),
                    ("west", c(horizon_core::geometry::Direction::West)),
                    ("waterlogged", wl),
                ]
            }
            "safety_net" => vec![("waterlogged", wl)],
            _ => return None,
        };
        self.with_props(state, &props)
    }
    fn fd_tray_heat_source(&self, block: &str) -> bool {
        tray_heat_source(block)
    }
}

/// `#farmersdelight:tray_heat_sources`, as the mod's tag file lists it.
fn tray_heat_source(block: &str) -> bool {
    {
        static TAG: LazyLock<Vec<String>> = LazyLock::new(|| {
            let path = Game::options_path()
                .with_file_name("horizon-mods")
                .join("farmersdelight/data/farmersdelight/tags/block/tray_heat_sources.json");
            let v: serde_json::Value = std::fs::read(path)
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_default();
            v["values"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|e| e.as_str().or_else(|| e["id"].as_str()).map(String::from))
                .flat_map(|e| match e.as_str() {
                    "#minecraft:campfires" => {
                        vec!["minecraft:campfire".to_owned(), "minecraft:soul_campfire".to_owned()]
                    }
                    "#minecraft:fire" => {
                        vec!["minecraft:fire".to_owned(), "minecraft:soul_fire".to_owned()]
                    }
                    _ => vec![e],
                })
                .collect()
        });
        TAG.iter().any(|t| t == block)
    }
}

impl Game<'_> {
    /// `native_hooks::Game::native_tick` (a hook point): every second each
    /// side publishes the boards and stoves its world holds; the
    /// authoritative side ticks the stoves; the window draws them.
    pub(super) fn fd_board_tick(&mut self) {
        if !self.native_on("farmersdelight") {
            return;
        }
        if self.ticks % 20 == 0 {
            self.fd_board_scan();
        }
        // A client of a server leaves the block entities to the server.
        if self.remote.is_none() {
            self.fd_stove_tick();
            self.fd_pot_tick();
            self.fd_skillet_tick();
        }
        self.fd_rope_tick();
        if self.native_window() {
            self.fd_board_display_tick();
            self.fd_pot_animate();
        }
    }
    /// The cutting boards and stoves of the loaded chunks' block entities.
    fn fd_board_scan(&mut self) {
        let dim = self.dim.id();
        let mut boards: Vec<([i32; 3], Board)> = Vec::new();
        let mut stoves: Vec<([i32; 3], Stove)> = Vec::new();
        let mut pots: Vec<([i32; 3], bool)> = Vec::new();
        let mut skillets: Vec<([i32; 3], Skillet)> = Vec::new();
        for chunk in self.world.chunks.values() {
            let Some(saved) = chunk.saved.as_ref() else {
                continue;
            };
            for tag in &saved.block_entities {
                let Some(Tag::String(id)) = tag.get("id") else {
                    continue;
                };
                let id = id.to_string_lossy();
                if id == POT {
                    let int = |k: &str| match tag.get(k) {
                        Some(Tag::Int(v)) => Some(*v),
                        _ => None,
                    };
                    if let (Some(x), Some(y), Some(z)) = (int("x"), int("y"), int("z")) {
                        pots.push(([x, y, z], pot_items(tag)[MEAL_SLOT].is_some()));
                    }
                    continue;
                }
                if id == SKILLET {
                    let int = |k: &str| match tag.get(k) {
                        Some(Tag::Int(v)) => Some(*v),
                        _ => None,
                    };
                    if let (Some(x), Some(y), Some(z)) = (int("x"), int("y"), int("z")) {
                        skillets.push(([x, y, z], skillet_from_tag(tag)));
                    }
                    continue;
                }
                if id != BOARD && id != STOVE {
                    continue;
                }
                let int = |k: &str| match tag.get(k) {
                    Some(Tag::Int(v)) => Some(*v),
                    _ => None,
                };
                let (Some(x), Some(y), Some(z)) = (int("x"), int("y"), int("z")) else {
                    continue;
                };
                if id == BOARD {
                    boards.push(([x, y, z], board_from_tag(tag)));
                } else {
                    stoves.push(([x, y, z], stove_from_tag(tag)));
                }
            }
        }
        let side = self.native_window();
        if let Ok(mut views) = VIEWS.lock() {
            views.entry(side).or_default().retain(|k, _| k.3 != dim);
        }
        if let Ok(mut views) = STOVE_VIEWS.lock() {
            views.entry(side).or_default().retain(|k, _| k.3 != dim);
        }
        for (pos, board) in boards {
            if self.fd_name(pos) == BOARD {
                self.fd_publish(pos, &board);
            }
        }
        if let Ok(mut meals) = POT_MEALS.lock() {
            meals.entry(side).or_default().retain(|k| k.3 != dim);
        }
        for (pos, meal) in pots {
            if self.fd_name(pos) != POT {
                continue;
            }
            let key = (pos[0], pos[1], pos[2], dim);
            self.fd_publish_pot(key, meal);
            if self.remote.is_none()
                && let Ok(mut active) = ACTIVE_POTS.lock()
            {
                active.insert(key);
            }
        }
        if let Ok(mut views) = SKILLET_VIEWS.lock() {
            views.entry(side).or_default().retain(|k, _| k.3 != dim);
        }
        for (pos, skillet) in skillets {
            if self.fd_name(pos) != SKILLET {
                continue;
            }
            let key = (pos[0], pos[1], pos[2], dim);
            self.fd_publish_skillet(key, &skillet);
            if self.remote.is_none()
                && let Ok(mut active) = ACTIVE_SKILLETS.lock()
            {
                active.insert(key);
            }
        }
        for (pos, stove) in stoves {
            if self.fd_name(pos) != STOVE {
                continue;
            }
            self.fd_publish_stove(pos, &stove.items);
            if self.remote.is_none()
                && !stove.is_empty()
                && let Ok(mut active) = ACTIVE_STOVES.lock()
            {
                active.insert((pos[0], pos[1], pos[2], dim));
            }
        }
    }

    /// `CuttingBoardRenderer` and `DefaultStoveRenderer` for every board and
    /// stove: one item display per model, posed as the renderer's pose stack.
    fn fd_board_display_tick(&mut self) {
        let dim = self.dim.id();
        let mut wanted: HashMap<Key, Want> = HashMap::new();
        if let Ok(views) = VIEWS.lock() {
            // The server's word first, the window's own world after.
            for side in [false, true] {
                for (k, v) in views.get(&side).into_iter().flatten() {
                    if k.3 == dim {
                        wanted.entry(*k).or_insert_with(|| Want::Board(v.clone()));
                    }
                }
            }
        }
        let mut stoves: Vec<(Key, [Option<ItemStack>; STOVE_SLOTS])> = Vec::new();
        if let Ok(views) = STOVE_VIEWS.lock() {
            for side in [false, true] {
                for (k, v) in views.get(&side).into_iter().flatten() {
                    if k.3 == dim && !stoves.iter().any(|(s, _)| s == k) {
                        stoves.push((*k, v.clone()));
                    }
                }
            }
        }
        for (key, items) in stoves {
            self.fd_stove_particles(key, &items);
            let state = self.world.block(key.0, key.1, key.2);
            let facing = self.fd_props(state).remove("facing").unwrap_or_else(|| "north".into());
            wanted.insert(key, Want::Stove(items, facing));
        }
        let mut skillets: Vec<(Key, Option<ItemStack>, u32)> = Vec::new();
        if let Ok(views) = SKILLET_VIEWS.lock() {
            for side in [false, true] {
                for (k, (stack, fire)) in views.get(&side).into_iter().flatten() {
                    if k.3 == dim && !skillets.iter().any(|(s, _, _)| s == k) {
                        skillets.push((*k, stack.clone(), *fire));
                    }
                }
            }
        }
        self.fd_skillet_animate(&skillets);
        for (key, stack, _) in skillets {
            let Some(stack) = stack else {
                continue;
            };
            if self.fd_name([key.0, key.1, key.2]) != SKILLET {
                continue;
            }
            let state = self.world.block(key.0, key.1, key.2);
            let facing = self.fd_props(state).remove("facing").unwrap_or_else(|| "north".into());
            wanted.insert(key, Want::Skillet(stack, facing));
        }
        let Ok(mut displays) = DISPLAYS.lock() else {
            return;
        };
        // Displays a world change took away are gone.
        displays.retain(|_, (ids, _)| ids.iter().all(|id| self.text_displays.contains_key(id)));
        // Blocks that hold nothing any more lose their models at once.
        let gone: Vec<Key> = displays
            .keys()
            .filter(|k| !wanted.contains_key(*k))
            .copied()
            .collect();
        for key in gone {
            for id in displays.remove(&key).map(|d| d.0).unwrap_or_default() {
                self.text_displays.remove(&id);
            }
        }
        for (key, want) in wanted {
            let sig = want.signature();
            if displays.get(&key).is_some_and(|(_, s)| *s == sig) {
                continue;
            }
            let models: Vec<(ItemStack, [f32; 3], [f32; 4], f32)> = match &want {
                Want::Board(view) => self
                    .fd_board_poses(view)
                    .into_iter()
                    .map(|(o, l, s)| (view.stack.clone(), o, l, s))
                    .collect(),
                Want::Stove(items, facing) => self.fd_stove_poses(items, facing),
                Want::Skillet(stack, facing) => self
                    .fd_skillet_poses(stack, facing)
                    .into_iter()
                    .map(|(o, l, s)| (stack.clone(), o, l, s))
                    .collect(),
            };
            let mut ids = Vec::new();
            for (stack, offset, left, scale) in models {
                let id = NEXT_DISPLAY.fetch_sub(1, Ordering::Relaxed);
                let mut display = RemoteTextDisplay::new(
                    Shows::Item,
                    [
                        f64::from(key.0) + f64::from(offset[0]),
                        f64::from(key.1) + f64::from(offset[1]),
                        f64::from(key.2) + f64::from(offset[2]),
                    ],
                    0.0,
                );
                display.metadata(&[
                    (12, 0, Value::Floats(vec![scale; 3])),
                    (13, 0, Value::Floats(left.to_vec())),
                    // The display draws its item half a turn about Y after its
                    // transformation; the renderers have no such turn.
                    (14, 0, Value::Floats(vec![0.0, 1.0, 0.0, 0.0])),
                ]);
                display.show(&[(24, 0, Value::Byte(FIXED_CONTEXT))], Some(Some(stack)));
                // Posed before the first frame draws it.
                display.fd_settle();
                self.text_displays.insert(id, display);
                ids.push(id);
            }
            // The old contents' models go in the same tick the new ones come.
            if let Some((old, _)) = displays.insert(key, (ids, sig)) {
                for id in old {
                    self.text_displays.remove(&id);
                }
            }
        }
    }
    /// The renderer's poses: per model, its offset in the block, its
    /// rotation and its scale.
    fn fd_board_poses(&mut self, view: &View) -> Vec<([f32; 3], [f32; 4], f32)> {
        let stack = &view.stack;
        // getBlockState().getValue(FACING).getOpposite().
        let direction = opposite(&view.facing);
        let f = -y_rot(direction);
        if view.carved {
            // renderItemCarved.
            let tool = &*stack.item;
            let pose_angle = if self.fd_item_in_tag("minecraft:pickaxes", tool)
                || self.fd_item_in_tag("minecraft:hoes", tool)
            {
                225.0
            } else if tool == "minecraft:trident" {
                135.0
            } else {
                180.0
            };
            let left = mul(
                axis_rotation([0.0, 1.0, 0.0], f + 180.0),
                axis_rotation([0.0, 0.0, 1.0], pose_angle),
            );
            return vec![([0.5, 0.23, 0.5], left, 0.6)];
        }
        // isGui3d of the model, unless the item lies flat on boards.
        let block_model = self
            .resolve_item(stack, DisplayContext::Fixed, None, 0)
            .uses_block_light()
            && !self.fd_item_in_tag("farmersdelight:flat_on_cutting_board", &stack.item);
        // getModelCount.
        let max = self.fd_max_stack(stack) as f32;
        let count = if stack.count > 1 {
            1 + ((stack.count as f32 / max) * 4.0).ceil() as usize
        } else {
            1
        };
        // The seed is Item.getId(item) + damage; ids differ between game
        // versions, so the item's name stands for its id.
        let id = stack
            .item
            .bytes()
            .fold(0i32, |h, b| h.wrapping_mul(31).wrapping_add(i32::from(b)));
        let mut random = JavaRandom::new(i64::from(id.wrapping_add(stack.damage as i32)));
        let mut poses = Vec::new();
        for i in 0..count {
            let (x, z) = if count == 1 {
                (0.0, 0.0)
            } else {
                let x = (random.next_float() * 2.0 - 1.0) * 0.15 * 0.5;
                let z = (random.next_float() * 2.0 - 1.0) * 0.15 * 0.5;
                (x, z)
            };
            let step = 0.03 * (i as f32 + 1.0);
            if block_model {
                // renderBlock.
                let left = axis_rotation([0.0, 1.0, 0.0], f);
                poses.push(([0.5 + x, 0.27 + step, 0.5 + z], left, 0.8));
            } else {
                // renderItemLayingDown.
                let left = mul(
                    axis_rotation([0.0, 1.0, 0.0], f),
                    axis_rotation([1.0, 0.0, 0.0], 90.0),
                );
                poses.push(([0.5 + x, 0.08 + step, 0.5 + z], left, 0.6));
            }
        }
        poses
    }
}

// ------------------------------------------------------ cooking pot menu
//
// `CookingPotMenu` (ModMenuTypes.COOKING_POT) and `CookingPotScreen`. The
// server runs the menu for each player whose use opened it, with vanilla's
// packets (OpenScreen, ContainerClick, the slot, carried and data slot
// sync); the window draws the screen and predicts its clicks with the same
// `doClick`, so nothing it shows is taken back.

/// `ModMenuTypes.COOKING_POT`.
pub(super) const POT_MENU: &str = "farmersdelight:cooking_pot";
/// `CookingPotMenu`'s slots: the pot's nine, the inventory's 27, the hotbar.
const POT_MENU_SLOTS: usize = POT_SLOTS + 36;
/// The screen's textures (copied where the GUI atlas reads them).
const POT_TEXTURE: &str = "gui/farmersdelight/cooking_pot";
const BOWL_ICON: &str = "gui/farmersdelight/empty_container_slot_bowl";
/// `CookingPotScreen.HEAT_ICON`.
const HEAT_ICON: [f32; 4] = [47.0, 55.0, 17.0, 15.0];
/// The click inputs (`ClickType` ordinals).
const PICKUP: i32 = 0;
const QUICK_MOVE: i32 = 1;
const SWAP: i32 = 2;
const CLONE: i32 = 3;
const THROW: i32 = 4;
const QUICK_CRAFT: i32 = 5;
const PICKUP_ALL: i32 = 6;

/// `quickcraftStatus`, `quickcraftType` and `quickcraftSlots`.
#[derive(Clone, Default)]
struct Drag {
    status: i32,
    kind: i32,
    slots: Vec<usize>,
}

/// A player's open cooking pot menu on the server.
struct PotMenu {
    window: i32,
    pos: [i32; 3],
    dim: &'static str,
    state: i32,
    /// `remoteSlots`, `remoteCarried`, `remoteDataSlots`.
    sent: Vec<Option<ItemStack>>,
    sent_carried: Option<ItemStack>,
    sent_data: [i32; 2],
    drag: Drag,
}
static POT_MENUS: LazyLock<Mutex<HashMap<u32, PotMenu>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
/// Menus a player's use opened, for the server's next pass: (player, pot, level).
static POT_OPENS: LazyLock<Mutex<Vec<(u32, [i32; 3], &'static str)>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
/// The open pots' `getContainer` item, which the screen reads of its block
/// entity (the meal's tooltip, and quick move into the container slot).
static POT_CONTAINERS: LazyLock<Mutex<HashMap<Key, Option<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The window's open pot screen: its block, the data slots, the drag.
#[derive(Default)]
struct ClientPot {
    pos: [i32; 3],
    data: [i32; 2],
    drag: Drag,
}
static CLIENT_POT: LazyLock<Mutex<ClientPot>> = LazyLock::new(|| Mutex::new(ClientPot::default()));

/// The slots a click on a cooking pot menu sees: the pot's handler, the
/// player's inventory and the menu's carried stack.
struct PotClick<'a> {
    pot: &'a mut [Option<ItemStack>; POT_SLOTS],
    inventory: &'a mut [Option<ItemStack>],
    offhand: &'a mut Option<ItemStack>,
    carried: &'a mut Option<ItemStack>,
    creative: bool,
    /// `blockEntity.getContainer()`'s item.
    container: Option<String>,
    max: &'a dyn Fn(&str) -> u32,
    /// `#farmersdelight:serving_containers`.
    serving: &'a dyn Fn(&str) -> bool,
    /// The result slot's `onTake` ran (`checkTakeAchievements`).
    took_output: bool,
    /// Stacks the player throws (`player.drop`).
    drops: Vec<ItemStack>,
}

impl PotClick<'_> {
    fn at(&self, i: usize) -> Option<&ItemStack> {
        match i {
            0..POT_SLOTS => self.pot[i].as_ref(),
            POT_SLOTS..36 => self.inventory[i].as_ref(),
            _ => self.inventory.get(i - 36).and_then(Option::as_ref),
        }
    }
    fn cell(&mut self, i: usize) -> &mut Option<ItemStack> {
        match i {
            0..POT_SLOTS => &mut self.pot[i],
            POT_SLOTS..36 => &mut self.inventory[i],
            _ => &mut self.inventory[i - 36],
        }
    }
    fn all(&self) -> Vec<Option<ItemStack>> {
        (0..POT_MENU_SLOTS).map(|i| self.at(i).cloned()).collect()
    }
    fn item_max(&self, s: &ItemStack) -> u32 {
        (self.max)(&s.item).max(1)
    }
    /// `Slot.mayPlace`: the meal display and the result take nothing.
    fn may_place(i: usize) -> bool {
        i != MEAL_SLOT && i != OUTPUT_SLOT
    }
    /// `Slot.mayPickup`: `CookingPotMealSlot` gives nothing.
    fn may_pickup(i: usize) -> bool {
        i != MEAL_SLOT
    }
    /// `Slot.allowModification`.
    fn allow_modification(i: usize) -> bool {
        Self::may_pickup(i) && Self::may_place(i)
    }
    /// `Slot.getMaxStackSize(stack)`: the pot's handler holds 64 at most.
    fn slot_max(&self, i: usize, s: &ItemStack) -> u32 {
        let max = self.item_max(s);
        if i < POT_SLOTS { max.min(SLOT_LIMIT) } else { max }
    }
    /// `Slot.onTake`: the result slot's `checkTakeAchievements`.
    fn on_take(&mut self, i: usize) {
        if i == OUTPUT_SLOT {
            self.took_output = true;
        }
    }
    /// `canItemQuickReplace(slot, stack, true)`.
    fn quick_replace(&self, i: usize, stack: &ItemStack) -> bool {
        match self.at(i) {
            None => true,
            Some(s) => s.same_item_same_components(stack) && s.count <= self.item_max(stack),
        }
    }
    /// `Slot.tryRemove`.
    fn try_remove(&mut self, i: usize, count: u32, decrement: u32) -> Option<ItemStack> {
        if !Self::may_pickup(i) {
            return None;
        }
        let have = self.at(i)?.count;
        if !Self::allow_modification(i) && decrement < have {
            return None;
        }
        let n = count.min(decrement).min(have);
        if n == 0 {
            return None;
        }
        let cell = self.cell(i);
        let s = cell.as_mut()?;
        let taken = ItemStack {
            count: n,
            ..s.clone()
        };
        s.count -= n;
        if s.count == 0 {
            *cell = None;
        }
        Some(taken)
    }
    /// `Slot.safeTake`.
    fn safe_take(&mut self, i: usize, count: u32, decrement: u32) -> Option<ItemStack> {
        let taken = self.try_remove(i, count, decrement)?;
        self.on_take(i);
        Some(taken)
    }
    /// `Slot.safeInsert(stack, n)`.
    fn safe_insert(&mut self, i: usize, stack: &mut ItemStack, n: u32) {
        if stack.count == 0 || !Self::may_place(i) {
            return;
        }
        let have = self.at(i).map_or(0, |s| s.count);
        let k = n
            .min(stack.count)
            .min(self.slot_max(i, stack).saturating_sub(have));
        if k == 0 {
            return;
        }
        let cell = self.cell(i);
        match cell.as_mut() {
            None => {
                *cell = Some(ItemStack {
                    count: k,
                    ..stack.clone()
                });
                stack.count -= k;
            }
            Some(s) if s.same_item_same_components(stack) => {
                s.count += k;
                stack.count -= k;
            }
            Some(_) => {}
        }
    }
    /// `moveItemStackTo`.
    fn move_to(&mut self, stack: &mut ItemStack, start: usize, end: usize, reverse: bool) -> bool {
        let order: Vec<usize> = if reverse {
            (start..end).rev().collect()
        } else {
            (start..end).collect()
        };
        let mut moved = false;
        if self.item_max(stack) > 1 {
            for &i in &order {
                if stack.count == 0 {
                    break;
                }
                let k = match self.at(i) {
                    Some(s) if s.same_item_same_components(stack) => self.slot_max(i, s),
                    _ => continue,
                };
                let Some(s) = self.cell(i).as_mut() else {
                    continue;
                };
                let j = s.count + stack.count;
                if j <= k {
                    stack.count = 0;
                    s.count = j;
                    moved = true;
                } else if s.count < k {
                    stack.count -= k - s.count;
                    s.count = k;
                    moved = true;
                }
            }
        }
        if stack.count > 0 {
            for &i in &order {
                if self.at(i).is_none() && Self::may_place(i) {
                    let n = stack.count.min(self.slot_max(i, stack));
                    *self.cell(i) = Some(ItemStack {
                        count: n,
                        ..stack.clone()
                    });
                    stack.count -= n;
                    moved = true;
                    break;
                }
            }
        }
        moved
    }
    /// `CookingPotMenu.quickMoveStack`.
    fn quick_move(&mut self, i: usize) -> Option<ItemStack> {
        let mut stack = self.cell(i).take()?;
        let copy = stack.clone();
        let moved = if i == OUTPUT_SLOT {
            self.move_to(&mut stack, POT_SLOTS, POT_MENU_SLOTS, true)
        } else if i > OUTPUT_SLOT {
            if (self.serving)(&stack.item) || self.container.as_deref() == Some(&*stack.item) {
                self.move_to(&mut stack, CONTAINER_SLOT, OUTPUT_SLOT, false)
            } else {
                self.move_to(&mut stack, 0, MEAL_SLOT, false)
            }
        } else {
            self.move_to(&mut stack, POT_SLOTS, POT_MENU_SLOTS, false)
        };
        let left = stack.count;
        *self.cell(i) = (left > 0).then_some(stack);
        if !moved || left == copy.count {
            return None;
        }
        self.on_take(i);
        Some(copy)
    }
    /// The hotbar slot (or the off hand, 40) a SWAP click names.
    fn hot(&mut self, button: i32) -> &mut Option<ItemStack> {
        if button == 40 {
            &mut *self.offhand
        } else {
            &mut self.inventory[button as usize]
        }
    }
    /// `Inventory.add`: stacks of the item first, then a free slot; what
    /// does not fit is dropped.
    fn add_to_inventory(&mut self, mut stack: ItemStack) {
        let max = self.item_max(&stack);
        for s in self.inventory.iter_mut().flatten() {
            if stack.count == 0 {
                return;
            }
            if s.same_item_same_components(&stack) && s.count < max {
                let n = (max - s.count).min(stack.count);
                s.count += n;
                stack.count -= n;
            }
        }
        if stack.count == 0 {
            return;
        }
        if let Some(free) = self.inventory.iter_mut().find(|s| s.is_none()) {
            *free = Some(stack);
        } else {
            self.drops.push(stack);
        }
    }
}

/// `AbstractContainerMenu.doClick` on a cooking pot menu.
fn pot_click(v: &mut PotClick, drag: &mut Drag, slot: i32, button: i32, input: i32) {
    let index = usize::try_from(slot).ok().filter(|&i| i < POT_MENU_SLOTS);
    if input == QUICK_CRAFT {
        let previous = drag.status;
        drag.status = button & 3;
        if (previous != 1 || drag.status != 2) && previous != drag.status {
            *drag = Drag::default();
        } else if v.carried.is_none() {
            *drag = Drag::default();
        } else if drag.status == 0 {
            drag.kind = (button >> 2) & 3;
            if drag.kind == 0 || drag.kind == 1 || (drag.kind == 2 && v.creative) {
                drag.status = 1;
                drag.slots.clear();
            } else {
                *drag = Drag::default();
            }
        } else if drag.status == 1 {
            let (Some(i), Some(c)) = (index, v.carried.clone()) else {
                return;
            };
            if v.quick_replace(i, &c)
                && PotClick::may_place(i)
                && (drag.kind == 2 || c.count as usize > drag.slots.len())
                && !drag.slots.contains(&i)
            {
                drag.slots.push(i);
            }
        } else if drag.status == 2 {
            if !drag.slots.is_empty() {
                if drag.slots.len() == 1 {
                    let (only, kind) = (drag.slots[0], drag.kind);
                    *drag = Drag::default();
                    pot_click(v, drag, only as i32, kind, PICKUP);
                    return;
                }
                let Some(c) = v.carried.clone() else {
                    *drag = Drag::default();
                    return;
                };
                let mut k = i64::from(c.count);
                let n_slots = drag.slots.len();
                for i in drag.slots.clone() {
                    if v.quick_replace(i, &c)
                        && PotClick::may_place(i)
                        && (drag.kind == 2 || c.count as usize >= n_slots)
                    {
                        let l = v.at(i).map_or(0, |s| s.count);
                        let m = v.item_max(&c).min(v.slot_max(i, &c));
                        // getQuickCraftPlaceCount.
                        let each = match drag.kind {
                            0 => c.count / n_slots as u32,
                            1 => 1,
                            _ => v.item_max(&c),
                        };
                        let n = (each + l).min(m);
                        k -= i64::from(n) - i64::from(l);
                        *v.cell(i) = (n > 0).then(|| ItemStack {
                            count: n,
                            ..c.clone()
                        });
                    }
                }
                *v.carried = (k > 0).then(|| ItemStack {
                    count: k as u32,
                    ..c
                });
            }
            *drag = Drag::default();
        } else {
            *drag = Drag::default();
        }
        return;
    }
    if drag.status != 0 {
        *drag = Drag::default();
        return;
    }
    match input {
        PICKUP | QUICK_MOVE if button == 0 || button == 1 => {
            let primary = button == 0;
            if slot == -999 {
                if let Some(mut c) = v.carried.take() {
                    if primary || c.count <= 1 {
                        v.drops.push(c);
                    } else {
                        v.drops.push(ItemStack {
                            count: 1,
                            ..c.clone()
                        });
                        c.count -= 1;
                        *v.carried = Some(c);
                    }
                }
                return;
            }
            let Some(i) = index else {
                return;
            };
            if input == QUICK_MOVE {
                if !PotClick::may_pickup(i) {
                    return;
                }
                let mut moved = v.quick_move(i);
                while let Some(m) = moved {
                    if !v.at(i).is_some_and(|s| *s.item == *m.item) {
                        break;
                    }
                    moved = v.quick_move(i);
                }
                return;
            }
            let item = v.at(i).cloned();
            match (item, v.carried.clone()) {
                (None, None) => {}
                (None, Some(mut c)) => {
                    let n = if primary { c.count } else { 1 };
                    v.safe_insert(i, &mut c, n);
                    *v.carried = (c.count > 0).then_some(c);
                }
                (Some(s), carried) if PotClick::may_pickup(i) => match carried {
                    None => {
                        let n = if primary { s.count } else { s.count.div_ceil(2) };
                        if let Some(taken) = v.try_remove(i, n, u32::MAX) {
                            *v.carried = Some(taken);
                            v.on_take(i);
                        }
                    }
                    Some(mut c) if PotClick::may_place(i) => {
                        if s.same_item_same_components(&c) {
                            let n = if primary { c.count } else { 1 };
                            v.safe_insert(i, &mut c, n);
                            *v.carried = (c.count > 0).then_some(c);
                        } else if c.count <= v.slot_max(i, &c) {
                            *v.carried = Some(s);
                            *v.cell(i) = Some(c);
                        }
                    }
                    Some(mut c) if s.same_item_same_components(&c) => {
                        let room = v.item_max(&c).saturating_sub(c.count);
                        if let Some(taken) = v.try_remove(i, s.count, room) {
                            c.count += taken.count;
                            *v.carried = Some(c);
                            v.on_take(i);
                        }
                    }
                    Some(_) => {}
                },
                _ => {}
            }
        }
        SWAP if (0..9).contains(&button) || button == 40 => {
            let Some(i) = index else {
                return;
            };
            let hot = v.hot(button).clone();
            match (hot, v.at(i).cloned()) {
                (None, None) => {}
                (None, Some(s)) => {
                    if PotClick::may_pickup(i) {
                        *v.hot(button) = Some(s);
                        *v.cell(i) = None;
                        v.on_take(i);
                    }
                }
                (Some(mut h), None) => {
                    if PotClick::may_place(i) {
                        let p = v.slot_max(i, &h);
                        if h.count > p {
                            *v.cell(i) = Some(ItemStack {
                                count: p,
                                ..h.clone()
                            });
                            h.count -= p;
                            *v.hot(button) = Some(h);
                        } else {
                            *v.hot(button) = None;
                            *v.cell(i) = Some(h);
                        }
                    }
                }
                (Some(mut h), Some(s)) => {
                    if PotClick::may_pickup(i) && PotClick::may_place(i) {
                        let p = v.slot_max(i, &h);
                        if h.count > p {
                            *v.cell(i) = Some(ItemStack {
                                count: p,
                                ..h.clone()
                            });
                            h.count -= p;
                            *v.hot(button) = Some(h);
                            v.on_take(i);
                            v.add_to_inventory(s);
                        } else {
                            *v.hot(button) = Some(s);
                            *v.cell(i) = Some(h);
                            v.on_take(i);
                        }
                    }
                }
            }
        }
        CLONE => {
            if v.creative
                && v.carried.is_none()
                && let Some(s) = index.and_then(|i| v.at(i).cloned())
            {
                let max = v.item_max(&s);
                *v.carried = Some(ItemStack { count: max, ..s });
            }
        }
        THROW => {
            let Some(i) = index.filter(|_| v.carried.is_none()) else {
                return;
            };
            let n = if button == 0 {
                1
            } else {
                v.at(i).map_or(0, |s| s.count)
            };
            let mut thrown = v.safe_take(i, n, u32::MAX);
            if let Some(t) = &thrown {
                v.drops.push(t.clone());
            }
            if button == 1 {
                while let Some(t) = thrown {
                    if !v.at(i).is_some_and(|s| *s.item == *t.item) {
                        break;
                    }
                    thrown = v.safe_take(i, n, u32::MAX);
                    if let Some(t) = &thrown {
                        v.drops.push(t.clone());
                    }
                }
            }
        }
        PICKUP_ALL => {
            let (Some(i), Some(mut c)) = (index, v.carried.clone()) else {
                return;
            };
            if v.at(i).is_some() && PotClick::may_pickup(i) {
                return;
            }
            let order: Vec<usize> = if button == 0 {
                (0..POT_MENU_SLOTS).collect()
            } else {
                (0..POT_MENU_SLOTS).rev().collect()
            };
            let max = v.item_max(&c);
            for pass in 0..2 {
                for &r in &order {
                    if c.count >= max {
                        break;
                    }
                    let Some(s) = v.at(r).cloned() else {
                        continue;
                    };
                    if !v.quick_replace(r, &c) || !PotClick::may_pickup(r) {
                        continue;
                    }
                    if pass == 0 && s.count == v.item_max(&s) {
                        continue;
                    }
                    if let Some(t) = v.safe_take(r, s.count, max - c.count) {
                        c.count += t.count;
                    }
                }
            }
            *v.carried = Some(c);
        }
        _ => {}
    }
}

// ------------------------------------------------- cabinets and baskets

/// `CabinetBlock` (`CabinetBlockEntity`, 27 slots, like a barrel).
fn is_cabinet(name: &str) -> bool {
    name.starts_with("farmersdelight:") && name.ends_with("_cabinet")
}
/// `BasketBlock` (`BasketBlockEntity`, 27 slots).
fn is_basket(name: &str) -> bool {
    matches!(name, "farmersdelight:wooden_basket" | "farmersdelight:bamboo_basket")
}
impl Game<'_> {
    /// `CabinetBlock.useWithoutItem` / `BasketBlock.useWithoutItem`:
    /// `player.openMenu` (a cabinet angers the piglins around).
    fn fd_container_use(&mut self, pos: [i32; 3], cabinet: bool) -> Option<bool> {
        // useWithoutItem is the main hand's turn only.
        if self.hand != Hand::Main {
            return None;
        }
        // Game::open_menu's container branch, with the block entity's 27
        // slots (ChestMenu.threeRows).
        let key = (pos[0], pos[1], pos[2]);
        self.world
            .block_entities
            .get_or_insert_with(key, || horizon_client::containers::BlockEntity::Chest(vec![None; 27]));
        self.container = Some(key);
        let first = self.container_opener_event(key, true);
        self.post_block_game_event("minecraft:container_open", pos, Some(Who::Player), None);
        if cabinet {
            self.anger_piglins(true);
        }
        if first {
            self.container_edge_sound(key, true);
        }
        Some(true)
    }
    /// `CabinetBlockEntity.openersCounter`: the first opener opens the
    /// cabinet (`open=true`), the last to leave closes it. None for other
    /// blocks; a basket has no openers.
    pub(super) fn fd_container_openers(&mut self, key: (i32, i32, i32), open: bool) -> Option<bool> {
        let name = self.fd_name([key.0, key.1, key.2]);
        if is_basket(&name) {
            return Some(false);
        }
        if !is_cabinet(&name) {
            return None;
        }
        let count = self.lid_openers.entry(key).or_default();
        let before = *count;
        *count = (before + if open { 1 } else { -1 }).max(0);
        let after = *count;
        if after == 0 {
            self.lid_openers.remove(&key);
        }
        let edge = (before == 0) != (after == 0);
        if edge {
            let pos = [key.0, key.1, key.2];
            let state = self.world.block(pos[0], pos[1], pos[2]);
            let value = if after > 0 { "true" } else { "false" };
            if let Some(next) = self.with_props(state, &[("open", value)]) {
                let _ = self.level_set(pos, next);
            }
        }
        Some(edge)
    }
    /// `CabinetBlockEntity.playSound`: the cabinet's open or close sound at
    /// its front face. True when the block was a cabinet or a basket.
    pub(super) fn fd_container_edge_sound(&mut self, key: (i32, i32, i32), open: bool) -> bool {
        let pos = [key.0, key.1, key.2];
        let name = self.fd_name(pos);
        if is_basket(&name) {
            return true;
        }
        if !is_cabinet(&name) {
            return false;
        }
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let facing = self.fd_props(state).remove("facing").unwrap_or_else(|| "north".into());
        let [dx, dz] = step(&facing);
        let at = [
            f64::from(pos[0]) + 0.5 + dx / 2.0,
            f64::from(pos[1]) + 0.5,
            f64::from(pos[2]) + 0.5 + dz / 2.0,
        ];
        let sound = if open {
            "farmersdelight:block.cabinet.open"
        } else {
            "farmersdelight:block.cabinet.close"
        };
        let pitch = self.world.random.next_float() * 0.1 + 0.9;
        self.audio.play(sound, Some(at), 0.5, pitch);
        true
    }
}

// ------------------------------------------------------------ rope fence

const ROPE_FENCE: &str = "farmersdelight:rope_fence";
const ROPE_FENCE_GATE: &str = "farmersdelight:rope_fence_gate";

impl Game<'_> {
    /// `RopeFenceBlock.connectsTo` for the side of the rope fence at `pos`
    /// facing `d`: a sturdy face that is no exception, another rope fence, or
    /// a rope fence gate turned across.
    fn fd_rope_connects(&self, pos: [i32; 3], d: horizon_core::geometry::Direction) -> bool {
        use horizon_core::geometry::Direction as D;
        let (step, back, back_cw_x) = match d {
            D::North => ([0, -1], D::South, true),
            D::South => ([0, 1], D::North, true),
            D::East => ([1, 0], D::West, false),
            _ => ([-1, 0], D::East, false),
        };
        let q = [pos[0] + step[0], pos[1], pos[2] + step[1]];
        let state = self.world.block(q[0], q[1], q[2]);
        let name = self.fd_name(q);
        if name == ROPE_FENCE {
            return true;
        }
        if name == ROPE_FENCE_GATE {
            // FenceGateBlock.connectsToDirection(state, back): the gate's
            // facing on the axis of back's clockwise turn (X for north and
            // south, Z for east and west).
            let facing = self.fd_props(state).remove("facing").unwrap_or_default();
            let gate_x = matches!(facing.as_str(), "east" | "west");
            return gate_x == back_cw_x;
        }
        // CrossCollisionBlock.isExceptionForConnection.
        let exception = name.ends_with("_leaves")
            || name.ends_with("shulker_box")
            || matches!(
                name.as_str(),
                "minecraft:barrier"
                    | "minecraft:carved_pumpkin"
                    | "minecraft:jack_o_lantern"
                    | "minecraft:melon"
                    | "minecraft:pumpkin"
            );
        !exception && self.res.behaviours().is_face_sturdy(state, back)
    }
    /// `process_level_events`' HEAD: a block action's rope fence updates are
    /// made in the same action (on the window, its prediction of them).
    pub(super) fn fd_rope_now(&mut self) {
        if self.native_on("farmersdelight") {
            self.fd_rope_tick();
        }
    }
    /// The window's prediction of the rope fences beside a block it places:
    /// their state with `placed` at `pos` (`updateShape` on the client).
    pub(super) fn fd_rope_predict(
        &self,
        pos: [i32; 3],
        placed: BlockStateId,
    ) -> Vec<([i32; 3], BlockStateId)> {
        use horizon_core::geometry::Direction as D;
        let mut out = Vec::new();
        // The fence at q and the side of it facing the placed block.
        for (step, side) in [([0, -1], D::South), ([1, 0], D::West), ([0, 1], D::North), ([-1, 0], D::East)] {
            let q = [pos[0] + step[0], pos[1], pos[2] + step[1]];
            if self.fd_name(q) != ROPE_FENCE {
                continue;
            }
            let state = self.world.block(q[0], q[1], q[2]);
            let name = &*self.res.facts().get(placed).name;
            let (back, back_cw_x) = match side {
                D::North => (D::South, true),
                D::South => (D::North, true),
                D::East => (D::West, false),
                _ => (D::East, false),
            };
            let connects = if name == ROPE_FENCE {
                true
            } else if name == ROPE_FENCE_GATE {
                let facing = self.fd_props(placed).remove("facing").unwrap_or_default();
                matches!(facing.as_str(), "east" | "west") == back_cw_x
            } else {
                let exception = name.ends_with("_leaves")
                    || name.ends_with("shulker_box")
                    || matches!(
                        name,
                        "minecraft:barrier"
                            | "minecraft:carved_pumpkin"
                            | "minecraft:jack_o_lantern"
                            | "minecraft:melon"
                            | "minecraft:pumpkin"
                    );
                !exception && self.res.behaviours().is_face_sturdy(placed, back)
            };
            let key = match side {
                D::North => "north",
                D::South => "south",
                D::East => "east",
                _ => "west",
            };
            if let Some(next) = self.with_props(state, &[(key, if connects { "true" } else { "false" })])
                && next != state
            {
                out.push((q, next));
            }
        }
        out
    }
    /// `RopeFenceBlock.updateShape` for the rope fences at and beside the
    /// blocks that changed.
    fn fd_rope_tick(&mut self) {
        use horizon_client::world::farmersdelight_ropes as ropes;
        use horizon_core::geometry::Direction as D;
        // The rope fence's and gate's states, for the writes to look up.
        if ropes::FENCES.read().is_ok_and(|f| f.is_empty()) {
            let of = |block: &str| -> Vec<u32> {
                self.res
                    .facts()
                    .states
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| &*s.name == block)
                    .map(|(i, _)| i as u32)
                    .collect()
            };
            let (fences, gates) = (of(ROPE_FENCE), of(ROPE_FENCE_GATE));
            let (rice, panicles) = (of(RICE), of(RICE_PANICLES));
            let mut tatami = [of(TATAMI), of(FULL_MAT), of(HALF_MAT)].concat();
            tatami.sort_unstable();
            if let (Ok(mut f), Ok(mut g), Ok(mut r), Ok(mut p), Ok(mut t)) = (
                ropes::FENCES.write(),
                ropes::GATES.write(),
                ropes::RICE.write(),
                ropes::PANICLES.write(),
                ropes::TATAMI.write(),
            ) {
                *f = fences;
                *g = gates;
                *r = rice;
                *p = panicles;
                *t = tatami;
            }
        }
        // This side's own writes. The authoritative side's rope fences, rice
        // and tatami answer their neighbours through the update_shape hook
        // point; the window predicts its rope fences beside the blocks it
        // changes (a broken block's neighbours let go at once).
        let me = &self.world as *const _ as usize;
        let dirty: Vec<[i32; 3]> = ropes::DIRTY
            .lock()
            .map(|mut d| {
                let (mine, rest): (Vec<_>, Vec<_>) = d.drain(..).partition(|(w, _)| *w == me);
                *d = rest;
                mine.into_iter().map(|(_, p)| p).collect()
            })
            .unwrap_or_default();
        if self.remote.is_none() {
            return;
        }
        let mut seen = std::collections::HashSet::new();
        for &p in &dirty {
            for d in [[0, 0], [0, -1], [1, 0], [0, 1], [-1, 0]] {
                let q = [p[0] + d[0], p[1], p[2] + d[1]];
                if !seen.insert(q) || self.fd_name(q) != ROPE_FENCE {
                    continue;
                }
                let state = self.world.block(q[0], q[1], q[2]);
                let c = |d| if self.fd_rope_connects(q, d) { "true" } else { "false" };
                let props = [
                    ("north", c(D::North)),
                    ("east", c(D::East)),
                    ("south", c(D::South)),
                    ("west", c(D::West)),
                ];
                if let Some(next) = self.with_props(state, &props)
                    && next != state
                {
                    let _ = self.world.set(q[0], q[1], q[2], next);
                }
            }
        }
    }
}

/// The creative screen is being drawn and its page buttons are not yet.
static CREATIVE_PAGES_DUE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// `draw_creative`'s HEAD: this frame's creative screen will draw its page buttons.
pub(super) fn creative_pages_arm() {
    CREATIVE_PAGES_DUE.store(true, Ordering::Relaxed);
}

/// The creative screen's page buttons at the screen origin: (x, y, step).
fn creative_page_buttons(ox: f32, oy: f32) -> [(f32, f32, i32); 2] {
    let right = ox + horizon_client::creative::Creative::WIDTH - 20.0;
    [(ox, oy - 50.0, -1), (right, oy - 50.0, 1)]
}

/// A `ContainerClick`'s claim about a stack against the server's.
fn hashed_matches(
    claim: &Option<horizon_protocol::item::Hashed>,
    net: &Option<horizon_protocol::item::Stack>,
) -> bool {
    match (claim, net) {
        (None, None) => true,
        (Some(h), Some(s)) => h.item == s.item && h.count == s.count,
        _ => false,
    }
}

impl Game<'_> {
    /// `getContainer`: the meal's container, else the meal's remainder.
    fn fd_pot_container(&self, pot: &Pot) -> Option<String> {
        let meal = pot.items[MEAL_SLOT].as_ref()?;
        match &pot.container {
            Some(c) => Some(c.item.to_string()),
            None => self.fd_crafting_remainder(&meal.item),
        }
    }
    /// The menu's slots: the pot's, then the inventory's and the hotbar's.
    fn fd_pot_menu_items(&self, pos: [i32; 3], inventory: &[Option<ItemStack>]) -> Vec<Option<ItemStack>> {
        let mut items = self.fd_pot(pos).items.to_vec();
        items.extend((9..36).chain(0..9).map(|i| inventory.get(i).cloned().flatten()));
        items
    }

    /// The server's pot menus each tick (`ServerPlayer.tick`): the menus
    /// players' uses opened, then `stillValid` and `broadcastChanges`.
    pub(super) fn fd_pot_menus_tick(&mut self, host: &mut super::lan::LanHost) {
        let opens: Vec<(u32, [i32; 3], &'static str)> = POT_OPENS
            .lock()
            .map(|mut o| std::mem::take(&mut *o))
            .unwrap_or_default();
        for (conn, pos, dim) in opens {
            if dim == self.dim.id() && self.fd_name(pos) == POT {
                self.fd_pot_open(host, conn, pos);
            }
        }
        let conns: Vec<u32> = POT_MENUS
            .lock()
            .map(|m| m.keys().copied().collect())
            .unwrap_or_default();
        for conn in conns {
            let Some(mut menu) = POT_MENUS.lock().ok().and_then(|mut m| m.remove(&conn)) else {
                continue;
            };
            let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
                continue;
            };
            let g = &host.hub.guests[gi];
            if !host.hub.here(g) {
                if let Ok(mut m) = POT_MENUS.lock() {
                    m.insert(conn, menu);
                }
                continue;
            }
            // CookingPotMenu.stillValid: the pot, within the interaction range.
            let valid = !g.dead
                && menu.dim == self.dim.id()
                && self.fd_name(menu.pos) == POT
                && g.within_block_range(menu.pos, 4.0);
            if !valid {
                // closeContainer: the client is told, and `removed` puts the
                // carried stack back.
                host.hub.send_to(
                    conn,
                    &horizon_protocol::packets::play::Clientbound::ContainerClose(
                        horizon_protocol::packets::play::ContainerClose {
                            container: menu.window,
                        },
                    ),
                );
                self.guest_carried_back(host, gi);
                continue;
            }
            self.fd_pot_broadcast(host, gi, &mut menu);
            if let Ok(mut m) = POT_MENUS.lock() {
                m.insert(conn, menu);
            }
        }
    }
    /// `ServerPlayer.openMenu` with the pot's `MenuProvider`: the open menu
    /// closes, the client opens the screen with the pot's name, and
    /// `initMenu` sends everything.
    fn fd_pot_open(&mut self, host: &mut super::lan::LanHost, conn: u32, pos: [i32; 3]) {
        self.lan_station_closed(host, conn, None);
        self.lan_crafting_closed(host, conn, None);
        self.guest_container_closed(host, conn);
        let pot = self.fd_pot(pos);
        // getDisplayName: the custom name, else `container.farmersdelight.cooking_pot`.
        let title = match pot
            .custom_name
            .as_deref()
            .and_then(|j| serde_json::from_str::<serde_json::Value>(j).ok())
        {
            Some(json) => horizon_client::lan::data::component_nbt(&json),
            None => horizon_protocol::text::translatable("container.farmersdelight.cooking_pot", &[]),
        };
        let Some(window) = host.hub.open_window(conn, POT_MENU, title) else {
            return;
        };
        let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
            return;
        };
        let mut menu = PotMenu {
            window,
            pos,
            dim: self.dim.id(),
            state: 0,
            sent: Vec::new(),
            sent_carried: None,
            sent_data: [i32::MIN; 2],
            drag: Drag::default(),
        };
        self.fd_pot_send_all(host, gi, &mut menu);
        if let Ok(mut m) = POT_MENUS.lock() {
            m.insert(conn, menu);
        }
    }
    /// `sendAllDataToRemote`: every slot, the carried stack, the data slots.
    fn fd_pot_send_all(&mut self, host: &mut super::lan::LanHost, gi: usize, menu: &mut PotMenu) {
        use horizon_protocol::packets::play::{self, Clientbound as C};
        let items = self.fd_pot_menu_items(menu.pos, &host.hub.guests[gi].inventory);
        let net: Vec<_> = items.iter().map(|s| host.hub.stack(s)).collect();
        let carried = host.hub.guests[gi].carried.clone();
        let carried_net = host.hub.stack(&carried);
        menu.state = (menu.state + 1) & 0x7FFF;
        let conn = host.hub.guests[gi].conn;
        host.hub.send_to(
            conn,
            &C::ContainerSetContent(play::ContainerSetContent {
                container: menu.window,
                state: menu.state,
                items: net,
                carried: carried_net,
            }),
        );
        menu.sent = items;
        menu.sent_carried = carried;
        menu.sent_data = [i32::MIN; 2];
        self.fd_pot_send_data(host, conn, menu);
    }
    /// The data slots that changed (`CookingPotBlockEntity.cookingPotData`:
    /// the cook time and its total), and the pot's container for the screen.
    fn fd_pot_send_data(&self, host: &super::lan::LanHost, conn: u32, menu: &mut PotMenu) {
        use horizon_protocol::packets::play::{self, Clientbound as C};
        let pot = self.fd_pot(menu.pos);
        for (k, value) in [pot.cook_time, pot.cook_time_total].into_iter().enumerate() {
            if menu.sent_data[k] != value {
                menu.sent_data[k] = value;
                host.hub.send_to(
                    conn,
                    &C::ContainerSetData(play::ContainerSetData {
                        container: menu.window,
                        id: k as i16,
                        value: value as i16,
                    }),
                );
            }
        }
        let key = (menu.pos[0], menu.pos[1], menu.pos[2], menu.dim);
        if let Ok(mut c) = POT_CONTAINERS.lock() {
            c.insert(key, self.fd_pot_container(&pot));
        }
    }
    /// `broadcastChanges`: the slots and carried stack the client was not
    /// told of, then the data slots.
    fn fd_pot_broadcast(&mut self, host: &mut super::lan::LanHost, gi: usize, menu: &mut PotMenu) {
        use horizon_protocol::packets::play::{self, Clientbound as C};
        let conn = host.hub.guests[gi].conn;
        let items = self.fd_pot_menu_items(menu.pos, &host.hub.guests[gi].inventory);
        for (slot, now) in items.iter().enumerate() {
            if menu.sent.get(slot) == Some(now) {
                continue;
            }
            let item = host.hub.stack(now);
            menu.state = (menu.state + 1) & 0x7FFF;
            host.hub.send_to(
                conn,
                &C::ContainerSetSlot(play::ContainerSetSlot {
                    container: menu.window,
                    state: menu.state,
                    slot: slot as i16,
                    item,
                }),
            );
        }
        menu.sent = items;
        let carried = host.hub.guests[gi].carried.clone();
        if carried != menu.sent_carried {
            let item = host.hub.stack(&carried);
            host.hub
                .send_to(conn, &C::SetCursorItem(play::SetCursorItem { item }));
            menu.sent_carried = carried;
        }
        self.fd_pot_send_data(host, conn, menu);
    }
    /// `handleContainerClick` on the player's pot menu; false for other windows.
    pub(super) fn fd_pot_menu_click(
        &mut self,
        host: &mut super::lan::LanHost,
        conn: u32,
        p: &horizon_protocol::packets::play::ContainerClick,
    ) -> bool {
        let Some(mut menu) = POT_MENUS.lock().ok().and_then(|mut m| {
            if m.get(&conn).is_some_and(|x| x.window == p.container) {
                m.remove(&conn)
            } else {
                None
            }
        }) else {
            return false;
        };
        self.fd_pot_clicked(host, conn, p, &mut menu);
        if let Ok(mut m) = POT_MENUS.lock() {
            m.insert(conn, menu);
        }
        true
    }
    fn fd_pot_clicked(
        &mut self,
        host: &mut super::lan::LanHost,
        conn: u32,
        p: &horizon_protocol::packets::play::ContainerClick,
        menu: &mut PotMenu,
    ) {
        use horizon_protocol::packets::play::{self, Clientbound as C};
        let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn) else {
            return;
        };
        if host.hub.guests[gi].spectator() {
            // A spectator's click changes nothing; its client is told again.
            self.fd_pot_send_all(host, gi, menu);
            return;
        }
        let g = &host.hub.guests[gi];
        if menu.dim != self.dim.id()
            || self.fd_name(menu.pos) != POT
            || !g.within_block_range(menu.pos, 4.0)
        {
            return;
        }
        // isValidSlotIndex.
        if !(p.slot == -999 || p.slot == -1 || (0..POT_MENU_SLOTS as i16).contains(&p.slot)) {
            return;
        }
        let stale = p.state != menu.state;
        let pos = menu.pos;
        let mut pot = self.fd_pot(pos);
        let mut items = pot.items.clone();
        let (took, drops) = {
            let max = |item: &str| self.res.items().get(item).map_or(64, |i| i.max_stack);
            let serving =
                |item: &str| self.fd_item_in_tag("farmersdelight:serving_containers", item);
            let container = self.fd_pot_container(&pot);
            let g = &mut host.hub.guests[gi];
            let creative = g.creative();
            let mut v = PotClick {
                pot: &mut items,
                inventory: &mut g.inventory,
                offhand: &mut g.offhand,
                carried: &mut g.carried,
                creative,
                container,
                max: &max,
                serving: &serving,
                took_output: false,
                drops: Vec::new(),
            };
            pot_click(
                &mut v,
                &mut menu.drag,
                i32::from(p.slot),
                i32::from(p.button),
                p.input,
            );
            (v.took_output, std::mem::take(&mut v.drops))
        };
        let mut changed = items != pot.items;
        pot.items = items;
        // CookingPotResultSlot.checkTakeAchievements: awardUsedRecipes, the
        // recipes' experience popped at the player.
        if took && !pot.used.is_empty() {
            let xp = pot_xp(&pot.used);
            pot.used.clear();
            changed = true;
            if xp > 0 {
                let at = host.hub.guests[gi].position;
                self.spawn_orbs(at, xp);
            }
        }
        if changed {
            self.fd_put_pot(pos, &pot, true);
        }
        if !drops.is_empty() {
            self.drop_guest_items(host, gi, drops);
        }
        if stale {
            // broadcastFullState.
            self.fd_pot_send_all(host, gi, menu);
            return;
        }
        // setRemoteSlotUnsafe for what the client claims, then broadcastChanges.
        let claimed: HashMap<usize, Option<horizon_protocol::item::Hashed>> = p
            .changed
            .iter()
            .map(|(s, h)| (*s as usize, h.clone()))
            .collect();
        let actual = self.fd_pot_menu_items(pos, &host.hub.guests[gi].inventory);
        for (slot, now) in actual.iter().enumerate() {
            let net = host.hub.stack(now);
            let agreed = match claimed.get(&slot) {
                Some(claim) => hashed_matches(claim, &net),
                None => menu.sent.get(slot) == Some(now),
            };
            if !agreed {
                menu.state = (menu.state + 1) & 0x7FFF;
                host.hub.send_to(
                    conn,
                    &C::ContainerSetSlot(play::ContainerSetSlot {
                        container: menu.window,
                        state: menu.state,
                        slot: slot as i16,
                        item: net,
                    }),
                );
            }
        }
        menu.sent = actual;
        let carried = host.hub.guests[gi].carried.clone();
        let net = host.hub.stack(&carried);
        if !hashed_matches(&p.carried, &net) {
            host.hub
                .send_to(conn, &C::SetCursorItem(play::SetCursorItem { item: net }));
        }
        menu.sent_carried = carried;
        self.fd_pot_send_data(host, conn, menu);
    }
    /// `AbstractContainerMenu.removed` of the player's pot menu (`window`;
    /// None: whichever is open, as another menu opens): the carried stack
    /// goes back. False when no such menu is open.
    pub(super) fn fd_pot_menu_closed(
        &mut self,
        host: &mut super::lan::LanHost,
        conn: u32,
        window: Option<i32>,
    ) -> bool {
        let removed = POT_MENUS.lock().ok().and_then(|mut m| {
            if m.get(&conn)
                .is_some_and(|x| window.is_none_or(|w| x.window == w))
            {
                m.remove(&conn)
            } else {
                None
            }
        });
        if removed.is_none() {
            return false;
        }
        if window.is_none()
            && let Some(gi) = host.hub.guests.iter().position(|g| g.conn == conn)
        {
            self.guest_carried_back(host, gi);
        }
        true
    }

    // ---- the window's screen

    /// The window shows a server's cooking pot menu (`CookingPotScreen`).
    pub(super) fn fd_pot_screen(&self) -> bool {
        self.container.is_some() && self.remote_menu_type() == Some(POT_MENU)
    }
    /// The screen opens: its block (the menu's extra data, here the clicked
    /// block), fresh data slots and no drag.
    pub(super) fn fd_pot_screen_opened(&mut self, target: Option<[i32; 3]>) {
        if let Ok(mut c) = CLIENT_POT.lock() {
            *c = ClientPot {
                pos: target.unwrap_or_default(),
                ..ClientPot::default()
            };
        }
    }
    /// `AbstractContainerMenu.setData` from the server.
    pub(super) fn fd_pot_screen_data(&mut self, id: usize, value: i16) {
        if let Ok(mut c) = CLIENT_POT.lock()
            && id < 2
        {
            c.data[id] = i32::from(value);
        }
    }
    fn fd_pot_screen_pos(&self) -> [i32; 3] {
        CLIENT_POT.lock().map(|c| c.pos).unwrap_or_default()
    }
    /// The screen's pot's `getContainer` item.
    pub(super) fn fd_pot_screen_container(&self) -> Option<String> {
        let [x, y, z] = self.fd_pot_screen_pos();
        POT_CONTAINERS
            .lock()
            .ok()
            .and_then(|c| c.get(&(x, y, z, self.dim.id())).cloned())
            .flatten()
    }
    /// `CookingPotMenu`'s slot positions, by the screen's slot ids.
    pub(super) fn fd_pot_slot_position(&self, i: usize) -> Option<(f32, f32)> {
        if !self.fd_pot_screen() {
            return None;
        }
        Some(match i {
            0..9 => (8.0 + i as f32 * 18.0, 142.0),
            9..36 => (
                8.0 + ((i - 9) % 9) as f32 * 18.0,
                84.0 + ((i - 9) / 9) as f32 * 18.0,
            ),
            300..306 => {
                let k = i - 300;
                (30.0 + (k % 3) as f32 * 18.0, 17.0 + (k / 3) as f32 * 18.0)
            }
            306 => (124.0, 26.0),
            307 => (92.0, 55.0),
            308 => (124.0, 55.0),
            _ => return None,
        })
    }
    /// `CookingPotScreen.isHovering(HEAT_ICON)`, the screen at `origin`.
    pub(super) fn fd_pot_over_heat_icon(&self, origin: (f32, f32)) -> bool {
        let (mx, my) = (self.cursor.0 - origin.0, self.cursor.1 - origin.1);
        let [x, y, w, h] = HEAT_ICON;
        mx >= x - 1.0 && mx < x + w + 1.0 && my >= y - 1.0 && my < y + h + 1.0
    }
    /// The heat tooltip's key (`TextUtils.container("cooking_pot.heated")`).
    pub(super) fn fd_pot_heat_key(&self) -> &'static str {
        if self.fd_pot_heated(self.fd_pot_screen_pos()) {
            "container.farmersdelight.cooking_pot.heated"
        } else {
            "container.farmersdelight.cooking_pot.not_heated"
        }
    }
    /// `CookingPotScreen.renderBg` and `renderLabels`: the background, the
    /// heat icon, the progress arrow, the container slot's empty icon
    /// (`getNoItemIcon`), the title and the inventory's.
    pub(super) fn fd_pot_draw(
        &self,
        batch: &mut horizon_client::hud::HudBatch,
        ox: f32,
        oy: f32,
    ) -> bool {
        if !self.fd_pot_screen() {
            return false;
        }
        let atlas = &self.gui.atlas;
        let blit = |batch: &mut horizon_client::hud::HudBatch,
                    name: &str,
                    [u, v, w, h]: [f32; 4],
                    x: f32,
                    y: f32| {
            if let Some(mut r) = atlas.sprites.get(name).copied() {
                let (du, dv) = ((r.u1 - r.u0) / r.w as f32, (r.v1 - r.v0) / r.h as f32);
                r.u0 += u * du;
                r.v0 += v * dv;
                r.u1 = r.u0 + w * du;
                r.v1 = r.v0 + h * dv;
                batch.rect_uv(x, y, w, h, r, u32::MAX);
            }
        };
        blit(batch, POT_TEXTURE, [0.0, 0.0, 176.0, 166.0], ox, oy);
        let (pos, [time, total]) = CLIENT_POT
            .lock()
            .map(|c| (c.pos, c.data))
            .unwrap_or_default();
        if self.fd_pot_heated(pos) {
            let [x, y, w, h] = HEAT_ICON;
            blit(batch, POT_TEXTURE, [176.0, 0.0, w, h], ox + x, oy + y);
        }
        // getCookProgressionScaled, drawn one wider.
        let l = if total != 0 && time != 0 { time * 24 / total } else { 0 };
        blit(
            batch,
            POT_TEXTURE,
            [176.0, 15.0, (l + 1) as f32, 17.0],
            ox + 89.0,
            oy + 25.0,
        );
        let container_empty = self
            .container
            .and_then(|k| self.world.block_entities.get(&k))
            .is_none_or(|e| e.slots().get(CONTAINER_SLOT).is_none_or(Option::is_none));
        if container_empty {
            blit(batch, BOWL_ICON, [0.0, 0.0, 16.0, 16.0], ox + 92.0, oy + 55.0);
        }
        let grey = horizon_client::hud::rgba(64, 64, 64, 255);
        let title = self.remote_menu_title().unwrap_or_else(|| {
            Text::literal(self.translate("container.farmersdelight.cooking_pot", "Cooking Pot"))
        });
        batch.text_styled(atlas, &title, ox + 28.0, oy + 6.0, grey, false);
        let inventory = self.translate("container.inventory", "Inventory");
        batch.text(atlas, &inventory, ox + 8.0, oy + 72.0, grey, false);
        true
    }
    /// Runs `f` on the window's copy of the open pot menu (its slots, the
    /// inventory, the carried stack, the drag) and keeps what it did.
    fn fd_pot_local<R>(&mut self, f: impl FnOnce(&mut PotClick, &mut Drag) -> R) -> Option<R> {
        let key = self.container?;
        let mut pot: [Option<ItemStack>; POT_SLOTS] = Default::default();
        if let Some(e) = self.world.block_entities.get(&key) {
            for (i, s) in e.slots().iter().take(POT_SLOTS).enumerate() {
                pot[i] = s.clone();
            }
        }
        let mut drag = CLIENT_POT.lock().map(|c| c.drag.clone()).unwrap_or_default();
        let mut inventory = self.inventory.clone();
        let mut offhand = self.offhand.clone();
        let mut carried = self.carried.clone();
        let result = {
            let max = |item: &str| self.res.items().get(item).map_or(64, |i| i.max_stack);
            let serving =
                |item: &str| self.fd_item_in_tag("farmersdelight:serving_containers", item);
            let mut v = PotClick {
                pot: &mut pot,
                inventory: &mut inventory,
                offhand: &mut offhand,
                carried: &mut carried,
                creative: self.creative,
                container: self.fd_pot_screen_container(),
                max: &max,
                serving: &serving,
                took_output: false,
                drops: Vec::new(),
            };
            f(&mut v, &mut drag)
        };
        if let Some(e) = self.world.block_entities.get_mut(&key) {
            for (cell, s) in e.slots_mut().iter_mut().zip(pot) {
                *cell = s;
            }
        }
        self.inventory = inventory;
        self.offhand = offhand;
        self.carried = carried;
        if let Ok(mut c) = CLIENT_POT.lock() {
            c.drag = drag;
        }
        Some(result)
    }
    /// `MultiPlayerGameMode.handleContainerInput` for each click (menu slot,
    /// button, input) on the open pot menu: the window's own `doClick`, then
    /// the menu slots it changed and the carried stack after it; true when
    /// one threw a stack (the client swings).
    pub(super) fn fd_pot_predict(
        &mut self,
        clicks: &[(i16, i32, i32)],
    ) -> (Vec<(Vec<(i16, Option<ItemStack>)>, Option<ItemStack>)>, bool) {
        self.fd_pot_local(|v, drag| {
            let mut out = Vec::new();
            let mut threw = false;
            for &(slot, button, input) in clicks {
                let before = v.all();
                pot_click(v, drag, i32::from(slot), button, input);
                let after = v.all();
                threw |= !v.drops.is_empty();
                v.drops.clear();
                let changed = (0..POT_MENU_SLOTS)
                    .filter(|&i| before[i] != after[i])
                    .map(|i| (i as i16, after[i].clone()))
                    .collect();
                out.push((changed, v.carried.clone()));
            }
            (out, threw)
        })
        .unwrap_or_default()
    }
    /// `AbstractContainerScreen.mouseReleased`'s Shift double click: a
    /// QUICK_MOVE of every slot on the clicked one's side (the pot's or the
    /// player's) that `canItemQuickReplace` the moved stack, each predicted in
    /// turn: (menu slot, changed slots, carried) for each.
    pub(super) fn fd_pot_quick_move_all(
        &mut self,
        clicked: usize,
        moved: &ItemStack,
    ) -> Vec<(i16, Vec<(i16, Option<ItemStack>)>, Option<ItemStack>)> {
        let pot_side = clicked < POT_SLOTS;
        self.fd_pot_local(|v, drag| {
            let mut out = Vec::new();
            for i in 0..POT_MENU_SLOTS {
                if (i < POT_SLOTS) != pot_side
                    || !PotClick::may_pickup(i)
                    || v.at(i).is_none()
                    || !v.quick_replace(i, moved)
                {
                    continue;
                }
                let before = v.all();
                pot_click(v, drag, i as i32, 0, QUICK_MOVE);
                let after = v.all();
                v.drops.clear();
                let changed = (0..POT_MENU_SLOTS)
                    .filter(|&k| before[k] != after[k])
                    .map(|k| (k as i16, after[k].clone()))
                    .collect();
                out.push((i as i16, changed, v.carried.clone()));
            }
            out
        })
        .unwrap_or_default()
    }
}

// ------------------------------------------------------------- skillet
//
// `SkilletBlock`, `SkilletBlockEntity`, `SkilletRenderer` and `SkilletItem`:
// a campfire recipe's food cooks in the placed skillet on a heat source and
// jumps out of its side; held, the skillet cooks the other hand's food while
// the use button is held near heat, and hits like an iron weapon.

const SKILLET: &str = "farmersdelight:skillet";
/// `SkilletBlock.MINIMUM_COOKING_TIME`.
const SKILLET_MIN_COOKING_TIME: i32 = 60;
/// The held skillet's cooking (`ModDataComponents.SKILLET_INGREDIENT` and
/// `COOKING_TIME_LENGTH`), kept under the stack's custom data.
const SKILLET_INGREDIENT: &str = "farmersdelight:skillet_ingredient";
const COOKING_TIME_LENGTH: &str = "farmersdelight:cooking_time_length";

/// What a placed skillet holds (`SkilletBlockEntity`).
#[derive(Clone, Default)]
struct Skillet {
    stack: Option<ItemStack>,
    cook_time: i32,
    cook_time_total: i32,
    /// `skilletStack`: the item it was placed from.
    skillet: Option<ItemStack>,
}
impl Skillet {
    /// `fireAspectLevel`.
    fn fire_aspect(&self) -> u32 {
        self.skillet
            .as_ref()
            .map_or(0, |s| s.enchantment_level("minecraft:fire_aspect"))
    }
}
fn skillet_from_tag(tag: &Tag) -> Skillet {
    let int = |k: &str| match tag.get(k) {
        Some(Tag::Int(v)) => *v,
        _ => 0,
    };
    let stack = match tag.get("Inventory").and_then(|i| i.get("Items")) {
        Some(Tag::List(list)) => list
            .iter()
            .find(|t| t.get("Slot") == Some(&Tag::Int(0)))
            .and_then(save::item_from_nbt),
        _ => None,
    };
    Skillet {
        stack,
        cook_time: int("CookTime"),
        cook_time_total: int("CookTimeTotal"),
        skillet: tag.get("Skillet").and_then(save::item_from_nbt),
    }
}
/// `SkilletBlockEntity.saveAdditional`.
fn skillet_tag(pos: [i32; 3], s: &Skillet) -> Tag {
    let items: Vec<Tag> = s
        .stack
        .iter()
        .map(|stack| {
            let mut item = save::item_to_nbt(stack);
            if let Tag::Compound(map) = &mut item {
                map.insert("Slot".into(), Tag::Int(0));
            }
            item
        })
        .collect();
    let mut tag = Tag::compound([
        ("id", Tag::String(SKILLET.into())),
        ("x", Tag::Int(pos[0])),
        ("y", Tag::Int(pos[1])),
        ("z", Tag::Int(pos[2])),
        (
            "Inventory",
            Tag::compound([("Items", Tag::List(items)), ("Size", Tag::Int(1))]),
        ),
        ("CookTime", Tag::Int(s.cook_time)),
        ("CookTimeTotal", Tag::Int(s.cook_time_total)),
    ]);
    if let (Some(item), Tag::Compound(map)) = (&s.skillet, &mut tag) {
        map.insert("Skillet".into(), save::item_to_nbt(item));
    }
    tag
}
/// What a removed skillet drops (`SkilletBlock.onRemove`): its food.
pub(super) fn skillet_food(tag: &Tag) -> Option<ItemStack> {
    skillet_from_tag(tag).stack
}
/// `SkilletBlock.getSkilletCookingTime`: a fifth of the recipe's seconds,
/// a twentieth less per Fire Aspect level, whole seconds, three at least.
fn skillet_cooking_time(original: i32, fire_aspect: u32) -> i32 {
    let cooking_time = if original > 0 { original } else { 600 };
    let seconds = cooking_time / 20;
    let mut reduction = 0.2f32;
    if fire_aspect > 0 {
        reduction = (f64::from(reduction) - f64::from(fire_aspect) * 0.05) as f32;
    }
    let result = (seconds as f32 * reduction) as i32 * 20;
    // Mth.clamp(value, min, max): the max wins over the min.
    result.max(SKILLET_MIN_COOKING_TIME).min(original)
}
/// The held skillet's ingredient and its recipe's cooking time.
fn skillet_cooking(stack: &ItemStack) -> Option<(ItemStack, i32)> {
    let data = stack.data()?.other("minecraft:custom_data")?;
    let ingredient = save::item_from_nbt(data.get(SKILLET_INGREDIENT)?)?;
    let length = match data.get(COOKING_TIME_LENGTH) {
        Some(Tag::Int(v)) => *v,
        _ => 0,
    };
    Some((ingredient, length))
}
/// Set (or with None remove) the held skillet's cooking.
fn set_skillet_cooking(stack: &mut ItemStack, cooking: Option<(&ItemStack, i32)>) {
    let tag = cooking.map(|(ingredient, length)| {
        Tag::compound([
            (SKILLET_INGREDIENT, save::item_to_nbt(ingredient)),
            (COOKING_TIME_LENGTH, Tag::Int(length)),
        ])
    });
    stack.data_mut().set_other("minecraft:custom_data", tag);
    stack.tidy();
}
/// The skillets each side knows: their food and Fire Aspect level.
static SKILLET_VIEWS: LazyLock<Mutex<HashMap<bool, HashMap<Key, (Option<ItemStack>, u32)>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
/// The authoritative side's skillets: they tick.
static ACTIVE_SKILLETS: LazyLock<Mutex<std::collections::HashSet<Key>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));
/// A broken skillet's item, from its block entity until its drop takes it
/// (`CopySkilletFunction`).
static SKILLET_LOOT: LazyLock<Mutex<HashMap<[i32; 3], ItemStack>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// `spill::Pending::record_saved`: a skillet's item, for its drop.
pub(super) fn skillet_loot_record(pos: [i32; 3], tag: &Tag) {
    if tag.get("id") != Some(&Tag::String(SKILLET.into())) {
        return;
    }
    if let Ok(mut loot) = SKILLET_LOOT.lock() {
        match skillet_from_tag(tag).skillet {
            Some(item) => loot.insert(pos, item),
            None => loot.remove(&pos),
        };
    }
}
/// `spill::Pending::attach`: `CopySkilletFunction` (the block entity's
/// `getSkilletAsItem` in place of the plain skillet).
pub(super) fn skillet_loot_attach(pos: [i32; 3], stack: &mut ItemStack) {
    if &*stack.item != SKILLET {
        return;
    }
    if let Some(item) = SKILLET_LOOT.lock().ok().and_then(|mut l| l.remove(&pos)) {
        *stack = item;
    }
}

/// `animateTick`'s odds for a block: `ClientLevel.animateTick` picks it with
/// 667 tries each within 16 and 32 blocks of the player.
fn animate_chance(pos: [i32; 3], center: [i32; 3]) -> f64 {
    let pick = |r: f64| -> f64 {
        let axis = |d: i32| -> f64 {
            let d = f64::from(d.abs());
            if d >= r { 0.0 } else { (r - d) / (r * r) }
        };
        1.0 - (1.0
            - axis(pos[0] - center[0]) * axis(pos[1] - center[1]) * axis(pos[2] - center[2]))
        .powi(667)
    };
    1.0 - (1.0 - pick(16.0)) * (1.0 - pick(32.0))
}

impl Game<'_> {
    fn fd_skillet(&self, pos: [i32; 3]) -> Skillet {
        spill::saved_entity(&self.world, pos)
            .map(|t| skillet_from_tag(&t))
            .unwrap_or_default()
    }
    /// `inventoryChanged`: saved, sent to the players when `update`, drawn
    /// by the window, ticking.
    fn fd_put_skillet(&mut self, pos: [i32; 3], s: &Skillet, update: bool) {
        spill::put_saved_entity(&mut self.world, pos, skillet_tag(pos, s));
        self.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
        let key = (pos[0], pos[1], pos[2], self.dim.id());
        if let Ok(mut active) = ACTIVE_SKILLETS.lock() {
            active.insert(key);
        }
        if update {
            self.mark_block_entity_updated((pos[0], pos[1], pos[2]));
        }
        self.fd_publish_skillet(key, s);
    }
    fn fd_publish_skillet(&mut self, key: Key, s: &Skillet) {
        let side = self.native_window();
        if let Ok(mut views) = SKILLET_VIEWS.lock() {
            views
                .entry(side)
                .or_default()
                .insert(key, (s.stack.clone(), s.fire_aspect()));
        }
    }
    fn fd_waterlogged(&self, pos: [i32; 3]) -> bool {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        self.fd_props(state).get("waterlogged").is_some_and(|v| v == "true")
    }
    /// `updateShape` from below: a skillet's `support` and a cooking pot's
    /// (unless it hangs by its handle) follow the tray heat source under it.
    fn fd_support_update(&mut self, pos: [i32; 3]) {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let props = self.fd_props(state);
        let Some(support) = props.get("support") else {
            return;
        };
        let below = self.fd_name([pos[0], pos[1] - 1, pos[2]]);
        let tray = self.fd_tray_heat_source(&below);
        let want = match support.as_str() {
            "handle" => return,
            "true" | "false" => if tray { "true" } else { "false" },
            _ => if tray { "tray" } else { "none" },
        };
        if support != want
            && let Some(next) = self.with_props(state, &[("support", want)])
        {
            let _ = self.level_set(pos, next);
        }
    }

    /// `SkilletBlock.useItemOn` with this hand's stack: Some(true) when it
    /// took the click, None when it passes to the block's default use.
    fn fd_skillet_use(&mut self, pos: [i32; 3]) -> Option<bool> {
        let mut skillet = self.fd_skillet(pos);
        let Some(held) = self.held().cloned() else {
            // removeItem: the whole food, into the hand (not a creative one's).
            let taken = skillet.stack.take();
            if taken.is_some() {
                self.fd_put_skillet(pos, &skillet, true);
            }
            if !self.creative {
                *self.held_mut() = taken;
            }
            self.swing_arm();
            return Some(true);
        };
        // addItemToCook.
        let recipe = self
            .recipes()
            .cook(CAMPFIRE_COOKING, &held)
            .map(|r| r.time as i32);
        let mut left = held.count;
        match recipe {
            Some(time) if skillet.stack.is_none() => {
                if self.fd_waterlogged(pos) {
                    self.fd_action_bar(Text::translate("block.farmersdelight.skillet.underwater", &[]));
                } else {
                    let n = held.count.min(self.fd_max_stack(&held).min(SLOT_LIMIT));
                    skillet.stack = Some(ItemStack {
                        count: n,
                        ..held.clone()
                    });
                    skillet.cook_time_total = skillet_cooking_time(time, skillet.fire_aspect());
                    skillet.cook_time = 0;
                    self.fd_put_skillet(pos, &skillet, true);
                    if self.fd_pot_heated(pos) {
                        self.fd_play(pos, "farmersdelight:block.skillet.add_food", 0.8, 1.0);
                    }
                    left -= n;
                }
            }
            _ => self.fd_action_bar(Text::translate("block.farmersdelight.skillet.invalid_item", &[])),
        }
        if left == held.count {
            return None;
        }
        if !self.creative {
            *self.held_mut() = (left > 0).then(|| ItemStack {
                count: left,
                ..held.clone()
            });
        }
        self.fd_play(pos, "minecraft:block.lantern.place", 0.7, 1.0);
        self.swing_arm();
        Some(true)
    }

    /// `SkilletBlockEntity.cookingTick` for every skillet, and the shape its
    /// support takes from below.
    fn fd_skillet_tick(&mut self) {
        let dim = self.dim.id();
        let keys: Vec<Key> = match ACTIVE_SKILLETS.lock() {
            Ok(active) => active.iter().filter(|k| k.3 == dim).copied().collect(),
            Err(_) => return,
        };
        for key in keys {
            let pos = [key.0, key.1, key.2];
            if !self.world.loaded(pos[0], pos[2]) {
                continue;
            }
            if self.fd_name(pos) != SKILLET {
                if let Ok(mut active) = ACTIVE_SKILLETS.lock() {
                    active.remove(&key);
                }
                continue;
            }
            self.fd_support_update(pos);
            let mut s = self.fd_skillet(pos);
            if self.fd_waterlogged(pos) {
                // ItemUtils.dropItems.
                if let Some(stack) = s.stack.take() {
                    self.fd_drop_stack(pos, stack);
                    self.fd_put_skillet(pos, &s, true);
                }
                continue;
            }
            let before = (s.cook_time, s.cook_time_total);
            let mut changed = false;
            if self.fd_pot_heated(pos) {
                match s.stack.clone() {
                    None => s.cook_time = 0,
                    Some(stack) => {
                        // cookAndOutputItems.
                        s.cook_time += 1;
                        if s.cook_time >= s.cook_time_total
                            && let Some(result) = self
                                .res
                                .recipes()
                                .cook(CAMPFIRE_COOKING, &stack)
                                .map(|r| r.result.clone())
                        {
                            let state = self.world.block(pos[0], pos[1], pos[2]);
                            let facing =
                                self.fd_props(state).remove("facing").unwrap_or_else(|| "north".into());
                            let [dx, dz] = step(clockwise(&facing));
                            self.spawn_item(
                                result,
                                [
                                    f64::from(pos[0]) + 0.5,
                                    f64::from(pos[1]) + 0.3,
                                    f64::from(pos[2]) + 0.5,
                                ],
                                [
                                    dx * f64::from(0.08f32),
                                    f64::from(0.25f32),
                                    dz * f64::from(0.08f32),
                                ],
                                0,
                            );
                            s.cook_time = 0;
                            s.stack = (stack.count > 1).then(|| ItemStack {
                                count: stack.count - 1,
                                ..stack
                            });
                            changed = true;
                        }
                    }
                }
            } else if s.cook_time > 0 {
                s.cook_time = (s.cook_time - 2).max(0).min(s.cook_time_total);
            }
            if changed {
                self.fd_put_skillet(pos, &s, true);
            } else if (s.cook_time, s.cook_time_total) != before {
                spill::put_saved_entity(&mut self.world, pos, skillet_tag(pos, &s));
                self.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
            }
        }
    }

    /// The window's skillets each tick: `SkilletBlockEntity.animationTick`
    /// (steam over cooking food, a Fire Aspect skillet's sparks) and
    /// `SkilletBlock.animateTick` (the sizzle).
    fn fd_skillet_animate(&mut self, views: &[(Key, Option<ItemStack>, u32)]) {
        let Some(p) = self.player.as_ref().map(|p| p.position) else {
            return;
        };
        let center = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
        for (key, stack, fire_aspect) in views {
            let pos = [key.0, key.1, key.2];
            if stack.is_none() || !self.fd_pot_heated(pos) {
                continue;
            }
            let (x, y, z) = (f64::from(pos[0]), f64::from(pos[1]), f64::from(pos[2]));
            let r = self.particles.random();
            if r.next_float() < 0.2 {
                let sx = x + 0.5 + (r.next_double() * 0.4 - 0.2);
                let sz = z + 0.5 + (r.next_double() * 0.4 - 0.2);
                let motion = if r.next_boolean() { 0.015 } else { 0.005 };
                horizon_client::particles::farmersdelight_steam::add(
                    &mut self.particles,
                    [sx, y + 0.1, sz],
                    [0.0, motion, 0.0],
                );
            }
            let r = self.particles.random();
            if *fire_aspect > 0 && r.next_float() < *fire_aspect as f32 * 0.05 {
                let sx = x + 0.5 + (r.next_double() * 0.4 - 0.2);
                let sz = z + 0.5 + (r.next_double() * 0.4 - 0.2);
                let motion = [
                    f64::from(r.next_float() - 0.5),
                    f64::from(r.next_float() * 0.5 + 0.2),
                    f64::from(r.next_float() - 0.5),
                ];
                self.particles.add(
                    &ParticleOptions::simple(ParticleType::EnchantedHit),
                    [sx, y + 0.1, sz],
                    motion,
                );
            }
            let chance = animate_chance(pos, center);
            let r = self.particles.random();
            if r.next_double() < chance && r.next_int_bound(10).unwrap_or(1) == 0 {
                let pitch = r.next_float() * 0.2 + 0.9;
                self.audio.play(
                    "farmersdelight:block.skillet.sizzle",
                    Some([x + 0.5, y, z + 0.5]),
                    0.4,
                    pitch,
                );
            }
        }
    }

    /// `SkilletRenderer`: the food in the pan, one model more per quarter of
    /// a full stack, each a little off the centre (from a random seeded by the
    /// item's id), stacked, turned to the skillet's facing, flat, half size.
    fn fd_skillet_poses(&self, stack: &ItemStack, facing: &str) -> Vec<([f32; 3], [f32; 4], f32)> {
        let id = self
            .res
            .items()
            .order
            .iter()
            .position(|i| *i == stack.item)
            .unwrap_or(0) as i32;
        let mut random = JavaRandom::new(i64::from(id.wrapping_add(stack.damage as i32)));
        let max = self.fd_max_stack(stack) as f32;
        let count = if stack.count > 1 {
            1 + ((stack.count as f32 / max) * 4.0).ceil() as usize
        } else {
            1
        };
        let left = mul(
            axis_rotation([0.0, 1.0, 0.0], -y_rot(facing)),
            axis_rotation([1.0, 0.0, 0.0], 90.0),
        );
        (0..count)
            .map(|i| {
                let xo = (random.next_float() * 2.0 - 1.0) * 0.15 * 0.5;
                let zo = (random.next_float() * 2.0 - 1.0) * 0.15 * 0.5;
                ([0.5 + xo, 0.1 + 0.03 * (i + 1) as f32, 0.5 + zo], left, 0.5)
            })
            .collect()
    }

    // ---- the held skillet

    /// `SkilletItem.isPlayerNearHeatSource`: on fire, or a heat source in
    /// the 3x3x3 around the player's block.
    fn fd_near_heat(&self) -> bool {
        let Some(p) = self.player.as_ref() else {
            return false;
        };
        if p.fire_ticks > 0 {
            return true;
        }
        let at = [p.position.x.floor() as i32, p.position.y.floor() as i32, p.position.z.floor() as i32];
        (-1..=1).any(|dx| {
            (-1..=1).any(|dy| {
                (-1..=1).any(|dz| self.fd_heat_source([at[0] + dx, at[1] + dy, at[2] + dz]).is_some())
            })
        })
    }
    /// `Entity.isUnderWater`: the eyes in water.
    fn fd_under_water(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(|p| p.is_eye_in_fluid(horizon_client::motion::WATER))
    }
    /// The other hand's stack.
    fn fd_other_hand(&self) -> Option<ItemStack> {
        match self.hand {
            Hand::Main => self.offhand.clone(),
            Hand::Off => self.inventory.get(self.selected).cloned().flatten(),
        }
    }
    fn fd_other_hand_mut(&mut self) -> &mut Option<ItemStack> {
        match self.hand {
            Hand::Main => &mut self.offhand,
            Hand::Off => {
                let selected = self.selected;
                &mut self.inventory[selected]
            }
        }
    }

    /// `Game::use_tool`'s HEAD for a skillet in the acting hand: placed only
    /// when sneaking (`SkilletItem.place`), else `SkilletItem.use`: near
    /// heat, the other hand's campfire food goes in and the use starts.
    pub(super) fn fd_skillet_item_use(
        &mut self,
        stack: &ItemStack,
        on_block: bool,
    ) -> Option<horizon_client::first_person::UseResult> {
        use horizon_client::first_person::UseResult;
        if &*stack.item != SKILLET || !self.native_on("farmersdelight") {
            return None;
        }
        if on_block {
            // BlockItem.useOn: placed when sneaking; otherwise PASS, and the
            // client's own UseItem comes next.
            let sneaking = self.keys.contains(&winit::keyboard::KeyCode::ShiftLeft);
            return (!sneaking).then_some(UseResult::Fail);
        }
        if !self.fd_near_heat() {
            return Some(UseResult::Pass);
        }
        if skillet_cooking(stack).is_some() {
            self.using = Some((self.held_slot(), 0));
            return Some(UseResult::Pass);
        }
        let other = self.fd_other_hand();
        let recipe = other
            .as_ref()
            .and_then(|o| self.recipes().cook(CAMPFIRE_COOKING, o).map(|r| r.time as i32));
        let (Some(other), Some(time)) = (other, recipe) else {
            self.fd_action_bar(Text::translate("item.farmersdelight.skillet.how_to_cook", &[]));
            return Some(UseResult::Pass);
        };
        if self.fd_under_water() {
            self.fd_action_bar(Text::translate("item.farmersdelight.skillet.underwater", &[]));
            return Some(UseResult::Pass);
        }
        let unit = ItemStack {
            count: 1,
            ..other.clone()
        };
        let mut skillet = stack.clone();
        set_skillet_cooking(&mut skillet, Some((&unit, time)));
        *self.held_mut() = Some(skillet);
        // cookingStackCopy.split(1), creative or not.
        *self.fd_other_hand_mut() = (other.count > 1).then(|| ItemStack {
            count: other.count - 1,
            ..other
        });
        self.using = Some((self.held_slot(), 0));
        Some(UseResult::Success { swing: false })
    }
    /// `SkilletItem.onUseTick` and `getUseDuration` for the skillet in use in
    /// `slot`: the sizzle now and then, and `finishUsingItem` when the
    /// cooking time is up. True when it was a skillet.
    pub(super) fn fd_skillet_use_tick(&mut self, slot: usize, ticks: u32, stack: &ItemStack) -> bool {
        if &*stack.item != SKILLET {
            return false;
        }
        let Some((ingredient, length)) = skillet_cooking(stack) else {
            self.using = None;
            return true;
        };
        let fire_aspect = stack.enchantment_level("minecraft:fire_aspect");
        let duration = skillet_cooking_time(length, fire_aspect).max(0) as u32;
        // The client's sizzle (1 in 50), heard by the cook.
        if self.world.random.next_int_bound(50).unwrap_or(1) == 0 {
            let pitch = self.world.random.next_float() * 0.2 + 0.9;
            let at = self.player.as_ref().map(|p| [p.position.x, p.position.y, p.position.z]);
            self.audio.play("farmersdelight:block.skillet.sizzle", at, 0.4, pitch);
        }
        let ticks = ticks + 1;
        if ticks < duration {
            self.using = Some((slot, ticks));
            return true;
        }
        // finishUsingItem: the cooked food to the inventory (or dropped).
        self.using = None;
        self.send_use_item_complete();
        if let Some(result) = self
            .res
            .recipes()
            .cook(CAMPFIRE_COOKING, &ingredient)
            .map(|r| r.result.clone())
            && let Some(rest) = self.give(result)
        {
            self.player_drop(rest);
        }
        if let Some(s) = self.stack_in_mut(slot).as_mut() {
            set_skillet_cooking(s, None);
        }
        true
    }
    /// `SkilletItem.releaseUsing`: the ingredient back to the inventory.
    pub(super) fn fd_skillet_released(&mut self, stack: &ItemStack) {
        if &*stack.item != SKILLET {
            return;
        }
        let Some((ingredient, _)) = skillet_cooking(stack) else {
            return;
        };
        let slots: Vec<usize> = (0..36).chain([super::shield::OFFHAND_SLOT]).collect();
        let Some(slot) = slots
            .into_iter()
            .find(|&i| self.stack_in(i).is_some_and(|s| s == stack))
        else {
            return;
        };
        if let Some(s) = self.stack_in_mut(slot).as_mut() {
            set_skillet_cooking(s, None);
        }
        // placeItemBackInInventory.
        if let Some(rest) = self.give(ingredient)
            && let Some(eye) = self.eye_position()
        {
            self.throw_from_eyes(rest, eye, false);
        }
    }
    /// The window's prediction of `SkilletItem.use`: whether the use starts.
    pub(super) fn fd_skillet_predict_use(&self, held: &ItemStack) -> Option<bool> {
        if &*held.item != SKILLET {
            return None;
        }
        if !self.fd_near_heat() {
            return Some(false);
        }
        if skillet_cooking(held).is_some() {
            return Some(true);
        }
        let other = self.fd_other_hand();
        let cookable = other
            .as_ref()
            .is_some_and(|o| self.recipes().cook(CAMPFIRE_COOKING, o).is_some());
        Some(cookable && !self.fd_under_water())
    }
    /// `SkilletEvents.playSkilletAttackSound` for a melee hit that landed with
    /// a skillet in the main hand: the strong sound when the attack strength
    /// (`getAttackStrengthScale(0)`, after the attack reset it) is above 0.8.
    pub(super) fn fd_skillet_hit(&mut self, kind: &str, direct: bool) {
        if direct || kind != "minecraft:player_attack" {
            return;
        }
        if !self
            .inventory
            .get(self.selected)
            .and_then(Option::as_ref)
            .is_some_and(|s| &*s.item == SKILLET)
        {
            return;
        }
        let Some(at) = self.player.as_ref().map(|p| [p.position.x, p.position.y, p.position.z]) else {
            return;
        };
        let delay = horizon_client::attack_strength::delay(self.main_hand_attack_speed());
        let power = (self.attack_ticks as f32 / delay).clamp(0.0, 1.0);
        if power > 0.8 {
            let pitch = 0.9 + self.world.random.next_float() * 0.2;
            self.audio.play("farmersdelight:item.skillet.attack.strong", Some(at), 1.0, pitch);
        } else {
            // The pitch draw comes first in the event, either way.
            let _ = self.world.random.next_float();
            self.audio.play("farmersdelight:item.skillet.attack.weak", Some(at), 0.8, 0.9);
        }
    }
}

// ---------------------------------------------------------------- crops

const RICE: &str = "farmersdelight:rice";
const RICE_PANICLES: &str = "farmersdelight:rice_panicles";
const WILD_RICE: &str = "farmersdelight:wild_rice";
const BUDDING_TOMATOES: &str = "farmersdelight:budding_tomatoes";
const TOMATOES: &str = "farmersdelight:tomatoes";

impl Game<'_> {
    /// `Level.getRawBrightness(pos, 0)`: the sky's light, undarkened, or the
    /// block light.
    fn fd_raw_light(&self, p: [i32; 3]) -> i32 {
        let light = self.world.light(p[0], p[1], p[2]);
        i32::from(light >> 4).max(i32::from(light & 15))
    }
    /// `RandomSource.nextInt(bound)`.
    fn fd_rand(&mut self, bound: i32) -> i32 {
        self.world.random.next_int_bound(bound.max(1)).unwrap_or(0)
    }
    /// `Mth.nextInt(random, min, max)`, both included.
    fn fd_rand_between(&mut self, min: i32, max: i32) -> i32 {
        min + self.fd_rand(max - min + 1)
    }
    fn fd_default(&self, block: &str) -> Option<BlockStateId> {
        self.pack.assets.registry.default_state(block)
    }
    fn fd_with_age(&self, state: BlockStateId, age: i32) -> Option<BlockStateId> {
        self.with_props(state, &[("age", &age.to_string())])
    }
    /// `RicePaniclesBlock.canSurvive` at `p`: light (or the sky), rice below.
    fn fd_panicles_survive(&self, p: [i32; 3]) -> bool {
        let light = self.world.light(p[0], p[1], p[2]);
        (self.fd_raw_light(p) >= 8 || light >> 4 == 15) && self.fd_name([p[0], p[1] - 1, p[2]]) == RICE
    }

    /// `BoneMealItem.useOn` on the mod's crops (their `BonemealableBlock`):
    /// Some(used) when the clicked block is one of them.
    pub(super) fn fd_bone_meal(&mut self, stack: &ItemStack) -> Option<bool> {
        if &*stack.item != "minecraft:bone_meal" || !self.native_on("farmersdelight") {
            return None;
        }
        let pos = self.pick()?.pos;
        let name = self.fd_name(pos);
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let age = self.fd_int_prop(state, "age");
        let above = [pos[0], pos[1] + 1, pos[2]];
        let above_state = self.world.block(above[0], above[1], above[2]);
        // isValidBonemealTarget.
        let valid = match name.as_str() {
            RICE => self.fd_name(above) != RICE_PANICLES || self.fd_int_prop(above_state, "age") < 3,
            RICE_PANICLES | TOMATOES => age < 3,
            BUDDING_TOMATOES | WILD_RICE => true,
            _ => return None,
        };
        if !valid {
            return Some(false);
        }
        // performBonemeal (isBonemealSuccess: always; the wild rice's 30%).
        match name.as_str() {
            RICE => {
                let growth = (age + self.fd_rand_between(1, 4)).min(7);
                if growth <= 3 {
                    if let Some(next) = self.fd_with_age(state, growth) {
                        let _ = self.level_set(pos, next);
                    }
                } else if self.fd_name(above) == RICE_PANICLES {
                    // The panicles' own bone meal (CropBlock's, a third of it).
                    let panicles_age = self.fd_int_prop(above_state, "age");
                    let more = self.fd_rand_between(2, 5) / 3;
                    if let Some(next) = self.fd_with_age(above_state, (panicles_age + more).min(3)) {
                        let _ = self.level_set(above, next);
                    }
                } else if self.fd_name(above) == "minecraft:air" && self.fd_panicles_survive(above) {
                    if let Some(next) = self.fd_with_age(state, 3) {
                        let _ = self.level_set(pos, next);
                    }
                    if let Some(panicles) = self.fd_default(RICE_PANICLES)
                        && let Some(panicles) = self.fd_with_age(panicles, growth - 4)
                    {
                        let _ = self.level_set(above, panicles);
                    }
                }
            }
            RICE_PANICLES => {
                let more = self.fd_rand_between(2, 5) / 3;
                if let Some(next) = self.fd_with_age(state, (age + more).min(3)) {
                    let _ = self.level_set(pos, next);
                }
            }
            TOMATOES => {
                let grown = age + self.fd_rand_between(2, 5) / 2;
                if grown <= 3
                    && let Some(next) = self.fd_with_age(state, grown)
                {
                    let _ = self.level_set(pos, next);
                }
            }
            BUDDING_TOMATOES => {
                let growth = (age + self.fd_rand_between(1, 4)).min(7);
                let next = if growth <= 3 {
                    self.fd_with_age(state, growth)
                } else {
                    self.fd_default(TOMATOES).and_then(|t| self.fd_with_age(t, growth - 4))
                };
                if let Some(next) = next {
                    let _ = self.level_set(pos, next);
                }
            }
            _ => {
                if self.world.random.next_float() < 0.3 {
                    self.fd_drop_stack(pos, ItemStack::new(WILD_RICE, 1));
                }
            }
        }
        // ITEM_INTERACT_FINISH and the growth particles (level event 1505).
        self.level_event_at(1505, pos, 15);
        self.swing_arm();
        self.stats.add("used", &stack.item, 1);
        self.consume_held();
        let _ = self.process_level_events();
        Some(true)
    }
    /// The window's `BoneMealItem.useOn` on the mod's crops: whether it
    /// succeeds (`isValidBonemealTarget`).
    pub(super) fn fd_bone_meal_predict(&self, pos: [i32; 3]) -> Option<bool> {
        if !self.native_on("farmersdelight") {
            return None;
        }
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let age = self.fd_int_prop(state, "age");
        let above = [pos[0], pos[1] + 1, pos[2]];
        match self.fd_name(pos).as_str() {
            RICE => Some(
                self.fd_name(above) != RICE_PANICLES
                    || self.fd_int_prop(self.world.block(above[0], above[1], above[2]), "age") < 3,
            ),
            RICE_PANICLES | TOMATOES => Some(age < 3),
            BUDDING_TOMATOES | WILD_RICE => Some(true),
            _ => None,
        }
    }

    /// Where `BlockPlaceContext` puts a block for a click on `face` of `pos`:
    /// the clicked block when it can be replaced, else the one beside it.
    fn fd_place_target(&self, pos: [i32; 3], face: [i32; 3]) -> [i32; 3] {
        let clicked = self.world.block(pos[0], pos[1], pos[2]);
        if self.res.facts().get(clicked).replaceable {
            pos
        } else {
            [pos[0] + face[0], pos[1] + face[1], pos[2] + face[2]]
        }
    }
    /// `FluidState.is(WATER) && getAmount() == 8`: a source, falling water,
    /// a block waterlogged or one holding its water (seagrass, kelp, rice).
    fn fd_full_water(&self, pos: [i32; 3]) -> bool {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let Some(s) = self.pack.assets.registry.state(state) else {
            return false;
        };
        let prop = |k: &str| s.properties.get(k).map(|v| v.to_string());
        match &*s.name {
            "minecraft:water" => prop("level").is_some_and(|l| l == "0" || l.parse::<u8>().is_ok_and(|l| l >= 8)),
            "minecraft:seagrass" | "minecraft:tall_seagrass" | "minecraft:kelp" | "minecraft:kelp_plant"
            | "minecraft:bubble_column" | RICE => true,
            WILD_RICE => prop("half").as_deref() == Some("lower"),
            _ => prop("waterlogged").as_deref() == Some("true"),
        }
    }
    /// The items whose block refuses the place they would go: `RiceBlock`
    /// and `WildRiceBlock.getStateForPlacement` want a full water block (the
    /// wild rice with air above it), `BuddingBushBlock.mayPlaceOn` farmland.
    /// Some(false) when the item can't be placed at `target`.
    fn fd_crop_place_ok(&self, item: &str, target: [i32; 3]) -> Option<bool> {
        let above = [target[0], target[1] + 1, target[2]];
        let below = self.fd_name([target[0], target[1] - 1, target[2]]);
        match item {
            "farmersdelight:rice" => Some(self.fd_full_water(target)),
            "farmersdelight:wild_rice" => Some(self.fd_full_water(target) && self.fd_name(above) == "minecraft:air"),
            "farmersdelight:tomato_seeds" => {
                Some(below == "minecraft:farmland" || below == "farmersdelight:rich_soil_farmland")
            }
            // Tatami mats: `canSurvive` (something under them); the full mat
            // also wants room for its head the player's way
            // (`TatamiMatBlock.getStateForPlacement`).
            "farmersdelight:half_tatami_mat" => Some(!is_air(&below)),
            "farmersdelight:full_tatami_mat" => {
                let head = add3(target, dir_step(horizontal_direction(self.yaw)));
                let head_state = self.world.block(head[0], head[1], head[2]);
                let under_head = self.fd_name([head[0], head[1] - 1, head[2]]);
                Some(!is_air(&below) && self.res.facts().get(head_state).replaceable && !is_air(&under_head))
            }
            _ => None,
        }
    }
    /// `Game::use_tool`'s HEAD for the crops' items: their use fails where
    /// their block can't go.
    pub(super) fn fd_crop_item_use(
        &mut self,
        stack: &ItemStack,
        on_block: bool,
    ) -> Option<horizon_client::first_person::UseResult> {
        if !on_block || !self.native_on("farmersdelight") || !self.may_build() {
            return None;
        }
        let hit = self.pick()?;
        let target = self.fd_place_target(hit.pos, hit.face);
        match self.fd_crop_place_ok(&stack.item, target) {
            Some(false) => Some(horizon_client::first_person::UseResult::Fail),
            _ => None,
        }
    }
    /// The window's prediction of the same: no block where it can't go.
    pub(super) fn fd_crop_predict_place(&self, pos: [i32; 3], face: [i32; 3], item: &str) -> bool {
        let target = self.fd_place_target(pos, face);
        self.fd_crop_place_ok(item, target) != Some(false)
    }

    /// `TomatoBlock.useWithoutItem`: a ripe vine gives one or two tomatoes
    /// (a rotten one in twenty times) and starts again.
    fn fd_tomato_pick(&mut self, pos: [i32; 3]) -> Option<bool> {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        if self.fd_int_prop(state, "age") < 3 {
            return None;
        }
        let quantity = 1 + self.fd_rand(2);
        self.fd_drop_stack(pos, ItemStack::new("farmersdelight:tomato", quantity as u32));
        if self.world.random.next_float() < 0.05 {
            self.fd_drop_stack(pos, ItemStack::new("farmersdelight:rotten_tomato", 1));
        }
        let pitch = 0.8 + self.world.random.next_float() * 0.4;
        self.fd_play(pos, "farmersdelight:block.tomatoes.pick_tomatoes", 1.0, pitch);
        if let Some(next) = self.fd_with_age(state, 0) {
            let _ = self.level_set(pos, next);
        }
        self.swing_arm();
        Some(true)
    }
}
// ---------------------------------------------------------------- tooltips

/// The mod's tooltip lines by item id, as the tables wrote them
/// (`farmersdelight-tooltips.json` in the content root): JSON text components.
static TOOLTIP_LINES: std::sync::OnceLock<HashMap<String, Vec<serde_json::Value>>> = std::sync::OnceLock::new();

impl Game<'_> {
    /// The tooltip hook point: `Item.appendHoverText`'s lines come right
    /// under the name, before the components' (`ItemStack.getTooltipLines`).
    pub(super) fn fd_tooltip(&self, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
        let map = TOOLTIP_LINES.get_or_init(|| {
            let path = Game::options_path()
                .with_file_name(".horizon-cache")
                .join("mod-content")
                .join("farmersdelight-tooltips.json");
            std::fs::read(path)
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_default()
        });
        let Some(list) = map.get(&*stack.item) else {
            return;
        };
        let at = lines.len().min(1);
        let added = list
            .iter()
            .map(|v| (horizon_client::component::to_text(v, &mut |_| Text::empty()), u32::MAX));
        lines.splice(at..at, added);
    }
}
// ---------------------------------------------------------------- tatami

const TATAMI: &str = "farmersdelight:tatami";
const FULL_MAT: &str = "farmersdelight:full_tatami_mat";
const HALF_MAT: &str = "farmersdelight:half_tatami_mat";

/// `Level.isEmptyBlock`.
fn is_air(name: &str) -> bool {
    matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
}

impl Game<'_> {
    /// `TatamiMatBlock.playerWillDestroy`: a creative player breaking the
    /// foot removes the head first, with its break particles and no drop.
    pub(super) fn fd_player_will_destroy(&mut self, pos: [i32; 3]) {
        if !self.creative || !self.native_on("farmersdelight") || self.fd_name(pos) != FULL_MAT {
            return;
        }
        let props = self.fd_props(self.world.block(pos[0], pos[1], pos[2]));
        if props.get("part").map(String::as_str) != Some("foot") {
            return;
        }
        let head = add3(pos, dir_step(props.get("facing").map(String::as_str).unwrap_or("north")));
        let head_state = self.world.block(head[0], head[1], head[2]);
        if self.fd_name(head) == FULL_MAT
            && self.fd_props(head_state).get("part").map(String::as_str) == Some("head")
            && let Some(air) = self.fd_default("minecraft:air")
        {
            let _ = self.level_set(head, air);
            self.level_event_at(2001, head, head_state.0 as i32);
        }
    }
}
impl Game<'_> {
    /// `MultiPlayerGameMode.destroyBlock`'s `fluidState.createLegacyBlock()`
    /// for the mod's blocks holding water: the window's break leaves water.
    pub(super) fn fd_predict_break(&self, pos: [i32; 3], state: BlockStateId) {
        use horizon_client::world::farmersdelight_ropes::BREAK_WATER;
        if !self.res.facts().get(state).name.starts_with("farmersdelight:")
            || self.fd_props(state).get("waterlogged").map(String::as_str) != Some("true")
        {
            return;
        }
        if let Some(water) = self.fd_default("minecraft:water")
            && let Ok(mut pending) = BREAK_WATER.lock()
        {
            *pending = Some((pos, water.0));
        }
    }
}
// ---------------------------------------------------------------- rope

const ROPE: &str = "farmersdelight:rope";

impl Game<'_> {
    /// `RopeBlock.tieToAnythingValid` / `tieToRopeAndWalls` toward `side`:
    /// bars, panes, ropes and walls always; on a horizontal placement any
    /// sturdy face that is no connection exception too.
    fn fd_rope_ties(&self, pos: [i32; 3], side: &str, horizontal: bool) -> bool {
        use horizon_core::geometry::Direction as D;
        let q = add3(pos, dir_step(side));
        let state = self.world.block(q[0], q[1], q[2]);
        let facts = self.res.facts().get(state);
        if facts.classes.iter().any(|c| &**c == "IronBarsBlock" || &**c == "WallBlock") {
            return true;
        }
        if !horizontal {
            return false;
        }
        let name = &*facts.name;
        let exception = name.ends_with("_leaves")
            || name.ends_with("shulker_box")
            || matches!(
                name,
                "minecraft:barrier"
                    | "minecraft:carved_pumpkin"
                    | "minecraft:jack_o_lantern"
                    | "minecraft:melon"
                    | "minecraft:pumpkin"
            );
        let back = match side {
            "north" => D::South,
            "south" => D::North,
            "east" => D::West,
            _ => D::East,
        };
        !exception && self.res.behaviours().is_face_sturdy(state, back)
    }
    /// `RopeBlock.getStateWithConnections` at `pos` for a click on `face`.
    fn fd_rope_props(&self, pos: [i32; 3], face: &str, water: bool) -> Vec<(&'static str, &'static str)> {
        let horizontal = !matches!(face, "up" | "down");
        let b = |v: bool| if v { "true" } else { "false" };
        let bell = self.fd_name([pos[0], pos[1] + 1, pos[2]]) == "minecraft:bell";
        vec![
            ("tied_to_bell", b(bell)),
            ("north", b(self.fd_rope_ties(pos, "north", horizontal))),
            ("south", b(self.fd_rope_ties(pos, "south", horizontal))),
            ("west", b(self.fd_rope_ties(pos, "west", horizontal))),
            ("east", b(self.fd_rope_ties(pos, "east", horizontal))),
            ("waterlogged", b(water)),
        ]
    }
    /// `RopeItem.updatePlacementContext`: a click that would place rope into
    /// rope (the rope itself, or the face behind it) goes on to the first
    /// free block below the rope's end, or with sneaking one step the
    /// clicked face's way. None: an ordinary placement; Some(None): none at
    /// all; else the position and the placement's face.
    pub(super) fn fd_rope_target(
        &self,
        hit_pos: [i32; 3],
        face: [i32; 3],
        sneaking: bool,
    ) -> Option<Option<([i32; 3], &'static str)>> {
        // RopeBlock.canBeReplaced with rope in hand: the click replaces it.
        let target = if self.fd_name(hit_pos) == ROPE {
            hit_pos
        } else {
            self.fd_place_target(hit_pos, face)
        };
        if self.fd_name(target) != ROPE {
            return None;
        }
        let (step, toward) = if sneaking { (face, face_name(face)) } else { ([0, -1, 0], "down") };
        let bottom = self.world.layout().min_y();
        let mut p = add3(target, step);
        while p[1] >= bottom {
            let name = self.fd_name(p);
            if name != ROPE {
                // Lava (any fluid but water) stops it.
                if name == "minecraft:lava" {
                    return Some(None);
                }
                let state = self.world.block(p[0], p[1], p[2]);
                return Some(self.res.facts().get(state).replaceable.then_some((p, toward)));
            }
            if toward != "down" {
                return Some(None);
            }
            p = add3(p, step);
        }
        Some(None)
    }
    /// The rope's state placed at `p` by the redirected click.
    pub(super) fn fd_rope_state(&self, p: [i32; 3], face: &str) -> Option<BlockStateId> {
        let props = self.fd_rope_props(p, face, self.fd_water_source(p));
        self.with_props(self.fd_default(ROPE)?, &props)
    }
    /// `Game::use_tool`'s HEAD for rope: a click into rope extends it.
    pub(super) fn fd_rope_item_use(
        &mut self,
        stack: &ItemStack,
        on_block: bool,
    ) -> Option<horizon_client::first_person::UseResult> {
        use horizon_client::first_person::UseResult;
        if &*stack.item != "farmersdelight:rope" || !on_block || !self.native_on("farmersdelight") || !self.may_build() {
            return None;
        }
        let hit = self.pick()?;
        let sneaking = self.keys.contains(&winit::keyboard::KeyCode::ShiftLeft);
        let Some((p, face)) = self.fd_rope_target(hit.pos, hit.face, sneaking)? else {
            return Some(UseResult::Fail);
        };
        let state = self.fd_rope_state(p, face)?;
        if self.level_set(p, state).is_err() {
            return Some(UseResult::Fail);
        }
        // BlockItem.place: the place sound, the use, one item gone.
        let sounds = self.res.facts().get(state).sounds.clone();
        self.fd_play(p, &sounds.place, (sounds.volume + 1.0) / 2.0, sounds.pitch * 0.8);
        self.stats.add("used", &stack.item, 1);
        if !self.creative {
            self.consume_held();
        }
        let _ = self.process_level_events();
        Some(UseResult::Success { swing: true })
    }
}
// ------------------------------------------------- the level's hook points
//
// The game's stable hook points for a level (`native_random_tick`,
// `native_update_shape`): the mod's crops grow in vanilla's own random ticks,
// and its blocks answer their neighbours' changes as their Java classes do.

/// `Level.setBlock` flags.
const UPDATE_CLIENTS: i32 = 2;
const UPDATE_ALL: i32 = 3;

impl super::GameLevel<'_> {
    fn fd_lname(&self, p: [i32; 3]) -> String {
        self.facts.get(self.world.block(p[0], p[1], p[2])).name.to_string()
    }
    fn fd_lprop(&self, state: BlockStateId, key: &str) -> Option<String> {
        self.registry.state(state)?.properties.get(key).cloned()
    }
    fn fd_lwith(&self, state: BlockStateId, props: &[(&str, &str)]) -> Option<BlockStateId> {
        let mut s = self.registry.state(state)?.clone();
        for (k, v) in props {
            s.properties.insert((*k).to_owned(), (*v).to_owned());
        }
        self.registry.resolve(&s).ok()
    }
    fn fd_lage(&self, state: BlockStateId) -> i32 {
        self.fd_lprop(state, "age").and_then(|a| a.parse().ok()).unwrap_or(0)
    }
    /// `Level.getRawBrightness(pos, 0)`.
    fn fd_llight(&self, p: [i32; 3]) -> i32 {
        let light = self.world.light(p[0], p[1], p[2]);
        i32::from(light >> 4).max(i32::from(light & 15))
    }
    fn fd_lrand(&mut self, bound: i32) -> i32 {
        self.world.random.next_int_bound(bound.max(1)).unwrap_or(0)
    }
    fn fd_lset(&mut self, ticks: &mut horizon_world::level::Ticks, p: [i32; 3], state: BlockStateId, flags: i32) {
        let b = self.behaviours;
        b.set_block(self, ticks, horizon_core::position::BlockPos::new(p[0], p[1], p[2]), state, flags);
    }
    /// `RicePaniclesBlock.canSurvive` at `p`: light (or the sky), rice below.
    fn fd_lpanicles_survive(&self, p: [i32; 3]) -> bool {
        let light = self.world.light(p[0], p[1], p[2]);
        (self.fd_llight(p) >= 8 || light >> 4 == 15) && self.fd_lname([p[0], p[1] - 1, p[2]]) == RICE
    }
    /// `CropBlock.getGrowthSpeed` (`BuddingBushBlock`'s when `budding`).
    fn fd_lgrowth_speed(&self, pos: [i32; 3], name: &str, budding: bool) -> f32 {
        let mut speed = 1.0f32;
        for dx in -1..=1 {
            for dz in -1..=1 {
                let soil = self.world.block(pos[0] + dx, pos[1] - 1, pos[2] + dz);
                let soil_name = self.facts.get(soil).name.to_string();
                let farmland = soil_name == "minecraft:farmland" || soil_name == "farmersdelight:rich_soil_farmland";
                let mut bonus = 0.0;
                if budding || farmland {
                    let moist = self.fd_lprop(soil, "moisture").is_some_and(|m| m != "0");
                    bonus = if farmland && moist { 3.0 } else { 1.0 };
                }
                if dx != 0 || dz != 0 {
                    bonus /= 4.0;
                }
                speed += bonus;
            }
        }
        let same = |dx: i32, dz: i32| self.fd_lname([pos[0] + dx, pos[1], pos[2] + dz]) == name;
        let east_west = same(-1, 0) || same(1, 0);
        let north_south = same(0, -1) || same(0, 1);
        if (east_west && north_south) || same(-1, -1) || same(1, -1) || same(1, 1) || same(-1, 1) {
            speed /= 2.0;
        }
        speed
    }

    /// `native_random_tick`: the crops' own `randomTick` (`RiceBlock`,
    /// `RicePaniclesBlock`, `BuddingTomatoBlock`, `TomatoBlock`); true when
    /// the block is one of them (its vanilla template's tick is skipped).
    pub(super) fn fd_random_tick(
        &mut self,
        ticks: &mut horizon_world::level::Ticks,
        pos: horizon_core::position::BlockPos,
        state: BlockStateId,
    ) -> bool {
        let name = self.facts.get(state).name.to_string();
        if !matches!(name.as_str(), RICE | RICE_PANICLES | BUDDING_TOMATOES | TOMATOES) {
            return false;
        }
        let p = [pos.x, pos.y, pos.z];
        let age = self.fd_lage(state);
        let above = [p[0], p[1] + 1, p[2]];
        let with_age = |level: &Self, a: i32| level.fd_lwith(state, &[("age", &a.to_string())]);
        match name.as_str() {
            // RiceBlock: light 6 above, one chance in three; at its last age
            // the panicles grow above it.
            RICE => {
                if self.fd_llight(above) < 6 || self.fd_lrand(3) != 0 {
                    return true;
                }
                if age >= 3 {
                    if self.fd_lname(above) == "minecraft:air"
                        && self.fd_lpanicles_survive(above)
                        && let Some(panicles) = self.registry.default_state(RICE_PANICLES)
                    {
                        self.fd_lset(ticks, above, panicles, UPDATE_ALL);
                    }
                } else if let Some(next) = with_age(self, age + 1) {
                    self.fd_lset(ticks, p, next, UPDATE_CLIENTS);
                }
            }
            // RicePaniclesBlock: CropBlock's randomTick.
            RICE_PANICLES => {
                if self.fd_llight(p) >= 9 && age < 3 {
                    let speed = self.fd_lgrowth_speed(p, &name, false);
                    if self.fd_lrand((25.0 / speed) as i32 + 1) == 0
                        && let Some(next) = with_age(self, age + 1)
                    {
                        self.fd_lset(ticks, p, next, UPDATE_CLIENTS);
                    }
                }
            }
            // BuddingBushBlock, then BuddingTomatoBlock.growPastMaxAge.
            BUDDING_TOMATOES => {
                if self.fd_llight(p) >= 9 && age <= 3 {
                    let speed = self.fd_lgrowth_speed(p, &name, true);
                    if self.fd_lrand((25.0 / speed) as i32 + 1) == 0 {
                        let next = if age >= 3 { self.registry.default_state(TOMATOES) } else { with_age(self, age + 1) };
                        if let Some(next) = next {
                            self.fd_lset(ticks, p, next, UPDATE_ALL);
                        }
                    }
                }
            }
            // TomatoBlock: one chance in six.
            _ => {
                if self.fd_llight(p) >= 9
                    && age < 3
                    && self.fd_lrand(6) == 0
                    && let Some(next) = with_age(self, age + 1)
                {
                    self.fd_lset(ticks, p, next, UPDATE_CLIENTS);
                }
            }
        }
        true
    }

    /// `native_update_shape`: the mod's blocks' `updateShape` toward their
    /// neighbour in `direction` (`neighbour`): Some(new state), or None to
    /// leave the block to its vanilla template.
    pub(super) fn fd_update_shape(
        &self,
        pos: horizon_core::position::BlockPos,
        state: BlockStateId,
        direction: horizon_core::geometry::Direction,
        neighbour: BlockStateId,
    ) -> Option<BlockStateId> {
        use horizon_core::geometry::Direction as D;
        let name = self.facts.get(state).name.to_string();
        let other = self.facts.get(neighbour).name.to_string();
        let p = [pos.x, pos.y, pos.z];
        let side = match direction {
            D::North => "north",
            D::South => "south",
            D::East => "east",
            D::West => "west",
            D::Up => "up",
            D::Down => "down",
        };
        let air = || self.registry.default_state("minecraft:air");
        match name.as_str() {
            // RopeFenceBlock.updateShape: a horizontal neighbour sets that side.
            ROPE_FENCE => {
                let (back, back_cw_x) = match direction {
                    D::North => (D::South, true),
                    D::South => (D::North, true),
                    D::East => (D::West, false),
                    D::West => (D::East, false),
                    _ => return Some(state),
                };
                let connects = if other == ROPE_FENCE {
                    true
                } else if other == ROPE_FENCE_GATE {
                    let facing = self.fd_lprop(neighbour, "facing").unwrap_or_default();
                    matches!(facing.as_str(), "east" | "west") == back_cw_x
                } else {
                    let exception = other.ends_with("_leaves")
                        || other.ends_with("shulker_box")
                        || matches!(
                            other.as_str(),
                            "minecraft:barrier"
                                | "minecraft:carved_pumpkin"
                                | "minecraft:jack_o_lantern"
                                | "minecraft:melon"
                                | "minecraft:pumpkin"
                        );
                    !exception && self.behaviours.is_face_sturdy(neighbour, back)
                };
                self.fd_lwith(state, &[(side, if connects { "true" } else { "false" })])
            }
            // TatamiHalfMatBlock: nothing under it, it breaks.
            HALF_MAT => {
                if is_air(&self.fd_lname([p[0], p[1] - 1, p[2]])) { air() } else { Some(state) }
            }
            // TatamiMatBlock: its other half, and something under it.
            FULL_MAT => {
                let part = self.fd_lprop(state, "part").unwrap_or_default();
                let facing = self.fd_lprop(state, "facing").unwrap_or_default();
                let toward = if part == "foot" { facing.as_str() } else { opposite(&facing) };
                let survives = !is_air(&self.fd_lname([p[0], p[1] - 1, p[2]]));
                if side == toward {
                    let paired = other == FULL_MAT && self.fd_lprop(neighbour, "part").as_deref() != Some(part.as_str());
                    if survives && paired { Some(state) } else { air() }
                } else if survives {
                    Some(state)
                } else {
                    air()
                }
            }
            // TatamiBlock: a paired tatami whose pair is gone unpairs.
            TATAMI => {
                let facing = self.fd_lprop(state, "facing").unwrap_or_default();
                let paired = self.fd_lprop(state, "paired").as_deref() == Some("true");
                if side == facing && paired && other != TATAMI {
                    self.fd_lwith(state, &[("paired", "false")])
                } else {
                    Some(state)
                }
            }
            // RiceBlock: the panicles above make it `supporting`; below, the
            // vanilla bush's survival stays.
            RICE if direction == D::Up => {
                self.fd_lwith(state, &[("supporting", if other == RICE_PANICLES { "true" } else { "false" })])
            }
            // RicePaniclesBlock.canSurvive: the rice under it.
            RICE_PANICLES if direction == D::Down => {
                if other == RICE { Some(state) } else { air() }
            }
            // CookingPotBlock / SkilletBlock.updateShape: on a vertical
            // update the tray follows the block below (`getTrayState`); a
            // hanging pot keeps its handle.
            POT | SKILLET if matches!(direction, D::Up | D::Down) => {
                let support = self.fd_lprop(state, "support").unwrap_or_default();
                if support == "handle" {
                    return Some(state);
                }
                let below = if direction == D::Down { other } else { self.fd_lname([p[0], p[1] - 1, p[2]]) };
                let tray = tray_heat_source(&below);
                let value = match (name.as_str(), tray) {
                    (POT, true) => "tray",
                    (POT, false) => "none",
                    (_, true) => "true",
                    (_, false) => "false",
                };
                self.fd_lwith(state, &[("support", value)])
            }
            _ => None,
        }
    }
}
