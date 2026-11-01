// Farmer's Delight block tables, generated from the mod's own facts.
//
// Horizon's content API gives a mod block the states, shapes and behaviour of
// a vanilla template. Farmer's Delight blocks have states no vanilla block
// has (a cooking pot's `support`, a feast's `servings`), so after the game has
// built its resource overlay this rewrites the overlay's block tables: every
// Farmer's Delight state is generated from the NeoForge oracle (the real mod's
// states, outline and collision shapes, sounds, hardness, light, class chain).
// Vanilla rows are never touched; the Farmer's Delight states are the tail of
// every per-state table and are laid out afresh, so nothing else moves.
use self::nbt::Tag;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

const NS: &str = "farmersdelight:";
/// Part of the content root's stamp: a change here builds the root again.
pub const VERSION: &str = "farmersdelight tables 4";
const RICE_BLOCK: &str = "farmersdelight:rice";
/// `ModEffects.NOURISHMENT`, which the game has no effect for yet: shown in
/// the tooltips only.
const NOURISHMENT: &str = "farmersdelight:nourishment";
/// `FoodValues`' effects of the mod's foods: (item, effect, duration ticks,
/// amplifier, probability, visible), and whether `ConsumableItem` lists them
/// in the tooltip (`hasFoodEffectTooltip`; plain `Item`s never do).
const FOOD_EFFECTS: &[(&str, &str, i32, i32, f32, bool, bool)] = &[
    ("apple_cider", "minecraft:absorption", 1200, 0, 1.0, true, true),
    ("wheat_dough", "minecraft:hunger", 600, 0, 0.3, true, false),
    ("raw_pasta", "minecraft:hunger", 600, 0, 0.3, true, false),
    ("chicken_cuts", "minecraft:hunger", 600, 0, 0.3, true, false),
    ("cake_slice", "minecraft:speed", 400, 0, 1.0, false, true),
    ("apple_pie_slice", "minecraft:speed", 600, 0, 1.0, false, true),
    ("sweet_berry_cheesecake_slice", "minecraft:speed", 600, 0, 1.0, false, true),
    ("chocolate_pie_slice", "minecraft:speed", 600, 0, 1.0, false, true),
    ("pumpkin_pie_slice", "minecraft:speed", 600, 0, 1.0, false, true),
    ("glow_berry_custard", "minecraft:glowing", 100, 0, 1.0, true, true),
    ("fruit_salad", "minecraft:regeneration", 100, 0, 1.0, true, true),
    ("mixed_salad", "minecraft:regeneration", 100, 0, 1.0, true, true),
    ("nether_salad", "minecraft:nausea", 240, 0, 0.3, true, false),
    ("cooked_rice", NOURISHMENT, 600, 0, 1.0, false, true),
    ("bone_broth", NOURISHMENT, 1200, 0, 1.0, false, true),
    ("beef_stew", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("chicken_soup", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("vegetable_soup", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("fish_stew", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("fried_rice", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("pumpkin_soup", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("baked_cod_stew", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("noodle_soup", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("onion_soup", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("bacon_and_eggs", NOURISHMENT, 1200, 0, 1.0, false, true),
    ("pasta_with_meatballs", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("pasta_with_mutton_chop", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("mushroom_rice", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("roasted_mutton_chops", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("vegetable_noodles", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("steak_and_potatoes", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("ratatouille", NOURISHMENT, 1200, 0, 1.0, false, true),
    ("squid_ink_pasta", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("grilled_salmon", NOURISHMENT, 3600, 0, 1.0, false, true),
    ("roast_chicken", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("stuffed_pumpkin", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("honey_glazed_ham", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("shepherds_pie", NOURISHMENT, 6000, 0, 1.0, false, true),
    ("gleaming_salad", NOURISHMENT, 6000, 0, 1.0, false, true),
];
/// Harmful `MobEffect`s (`MobEffectCategory.HARMFUL`: red in a tooltip).
const HARMFUL: &[&str] = &["minecraft:hunger", "minecraft:nausea"];
/// `MobEffect.createModifiers` of the effects above with attribute
/// modifiers, at amplifier 0: (effect, `attribute.modifier.plus.<op>`,
/// amount as `ATTRIBUTE_MODIFIER_FORMAT` shows it, attribute name key).
const EFFECT_MODIFIERS: &[(&str, u8, &str, &str)] = &[
    ("minecraft:speed", 2, "20", "attribute.name.movement_speed"),
    ("minecraft:absorption", 0, "4", "attribute.name.max_absorption"),
];
/// `DogFoodItem.EFFECTS`, given to the tamed animal it is fed to.
const DOG_FOOD_EFFECTS: &[&str] = &["minecraft:speed", "minecraft:strength", "minecraft:resistance"];
/// `HorseFeedItem.EFFECTS`: (effect, amplifier), five minutes each.
const HORSE_FEED_EFFECTS: &[(&str, i32)] = &[("minecraft:speed", 1), ("minecraft:jump_boost", 0)];
/// `PlaceableItem`s: their tooltip says so (`TextUtils.PLACEABLE`).
const PLACEABLE: &[&str] = &[
    "apple_pie", "sweet_berry_cheesecake", "chocolate_pie", "roast_chicken_block",
    "stuffed_pumpkin_block", "honey_glazed_ham_block", "shepherds_pie_block",
    "gleaming_salad_block", "rice_roll_medley_block",
];
/// `TooltipEvents.addTooltipToVanillaSoups`: `FoodValues.VANILLA_SOUP_EFFECTS`.
const VANILLA_SOUPS: &[(&str, i32)] = &[
    ("minecraft:mushroom_stew", 3600),
    ("minecraft:beetroot_soup", 3600),
    ("minecraft:rabbit_stew", 6000),
];
/// The cooking pot's menu type.
pub const MENU: &str = "farmersdelight:cooking_pot";
/// The stove's damage type.
const STOVE_BURN: &str = "farmersdelight:stove_burn";
/// `ModItems.CREATIVE_TAB_ITEMS`: every `registerWithTab` item, in the order
/// `ModItems` registers them (the tab's `displayItems`).
const TAB_ITEMS: &[&str] = &[
    "stove", "cooking_pot", "skillet", "cutting_board", "wooden_basket", "bamboo_basket",
    "carrot_crate", "potato_crate", "beetroot_crate", "cabbage_crate", "tomato_crate",
    "onion_crate", "rice_bale", "rice_bag", "straw_bale", "safety_net", "oak_cabinet",
    "spruce_cabinet", "birch_cabinet", "jungle_cabinet", "acacia_cabinet", "dark_oak_cabinet",
    "mangrove_cabinet", "cherry_cabinet", "bamboo_cabinet", "crimson_cabinet", "warped_cabinet",
    "tatami", "full_tatami_mat", "half_tatami_mat", "canvas_rug", "rope_fence",
    "rope_fence_gate", "organic_compost", "rich_soil", "rich_soil_farmland", "rope",
    "canvas_sign", "hanging_canvas_sign", "white_canvas_sign", "white_hanging_canvas_sign",
    "light_gray_canvas_sign", "light_gray_hanging_canvas_sign", "gray_canvas_sign",
    "gray_hanging_canvas_sign", "black_canvas_sign", "black_hanging_canvas_sign",
    "brown_canvas_sign", "brown_hanging_canvas_sign", "red_canvas_sign",
    "red_hanging_canvas_sign", "orange_canvas_sign", "orange_hanging_canvas_sign",
    "yellow_canvas_sign", "yellow_hanging_canvas_sign", "lime_canvas_sign",
    "lime_hanging_canvas_sign", "green_canvas_sign", "green_hanging_canvas_sign",
    "cyan_canvas_sign", "cyan_hanging_canvas_sign", "light_blue_canvas_sign",
    "light_blue_hanging_canvas_sign", "blue_canvas_sign", "blue_hanging_canvas_sign",
    "purple_canvas_sign", "purple_hanging_canvas_sign", "magenta_canvas_sign",
    "magenta_hanging_canvas_sign", "pink_canvas_sign", "pink_hanging_canvas_sign",
    "flint_knife", "iron_knife", "diamond_knife", "netherite_knife", "golden_knife", "straw",
    "canvas", "tree_bark", "sandy_shrub", "wild_cabbages", "wild_onions", "wild_tomatoes",
    "wild_carrots", "wild_potatoes", "wild_beetroots", "wild_rice", "brown_mushroom_colony",
    "red_mushroom_colony", "cabbage", "tomato", "onion", "rice_panicle", "rice",
    "cabbage_seeds", "tomato_seeds", "rotten_tomato", "fried_egg", "milk_bottle", "hot_cocoa",
    "apple_cider", "melon_juice", "tomato_sauce", "wheat_dough", "raw_pasta", "pumpkin_slice",
    "cabbage_leaf", "minced_beef", "beef_patty", "chicken_cuts", "cooked_chicken_cuts",
    "bacon", "cooked_bacon", "cod_slice", "cooked_cod_slice", "salmon_slice",
    "cooked_salmon_slice", "mutton_chops", "cooked_mutton_chops", "ham", "smoked_ham",
    "pie_crust", "apple_pie", "sweet_berry_cheesecake", "chocolate_pie", "cake_slice",
    "apple_pie_slice", "sweet_berry_cheesecake_slice", "chocolate_pie_slice",
    "pumpkin_pie_slice", "sweet_berry_cookie", "honey_cookie", "melon_popsicle",
    "glow_berry_custard", "fruit_salad", "mixed_salad", "nether_salad", "barbecue_stick",
    "egg_sandwich", "chicken_sandwich", "hamburger", "bacon_sandwich", "mutton_wrap",
    "dumplings", "stuffed_potato", "cabbage_rolls", "salmon_roll", "cod_roll", "kelp_roll",
    "kelp_roll_slice", "cooked_rice", "bone_broth", "beef_stew", "chicken_soup",
    "vegetable_soup", "fish_stew", "fried_rice", "pumpkin_soup", "baked_cod_stew",
    "noodle_soup", "onion_soup", "bacon_and_eggs", "pasta_with_meatballs",
    "pasta_with_mutton_chop", "mushroom_rice", "roasted_mutton_chops", "vegetable_noodles",
    "steak_and_potatoes", "ratatouille", "squid_ink_pasta", "grilled_salmon",
    "roast_chicken_block", "roast_chicken", "stuffed_pumpkin_block", "stuffed_pumpkin",
    "honey_glazed_ham_block", "honey_glazed_ham", "shepherds_pie_block", "shepherds_pie",
    "gleaming_salad_block", "gleaming_salad", "rice_roll_medley_block", "dog_food",
    "horse_feed",
];

/// The blocks with a block entity in Farmer's Delight (`ModBlockEntityTypes`):
/// removing one goes through the block entity's removal.
fn has_block_entity(name: &str) -> bool {
    let path = name.strip_prefix(NS).unwrap_or(name);
    path == "cutting_board"
        || path == "cooking_pot"
        || path == "skillet"
        || path == "stove"
        || path.ends_with("_basket")
        || path.ends_with("_cabinet")
        // Every canvas sign (`canvas_wall_sign` too).
        || (path.contains("canvas") && path.ends_with("sign"))
}
type Bx = [f64; 6];

/// One generated state.
struct St {
    block: usize,
    props: Vec<(String, String)>,
    o: Value, // the oracle's state facts
    outline: Vec<Bx>,
    collision: Vec<Bx>,
    occlusion: Vec<Bx>,
    waterlogged: bool,
}

/// One Farmer's Delight block.
struct Bl {
    name: String,
    /// Property names (sorted) and their values in Java order.
    props: Vec<(String, Vec<String>)>,
    oracle: Value,
    first: usize,
    count: usize,
    default: usize,
    /// The block's first row in the old overlay: what the template gave it.
    old_first: usize,
}

fn rd(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
        .map_err(|e| format!("{}: {e}", path.display()))
}
fn wr(path: &Path, v: &Value) -> Result<(), String> {
    fs::write(path, serde_json::to_vec(v).unwrap()).map_err(|e| format!("{}: {e}", path.display()))
}
fn boxes(v: &Value) -> Vec<Bx> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|b| {
            let a = b.as_array()?;
            let f = |i: usize| a.get(i).and_then(Value::as_f64);
            Some([f(0)?, f(1)?, f(2)?, f(3)?, f(4)?, f(5)?])
        })
        .collect()
}

// ---------------------------------------------------------------- shapes

/// A shape as Horizon's tables hold it: per-axis coordinates and the
/// occupancy of the cells between them, `(x * sy + y) * sz + z`.
struct Grid {
    c: [Vec<f64>; 3],
    occ: Vec<bool>,
}
impl Grid {
    fn new(bs: &[Bx]) -> Grid {
        let mut c: [Vec<f64>; 3] = Default::default();
        for b in bs {
            for a in 0..3 {
                c[a].push(b[a]);
                c[a].push(b[a + 3]);
            }
        }
        for v in &mut c {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            v.dedup();
        }
        let s = [c[0].len() - 1, c[1].len() - 1, c[2].len() - 1];
        let mut occ = vec![false; s[0] * s[1] * s[2]];
        for x in 0..s[0] {
            for y in 0..s[1] {
                for z in 0..s[2] {
                    let m = [
                        (c[0][x] + c[0][x + 1]) / 2.0,
                        (c[1][y] + c[1][y + 1]) / 2.0,
                        (c[2][z] + c[2][z + 1]) / 2.0,
                    ];
                    occ[(x * s[1] + y) * s[2] + z] = bs.iter().any(|b| {
                        (0..3).all(|a| b[a] <= m[a] && m[a] <= b[a + 3])
                    });
                }
            }
        }
        Grid { c, occ }
    }
    fn sizes(&self) -> [usize; 3] {
        [self.c[0].len() - 1, self.c[1].len() - 1, self.c[2].len() - 1]
    }
    fn at(&self, x: isize, y: isize, z: isize) -> bool {
        let s = self.sizes();
        if x < 0 || y < 0 || z < 0 || x as usize >= s[0] || y as usize >= s[1] || z as usize >= s[2] {
            return false;
        }
        self.occ[(x as usize * s[1] + y as usize) * s[2] + z as usize]
    }
    fn tag(&self) -> Tag {
        let s = self.sizes();
        let mut min = [i32::MAX; 3];
        let mut max = [i32::MIN; 3];
        for x in 0..s[0] {
            for y in 0..s[1] {
                for z in 0..s[2] {
                    if self.occ[(x * s[1] + y) * s[2] + z] {
                        for (a, v) in [x, y, z].into_iter().enumerate() {
                            min[a] = min[a].min(v as i32);
                            max[a] = max[a].max(v as i32 + 1);
                        }
                    }
                }
            }
        }
        let longs = |v: &Vec<f64>| Tag::LongArray(v.iter().map(|f| f.to_bits() as i64).collect());
        Tag::Compound(vec![
            ("canonical".into(), Tag::Byte(0)),
            ("cube".into(), Tag::Byte(0)),
            ("cube_axes".into(), Tag::Byte(0)),
            ("discrete_max".into(), Tag::IntArray(max.to_vec())),
            ("discrete_min".into(), Tag::IntArray(min.to_vec())),
            ("occupied".into(), Tag::ByteArray(self.occ.iter().map(|&b| b as i8).collect())),
            ("x".into(), longs(&self.c[0])),
            ("y".into(), longs(&self.c[1])),
            ("z".into(), longs(&self.c[2])),
        ])
    }
    /// `VoxelShape.forAllEdges`: the shape's outline as merged segments.
    fn edges(&self) -> Vec<[f64; 6]> {
        let s = self.sizes();
        let mut out = Vec::new();
        for a in 0..3 {
            let (b, c) = ((a + 1) % 3, (a + 2) % 3);
            for i in 0..=s[b] {
                for j in 0..=s[c] {
                    let mut run: Option<usize> = None;
                    for k in 0..=s[a] {
                        let on = k < s[a] && {
                            let mut n = [false; 4];
                            for (q, (di, dj)) in [(-1, -1), (0, -1), (-1, 0), (0, 0)].into_iter().enumerate() {
                                let mut p = [0isize; 3];
                                p[a] = k as isize;
                                p[b] = i as isize + di;
                                p[c] = j as isize + dj;
                                n[q] = self.at(p[0], p[1], p[2]);
                            }
                            let cnt = n.iter().filter(|&&x| x).count();
                            cnt == 1 || cnt == 3 || (cnt == 2 && n[0] == n[3])
                        };
                        match (on, run) {
                            (true, None) => run = Some(k),
                            (false, Some(st)) => {
                                let mut e = [0.0; 6];
                                e[a] = self.c[a][st];
                                e[a + 3] = self.c[a][k];
                                e[b] = self.c[b][i];
                                e[b + 3] = self.c[b][i];
                                e[c] = self.c[c][j];
                                e[c + 3] = self.c[c][j];
                                out.push(e);
                                run = None;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        out
    }
}

fn is_full(bs: &[Bx]) -> bool {
    !bs.is_empty() && covers(bs, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0])
}
/// Whether the union of `bs` covers `r` (shapes are on a 1/16 grid; sampled
/// at 1/64 steps, which also catches the odd 1/32).
fn covers(bs: &[Bx], r: Bx) -> bool {
    let n = 64.0;
    let steps = |lo: f64, hi: f64| -> Vec<f64> {
        let a = (lo * n).round() as i64;
        let b = (hi * n).round() as i64;
        (a..b).map(|i| (i as f64 + 0.5) / n).collect()
    };
    for x in steps(r[0], r[3]) {
        for y in steps(r[1], r[4]) {
            for z in steps(r[2], r[5]) {
                if !bs.iter().any(|b| b[0] <= x && x <= b[3] && b[1] <= y && y <= b[4] && b[2] <= z && z <= b[5]) {
                    return false;
                }
            }
        }
    }
    true
}
/// `getFaceShape(direction)`: the part of the shape on that face, extruded
/// through the block. Directions in Java order: down, up, north, south, west, east.
fn face(bs: &[Bx], d: usize) -> Vec<Bx> {
    let (axis, pos) = match d {
        0 => (1, false),
        1 => (1, true),
        2 => (2, false),
        3 => (2, true),
        4 => (0, false),
        _ => (0, true),
    };
    let eps = 1e-7;
    bs.iter()
        .filter(|b| if pos { b[axis + 3] >= 1.0 - eps } else { b[axis] <= eps })
        .map(|b| {
            let mut e = *b;
            e[axis] = 0.0;
            e[axis + 3] = 1.0;
            e
        })
        .collect()
}
/// The 18 sturdiness bits: FULL, CENTER and RIGID support per direction.
fn sturdy(support: &[Bx]) -> u32 {
    let px = |v: f64| v / 16.0;
    let mut bits = 0;
    for d in 0..6 {
        let f = face(support, d);
        if f.is_empty() {
            continue;
        }
        if covers(&f, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]) {
            bits |= 1 << d;
        }
        // CENTER: Block.column(2, 0, 10), x/z 7..9, y 0..10.
        if covers(&f, [px(7.0), 0.0, px(7.0), px(9.0), px(10.0), px(9.0)]) {
            bits |= 1 << (6 + d);
        }
        // RIGID: the block minus the column 2..14 through it: a 2-pixel ring.
        let ring = [
            [0.0, 0.0, 0.0, px(2.0), 1.0, 1.0],
            [px(14.0), 0.0, 0.0, 1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0, 1.0, 1.0, px(2.0)],
            [0.0, 0.0, px(14.0), 1.0, 1.0, 1.0],
        ];
        if ring.iter().all(|r| covers(&f, *r)) {
            bits |= 1 << (12 + d);
        }
    }
    bits
}
fn bounds(bs: &[Bx]) -> Option<Bx> {
    let mut r = [f64::MAX, f64::MAX, f64::MAX, f64::MIN, f64::MIN, f64::MIN];
    for b in bs {
        for a in 0..3 {
            r[a] = r[a].min(b[a]);
            r[a + 3] = r[a + 3].max(b[a + 3]);
        }
    }
    (!bs.is_empty()).then_some(r)
}
/// `BlockBehaviour.calculateSolid` (no forced flags).
fn solid(collision: &[Bx]) -> bool {
    bounds(collision).is_some_and(|b| {
        let (x, y, z) = (b[3] - b[0], b[4] - b[1], b[5] - b[2]);
        (x + y + z) / 3.0 >= 0.7291666666666666 || y >= 1.0
    })
}

/// The shape table of one NBT file: new shapes appended, deduplicated.
struct Shapes<'a> {
    list: &'a mut Vec<Tag>,
    empty: i32,
    full: i32,
    seen: HashMap<Vec<u8>, i32>,
}
impl<'a> Shapes<'a> {
    fn new(list: &'a mut Vec<Tag>) -> Result<Self, String> {
        let find = |c: i8| {
            list.iter()
                .position(|t| t.get("canonical") == Some(&Tag::Byte(c)))
                .map(|i| i as i32)
                .ok_or(format!("no canonical shape {c}"))
        };
        let (empty, full) = (find(1)?, find(2)?);
        Ok(Shapes { list, empty, full, seen: HashMap::new() })
    }
    fn id(&mut self, bs: &[Bx]) -> i32 {
        if bs.is_empty() {
            return self.empty;
        }
        if is_full(bs) && bs.iter().all(|b| b.iter().all(|v| (0.0..=1.0).contains(v))) {
            return self.full;
        }
        let tag = Grid::new(bs).tag();
        let key = nbt::write(false, "", &tag);
        if let Some(&i) = self.seen.get(&key) {
            return i;
        }
        self.list.push(tag);
        let i = self.list.len() as i32 - 1;
        self.seen.insert(key, i);
        i
    }
}

// ---------------------------------------------------------- rotations

const H: [&str; 4] = ["north", "east", "south", "west"];
fn rot_dir(v: &str, quarter: usize) -> String {
    match H.iter().position(|d| *d == v) {
        Some(i) => H[(i + quarter) % 4].into(),
        None => v.into(),
    }
}
fn mirror_dir(v: &str, front_back: bool) -> String {
    match (v, front_back) {
        ("north", false) => "south".into(),
        ("south", false) => "north".into(),
        ("east", true) => "west".into(),
        ("west", true) => "east".into(),
        _ => v.into(),
    }
}
/// The properties of a state rotated clockwise by `quarter` turns.
fn rotate(props: &[(String, String)], quarter: usize) -> Vec<(String, String)> {
    let get = |k: &str| props.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    props
        .iter()
        .map(|(k, v)| {
            let nv = match k.as_str() {
                "facing" | "horizontal_facing" => rot_dir(v, quarter),
                "rotation" => ((v.parse::<usize>().unwrap_or(0) + 4 * quarter) % 16).to_string(),
                "axis" if quarter % 2 == 1 => match v.as_str() {
                    "x" => "z".into(),
                    "z" => "x".into(),
                    o => o.into(),
                },
                d if H.contains(&d) => {
                    // The side that turns into this one.
                    let from = H[(H.iter().position(|x| *x == d).unwrap() + 4 - quarter % 4) % 4];
                    get(from).unwrap_or_else(|| v.clone())
                }
                _ => v.clone(),
            };
            (k.clone(), nv)
        })
        .collect()
}
fn mirror(props: &[(String, String)], front_back: bool) -> Vec<(String, String)> {
    let get = |k: &str| props.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    props
        .iter()
        .map(|(k, v)| {
            let nv = match k.as_str() {
                "facing" | "horizontal_facing" => mirror_dir(v, front_back),
                "rotation" => {
                    let r = v.parse::<i32>().unwrap_or(0);
                    let m = if front_back { (8 - r).rem_euclid(16) } else { (16 - r) % 16 };
                    m.to_string()
                }
                d if H.contains(&d) => get(&mirror_dir(d, front_back)).unwrap_or_else(|| v.clone()),
                _ => v.clone(),
            };
            (k.clone(), nv)
        })
        .collect()
}

// --------------------------------------------------------------- tables

/// Java short class names Horizon knows, mapped from 1.21.1's chain.
fn classes(oracle: &Value, known: &BTreeMap<String, Value>) -> Vec<String> {
    let mut out = Vec::new();
    for c in oracle["class"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        if c.starts_with("vectorwing.") {
            continue;
        }
        let short = c.rsplit('.').next().unwrap_or(c);
        let short = match short {
            "BushBlock" => "VegetationBlock",
            s => s,
        };
        if short == "BlockBehaviour" || !known.contains_key(short) {
            continue;
        }
        out.push(short.to_string());
    }
    if out.is_empty() {
        out.push("Block".into());
    }
    out
}

/// The canvas signs' boards. The game draws a sign's board as a block model
/// on vanilla's sign templates (`template_sign_rot_N`, `template_wall_sign`,
/// the hanging ones) with a board texture laid out for them; the mod's own
/// blockstates point at a particle-only model, its `CanvasSignRenderer`
/// drawing the entity model with `entity/signs/[hanging/]canvas*.png`. Each
/// canvas sign gets vanilla's blockstate and models, on its entity texture
/// moved into the templates' layout.
fn canvas_signs(dir: &Path, blocks: &[Bl]) -> Result<(), String> {
    let put = |rel: String, bytes: Vec<u8>| -> Result<(), String> {
        let p = dir.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        // The root's file may be a link to the mod's own: replaced, not written through.
        let _ = fs::remove_file(&p);
        fs::write(&p, bytes).map_err(|e| format!("{}: {e}", p.display()))
    };
    let model = |parent: String, texture: &str| {
        serde_json::to_vec_pretty(&json!({
            "parent": parent,
            "textures": { "all": format!("farmersdelight:block/{texture}"), "particle": "minecraft:block/spruce_planks" },
        }))
        .unwrap()
    };
    let mut repacked = std::collections::HashSet::new();
    for b in blocks {
        let Some(path) = b.name.strip_prefix(NS) else { continue };
        let Some((kind, color)) = ["wall_hanging_canvas_sign", "hanging_canvas_sign", "canvas_wall_sign", "canvas_sign"]
            .iter()
            .find_map(|k| path.strip_suffix(k).map(|c| (*k, c.trim_end_matches('_'))))
        else {
            continue;
        };
        let hanging = kind.contains("hanging");
        let suffix = if color.is_empty() { String::new() } else { format!("_{color}") };
        let texture = format!("{}canvas_sign{suffix}", if hanging { "hanging_" } else { "" });
        if repacked.insert(texture.clone()) {
            let src = dir.join(format!(
                "assets/farmersdelight/textures/entity/signs/{}canvas{suffix}.png",
                if hanging { "hanging/" } else { "" }
            ));
            put(format!("assets/farmersdelight/textures/block/{texture}.png"), repack_sign(&src, hanging)?)?;
            // The editor's picture, where the GUI atlas reads it by the sign's
            // wood type (`canvas[_color]`): `HangingCanvasSignEditScreen`'s own
            // `gui/hanging_signs` texture; `CanvasSignEditScreen`'s sign
            // model, flat as vanilla's `gui/signs` pictures are.
            let gui = format!("assets/minecraft/textures/gui/{}/canvas{suffix}.png", if hanging { "hanging_signs" } else { "signs" });
            let picture = if hanging {
                let from = dir.join(format!("assets/farmersdelight/textures/gui/hanging_signs/canvas{suffix}.png"));
                fs::read(&from).map_err(|e| format!("{}: {e}", from.display()))?
            } else {
                gui_sign(&src)?
            };
            put(gui, picture)?;
        }
        let facing = |m: &str| {
            json!({
                "facing=south": { "model": m },
                "facing=west": { "model": m, "y": 90 },
                "facing=north": { "model": m, "y": 180 },
                "facing=east": { "model": m, "y": 270 },
            })
        };
        let rotations = |prefix: &str, model: &str, variants: &mut Map<String, Value>| {
            for r in 0..16 {
                let mut v = json!({ "model": format!("farmersdelight:block/{model}_rot_{}", r % 4) });
                if r >= 4 {
                    v["y"] = json!(r / 4 * 90);
                }
                variants.insert(format!("{prefix}rotation={r}"), v);
            }
        };
        let variants = match kind {
            "canvas_sign" => {
                for n in 0..4 {
                    put(format!("assets/farmersdelight/models/block/{path}_rot_{n}.json"),
                        model(format!("minecraft:block/template_sign_rot_{n}"), &texture))?;
                }
                let mut v = Map::new();
                rotations("", path, &mut v);
                Value::Object(v)
            }
            "hanging_canvas_sign" => {
                for n in 0..4 {
                    put(format!("assets/farmersdelight/models/block/{path}_rot_{n}.json"),
                        model(format!("minecraft:block/template_hanging_sign_rot_{n}"), &texture))?;
                    put(format!("assets/farmersdelight/models/block/{path}_attached_rot_{n}.json"),
                        model(format!("minecraft:block/template_attached_hanging_sign_rot_{n}"), &texture))?;
                }
                let mut v = Map::new();
                rotations("attached=false,", path, &mut v);
                rotations("attached=true,", &format!("{path}_attached"), &mut v);
                Value::Object(v)
            }
            _ => {
                let template = if hanging { "template_wall_hanging_sign" } else { "template_wall_sign" };
                put(format!("assets/farmersdelight/models/block/{path}.json"),
                    model(format!("minecraft:block/{template}"), &texture))?;
                facing(&format!("farmersdelight:block/{path}"))
            }
        };
        put(format!("assets/farmersdelight/blockstates/{path}.json"),
            serde_json::to_vec_pretty(&json!({ "variants": variants })).unwrap())?;
    }
    Ok(())
}

/// A sign's entity texture (`SignRenderer`'s / `HangingSignRenderer`'s model
/// layout, 64x32) moved into the 32x32 layout the game's sign templates map:
/// each face's rectangle copied to where the template reads it.
fn repack_sign(src: &Path, hanging: bool) -> Result<Vec<u8>, String> {
    sign_texture(src, if hanging { HANGING_LAYOUT } else { STANDING_LAYOUT }, [32, 32])
}
/// `SignEditScreen`'s picture of a standing sign (`gui/signs/<wood>`, 24x26):
/// the board's front over the stick's.
fn gui_sign(src: &Path) -> Result<Vec<u8>, String> {
    const GUI: &[[usize; 6]] = &[[0, 0, 24, 12, 2, 2], [11, 12, 13, 26, 2, 16]];
    sign_texture(src, GUI, [24, 26])
}
// (to x0, y0, x1, y1, from x0, y0), in the 64x32 entity texture's pixels.
const STANDING_LAYOUT: &[[usize; 6]] = &[
    [0, 0, 24, 2, 2, 0],     // board top
    [0, 2, 24, 14, 2, 2],    // board front
    [24, 2, 26, 14, 26, 2],  // board side
    [0, 16, 24, 28, 28, 2],  // board back
    [24, 16, 26, 28, 0, 2],  // board side
    [0, 28, 24, 30, 26, 0],  // board bottom
    [28, 0, 30, 14, 2, 16],  // stick front
    [30, 0, 32, 14, 4, 16],  // stick side
    [28, 16, 30, 30, 6, 16], // stick back
    [30, 16, 32, 30, 0, 16], // stick side
    [28, 30, 30, 32, 4, 14], // stick bottom
];
const HANGING_LAYOUT: &[[usize; 6]] = &[
    [0, 0, 16, 4, 4, 0],      // plank top
    [0, 4, 16, 6, 4, 4],      // plank front
    [16, 4, 20, 6, 20, 4],    // plank side
    [0, 7, 16, 9, 24, 4],     // plank back
    [16, 7, 20, 9, 0, 4],     // plank side
    [0, 9, 16, 13, 20, 0],    // plank bottom
    [20, 0, 32, 6, 14, 6],    // V chains
    [22, 7, 25, 13, 0, 6],    // chain
    [28, 8, 31, 12, 6, 7],    // short chain
    [2, 14, 16, 16, 2, 12],   // board top
    [2, 16, 16, 26, 2, 14],   // board front
    [16, 16, 18, 26, 16, 14], // board side
    [18, 16, 32, 26, 18, 14], // board back
    [0, 16, 2, 26, 0, 14],    // board side
    [2, 26, 16, 28, 16, 12],  // board bottom
];
/// A new `size` texture of `layout`'s rectangles of the 64x32 sign texture
/// at `src` (scaled with it).
fn sign_texture(src: &Path, layout: &[[usize; 6]], size: [usize; 2]) -> Result<Vec<u8>, String> {
    let err = |e: &dyn std::fmt::Display| format!("{}: {e}", src.display());
    let file = fs::File::open(src).map_err(|e| err(&e))?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| err(&e))?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or_else(|| err(&"too large"))?];
    let info = reader.next_frame(&mut buf).map_err(|e| err(&e))?;
    let (w, h) = (info.width as usize, info.height as usize);
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        _ => 1,
    };
    let at = |x: usize, y: usize| -> [u8; 4] {
        let i = (y * w + x) * channels;
        match channels {
            4 => [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]],
            3 => [buf[i], buf[i + 1], buf[i + 2], 255],
            2 => [buf[i], buf[i], buf[i], buf[i + 1]],
            _ => [buf[i], buf[i], buf[i], 255],
        }
    };
    let s = (w / 64).max(1);
    if w < 64 * s || h < 32 * s {
        return Err(err(&"not a 64x32 sign texture"));
    }
    let (out_w, out_h) = (size[0] * s, size[1] * s);
    let mut out = vec![0u8; out_w * out_h * 4];
    for r in layout {
        for y in r[1] * s..r[3] * s {
            for x in r[0] * s..r[2] * s {
                let px = at(r[4] * s + x - r[0] * s, r[5] * s + y - r[1] * s);
                out[(y * out_w + x) * 4..][..4].copy_from_slice(&px);
            }
        }
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, out_w as u32, out_h as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| err(&e))?;
        writer.write_image_data(&out).map_err(|e| err(&e))?;
    }
    Ok(bytes)
}

pub struct Report {
    pub blocks: usize,
    pub states_before: usize,
    pub states_after: usize,
    pub shapes_added: usize,
}

pub fn apply(dir: &Path, oracle: &Value) -> Result<Report, String> {
    let oblocks = oracle["blocks"].as_object().ok_or("oracle: no blocks")?;

    // -- facts.json: where the Farmer's Delight tail starts and its blocks.
    let facts_path = dir.join("facts.json");
    let mut facts = rd(&facts_path)?;
    let rows = facts["states"].as_array().ok_or("facts: no states")?;
    let first = rows
        .iter()
        .position(|r| r[0].as_str().is_some_and(|n| n.starts_with(NS)))
        .ok_or("no Farmer's Delight states in the overlay")?;
    let states_before = rows.len();
    let mut blocks: Vec<Bl> = Vec::new();
    for (i, r) in rows.iter().enumerate().skip(first) {
        let name = r[0].as_str().ok_or("facts: a state without a name")?;
        if !name.starts_with(NS) {
            return Err(format!("{name} after the Farmer's Delight states: another mod's blocks"));
        }
        if blocks.last().is_none_or(|b| b.name != name) {
            let o = oblocks.get(name).ok_or(format!("{name} is not in the oracle"))?.clone();
            let mut props: Vec<(String, Vec<String>)> = o["properties"]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(k, v)| {
                    (k.clone(), v.as_array().into_iter().flatten().filter_map(|s| s.as_str().map(String::from)).collect())
                })
                .collect();
            // RiceBlock.getFluidState: a water source in every state. The game
            // reads a block's water from `waterlogged`, so the rice's states
            // carry it, always true (the mod's own state keys leave it out).
            if name == RICE_BLOCK {
                props.push(("waterlogged".into(), vec!["true".into()]));
            }
            props.sort_by(|a, b| a.0.cmp(&b.0));
            blocks.push(Bl { name: name.into(), props, oracle: o, first: 0, count: 0, default: 0, old_first: i });
        }
    }

    // -- the new states, Java order: the first property varies slowest.
    let mut states: Vec<St> = Vec::new();
    for (bi, b) in blocks.iter_mut().enumerate() {
        b.first = first + states.len();
        let mut combos: Vec<Vec<(String, String)>> = vec![vec![]];
        for (k, vs) in &b.props {
            combos = combos
                .into_iter()
                .flat_map(|c| vs.iter().map(move |v| {
                    let mut c = c.clone();
                    c.push((k.clone(), v.clone()));
                    c
                }))
                .collect();
        }
        let default_key = b.oracle["default"].as_str().unwrap_or("").to_string();
        for (ci, props) in combos.into_iter().enumerate() {
            let key = props
                .iter()
                .filter(|(k, _)| !(b.name == RICE_BLOCK && k == "waterlogged"))
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(",");
            let o = b.oracle["states"].get(&key).cloned().ok_or(format!("{}: no oracle state {key}", b.name))?;
            if key == default_key {
                b.default = b.first + ci;
            }
            let outline = boxes(&o["outline"]);
            let collision = boxes(&o["collision"]);
            let can_occlude = o["can_occlude"].as_bool().unwrap_or(false);
            // getOcclusionShape: the outline, unless the block has its own
            // (RopeFenceBlock.POST, BasketBlock.RENDER_SHAPE).
            let occlusion = if !can_occlude {
                vec![]
            } else if b.name == "farmersdelight:rope_fence" {
                vec![[7.0 / 16.0, 0.0, 7.0 / 16.0, 9.0 / 16.0, 1.0, 9.0 / 16.0]]
            } else if b.name.ends_with("_basket") {
                vec![[1.0 / 16.0, 1.0 / 16.0, 1.0 / 16.0, 15.0 / 16.0, 15.0 / 16.0, 15.0 / 16.0]]
            } else {
                outline.clone()
            };
            // `RiceBlock.getFluidState` is always a water source, as is
            // `WildRiceBlock`'s lower half: they hold the water they stand in.
            let water_plant = b.name == "farmersdelight:rice"
                || (b.name == "farmersdelight:wild_rice" && props.iter().any(|(k, v)| k == "half" && v == "lower"));
            let waterlogged = water_plant || props.iter().any(|(k, v)| k == "waterlogged" && v == "true");
            states.push(St { block: bi, props, o, outline, collision, occlusion, waterlogged });
        }
        b.count = b.first + 0;
        b.count = states.len() + first - b.first;
    }
    let total = first + states.len();
    let id_of: HashMap<(usize, Vec<(String, String)>), usize> =
        states.iter().enumerate().map(|(i, s)| ((s.block, s.props.clone()), first + i)).collect();

    // -- facts.json
    {
        let rows = facts["states"].as_array_mut().unwrap();
        rows.truncate(first);
        for s in &states {
            let b = &blocks[s.block];
            let desc = if s.props.is_empty() {
                format!("Block{{{}}}", b.name)
            } else {
                format!("Block{{{}}}[{}]", b.name, s.props.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(","))
            };
            let mut st = Map::new();
            if !s.props.is_empty() {
                st.insert("properties".into(), Value::Object(s.props.iter().map(|(k, v)| (k.clone(), json!(v))).collect()));
            }
            st.insert("id".into(), json!(b.name));
            rows.push(json!([b.name, false, s.waterlogged, desc, st]));
        }
        let defaults = facts["defaults"].as_object_mut().ok_or("facts: no defaults")?;
        for b in &blocks {
            defaults.insert(b.name.clone(), json!(b.default));
        }
        wr(&facts_path, &facts)?;
    }

    // -- gameplay-facts.json
    let gp_path = dir.join("gameplay-facts.json");
    let mut gp = rd(&gp_path)?;
    let items = rd(&dir.join("items.json"))?;
    let item_of = |b: &Bl| -> String {
        let i = b.oracle["item"].as_str().unwrap_or("minecraft:air");
        if items.get(i).is_some() { i.into() } else { "minecraft:air".into() }
    };
    let mut derived = Vec::with_capacity(states.len());
    {
        let rows = gp["states"].as_array_mut().ok_or("gameplay: no states")?;
        let old: Vec<Value> = rows[first..].to_vec();
        rows.truncate(first);
        for (i, s) in states.iter().enumerate() {
            let b = &blocks[s.block];
            let tpl = &old[b.old_first - first];
            let o = &s.o;
            let can_occlude = o["can_occlude"].as_bool().unwrap_or(false);
            let full_collision = is_full(&s.collision);
            let solid_render = can_occlude && is_full(&s.occlusion);
            let skylight_down = !is_full(&s.outline) && !s.waterlogged;
            let dampening = if solid_render { 15 } else if skylight_down { 0 } else { 1 };
            let snd = &o["sound"];
            let ev = |k: &str| json!({"id": snd[k].as_str().unwrap_or("minecraft:block.stone.break")});
            let cls = b.oracle["class"].to_string();
            let plant = cls.contains("BushBlock") || cls.contains("CropBlock");
            let pie = cls.contains("PieBlock") || cls.contains("FeastBlock");
            let push = if plant || pie { json!("POPPED") } else { tpl["push"].clone() };
            let edges: Vec<Value> = if s.outline.is_empty() {
                vec![]
            } else {
                Grid::new(&s.outline).edges().iter().map(|e| json!(e)).collect()
            };
            let row = json!({
                "id": first + i,
                "emission": o["emission"],
                "dampening": dampening,
                "skylight_down": skylight_down,
                "shape_occlusion": false,
                "can_occlude": can_occlude,
                "solid_render": solid_render,
                "hardness": o["hardness"],
                "requires_tool": o["requires_tool"],
                "replaceable": o["replaceable"],
                // The mod's crops take their random ticks through the
                // random_tick hook point, which grows them their own way.
                "random_ticks": o["random_ticks"],
                "block_entity": has_block_entity(&b.name),
                "render_shape": "MODEL",
                "liquid": false,
                "ignited_by_lava": tpl["ignited_by_lava"],
                "push": push,
                "offset": false,
                "full_collision": full_collision,
                "map_color": o["map_color"],
                "outline_offset": false,
                "outline": s.outline,
                "outline_edges": edges,
                "friction": b.oracle["friction"],
                "speed_factor": b.oracle["speed_factor"],
                "jump_factor": b.oracle["jump_factor"],
                "explosion_resistance": b.oracle["explosion_resistance"],
                "item": item_of(b),
                "sound": {
                    "volume": snd["volume"], "pitch": snd["pitch"],
                    "break": ev("break"), "step": ev("step"), "place": ev("place"),
                    "hit": ev("hit"), "fall": ev("fall"),
                },
            });
            derived.push((full_collision, solid_render, can_occlude));
            rows.push(row);
        }
        wr(&gp_path, &gp)?;
    }

    // -- block-behaviours.json
    let bh_path = dir.join("block-behaviours.json");
    let mut bh = rd(&bh_path)?;
    let fluids: Vec<String> = bh["fluids"].as_array().ok_or("behaviours: no fluids")?
        .iter().filter_map(|f| f.as_str().map(String::from)).collect();
    let water = fluids.iter().position(|f| f == "minecraft:water").ok_or("no water fluid")?;
    {
        let rows = bh["states"].as_array_mut().ok_or("behaviours: no states")?;
        let old: Vec<Value> = rows[first..].to_vec();
        rows.truncate(first);
        for (i, s) in states.iter().enumerate() {
            let b = &blocks[s.block];
            let tpl = &old[b.old_first - first];
            let (full_collision, _, _) = derived[i];
            let mut flags = 0u32;
            if solid(&s.collision) {
                flags |= 1;
            }
            if full_collision {
                flags |= 4; // a redstone conductor
            }
            if s.collision.is_empty() {
                flags |= 32;
            }
            let (fk, fa) = if s.waterlogged { (water, 8) } else { (0, 0) };
            rows.push(json!([sturdy(&s.collision), flags, fk, fa, false, 0, 0, tpl[7], tpl[8]]));
        }
        // Per-block entries: the oracle's vanilla class chain.
        let blist = bh["blocks"].as_array_mut().ok_or("behaviours: no blocks")?;
        let mut known: BTreeMap<String, Value> = BTreeMap::new();
        for e in blist.iter() {
            if e["name"].as_str().is_some_and(|n| n.starts_with("minecraft:")) {
                if let Some(head) = e["class"][0].as_str() {
                    known.entry(head.to_string()).or_insert_with(|| e.clone());
                }
                for c in e["class"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                    known.entry(c.to_string()).or_insert(Value::Null);
                }
            }
        }
        // RopeFenceGateBlock: NeoForge's FenceGateBlock(properties, open, close),
        // the oak gate's wood type with the mod's own gate sounds.
        let oak_gate_type = blist
            .iter()
            .find(|e| e["name"] == "minecraft:oak_fence_gate")
            .map(|e| e["fields"]["type"].clone());
        for e in blist.iter_mut() {
            let Some(name) = e["name"].as_str().map(String::from) else { continue };
            let Some(b) = blocks.iter().find(|b| b.name == name) else { continue };
            // RicePaniclesBlock.mayPlaceOn is the rice below it, not
            // farmland: the mod's runtime keeps it, not CropBlock's rules.
            let chain = if name == "farmersdelight:rice_panicles" {
                vec!["Block".to_string()]
            } else {
                classes(&b.oracle, &known)
            };
            let donor = known.get(&chain[0]).filter(|d| !d.is_null());
            let mut fields = json!({});
            if name == "farmersdelight:rope_fence_gate"
                && let Some(mut wood) = oak_gate_type.clone().filter(|t| !t.is_null())
            {
                wood["fenceGateOpen"] = json!("farmersdelight:block.rope_fence_gate.open");
                wood["fenceGateClose"] = json!("farmersdelight:block.rope_fence_gate.close");
                fields["type"] = wood;
            }
            *e = json!({
                "name": name,
                "class": chain,
                "interfaces": donor.map(|d| d["interfaces"].clone()).unwrap_or(json!(["ItemLike", "FeatureElement"])),
                "fields": fields,
                "dynamic_shape": false,
                "item": item_of(b),
            });
        }
        // The block items' bindings (what a use of the item places): each
        // Farmer's Delight item places the block the mod's item names
        // (`ItemNameBlockItem` crops: cabbage seeds, tomato seeds, onion,
        // rice), a sign its standing block or its wall block
        // (`SignItem`, `HangingSignItem`: `StandingAndWallBlockItem`).
        let mut by_item: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for b in &blocks {
            let i = item_of(b);
            if i != "minecraft:air" && i.starts_with(NS) {
                by_item.entry(i).or_default().push(b.name.clone());
            }
        }
        if let Some(bindings) = bh["items"].as_object_mut() {
            for (item, names) in by_item {
                // The other blocks of a crop (`tomatoes`, `tomatoes_on_rope`,
                // `rice_panicles`) carry the seed's item but are never placed by it.
                let placed: Vec<&String> = names
                    .iter()
                    .filter(|n| {
                        !matches!(
                            n.as_str(),
                            "farmersdelight:tomatoes" | "farmersdelight:tomatoes_on_rope" | "farmersdelight:rice_panicles"
                        )
                    })
                    .collect();
                let walls: Vec<&&String> = placed.iter().filter(|n| n.contains("wall_")).collect();
                let stands: Vec<&&String> = placed.iter().filter(|n| !n.contains("wall_")).collect();
                let binding = match (stands.as_slice(), walls.as_slice()) {
                    ([block], []) => json!({ "class": ["BlockItem"], "fields": { "block": block } }),
                    ([block], [wall]) => {
                        let hanging = block.contains("hanging");
                        let mut class = vec!["StandingAndWallBlockItem", "BlockItem"];
                        if hanging {
                            class.insert(0, "HangingSignItem");
                        }
                        json!({
                            "class": class,
                            "fields": {
                                "attachmentDirection": if hanging { "UP" } else { "DOWN" },
                                "block": block,
                                "wallBlock": wall,
                            }
                        })
                    }
                    _ => continue,
                };
                bindings.insert(item, binding);
            }
        }
        wr(&bh_path, &bh)?;
    }

    // -- mob-data.json: path type, spawn and support flags, collision top.
    {
        let p = dir.join("mob-data.json");
        let mut md = rd(&p)?;
        let rows = md["states"].as_array_mut().ok_or("mob-data: no states")?;
        rows.truncate(first);
        for (i, s) in states.iter().enumerate() {
            let (full_collision, solid_render, _) = derived[i];
            let st = sturdy(&s.collision);
            let emission = s.o["emission"].as_u64().unwrap_or(0);
            let mut f = 0u16;
            if !full_collision {
                f |= 1 | 4; // pathfindable on land and in the air
            }
            if s.waterlogged {
                f |= 2;
            }
            if st & 2 != 0 && emission < 14 {
                f |= 8 | 16 | 32;
            }
            if full_collision {
                f |= 64;
            }
            if !full_collision && !s.waterlogged {
                f |= 128;
            }
            if st & 2 != 0 {
                f |= 256;
            }
            if solid(&s.collision) {
                f |= 512;
            }
            if solid_render {
                f |= 1024;
            }
            let path = if full_collision { "BLOCKED" } else if s.waterlogged { "WATER" } else { "OPEN" };
            let top = bounds(&s.collision).map_or(0.0, |b| b[4]);
            rows.push(json!([path, f, top]));
        }
        wr(&p, &md)?;
    }

    // -- worldgen/block_flags.json: [is_air, has_fluid]
    {
        let p = dir.join("worldgen/block_flags.json");
        let mut v = rd(&p)?;
        let rows = v.as_array_mut().ok_or("block_flags: not a list")?;
        rows.truncate(first);
        rows.extend(states.iter().map(|s| json!([false, s.waterlogged])));
        wr(&p, &v)?;
    }

    // -- worldgen/blocks/structure-blocks.json: rotations, mirrors, full blocks.
    {
        let p = dir.join("worldgen/blocks/structure-blocks.json");
        let mut v = rd(&p)?;
        let find = |s: &St, props: Vec<(String, String)>| -> usize {
            *id_of.get(&(s.block, props)).unwrap_or(&id_of[&(s.block, s.props.clone())])
        };
        let rot: Vec<Value> = states.iter().map(|s| json!([find(s, rotate(&s.props, 1)), find(s, rotate(&s.props, 2)), find(s, rotate(&s.props, 3))])).collect();
        let mir: Vec<Value> = states.iter().map(|s| json!([find(s, mirror(&s.props, false)), find(s, mirror(&s.props, true))])).collect();
        let full: Vec<Value> = derived.iter().map(|d| json!(d.0)).collect();
        for (key, new) in [("rotate", rot), ("mirror", mir), ("full_block", full)] {
            let a = v[key].as_array_mut().ok_or(format!("structure-blocks: no {key}"))?;
            a.truncate(first);
            a.extend(new);
        }
        v["states"] = json!(total);
        wr(&p, &v)?;
    }

    // -- per-block JSON maps
    {
        let p = dir.join("model-states.json");
        let mut v = rd(&p)?;
        for b in &blocks {
            let mut schema = Map::new();
            for (k, vs) in &b.props {
                let ints: Option<Vec<i64>> = vs.iter().map(|x| x.parse().ok()).collect();
                let bools = vs.iter().all(|x| x == "true" || x == "false");
                schema.insert(k.clone(), match ints {
                    Some(n) if !bools && !n.is_empty() => json!({"integer": [n.iter().min(), n.iter().max()]}),
                    _ => {
                        let mut s = vs.clone();
                        s.sort();
                        json!({"named": s})
                    }
                });
            }
            let st: Vec<Value> = states.iter().filter(|s| blocks[s.block].name == b.name)
                .map(|s| Value::Object(s.props.iter().map(|(k, v)| (k.clone(), json!(v))).collect()))
                .collect();
            v[&b.name] = json!({"schema": schema, "states": st});
        }
        wr(&p, &v)?;

        let p = dir.join("render-definitions.json");
        let mut v = rd(&p)?;
        for b in &blocks {
            let rule = v[&b.name]["rule"].clone();
            let can = b.oracle["states"][b.oracle["default"].as_str().unwrap_or("")]["can_occlude"].clone();
            v[&b.name] = json!({
                "rule": if rule.is_null() { json!("never") } else { rule },
                "states": (b.first..b.first + b.count).collect::<Vec<_>>(),
                "default": b.default,
                "can_occlude": can,
                "bars_tag": false,
            });
        }
        wr(&p, &v)?;

        let p = dir.join("command-registries.json");
        let mut v = rd(&p)?;
        for b in &blocks {
            v["blocks"][&b.name] = json!(b.props.iter().map(|(k, vs)| json!([k, vs])).collect::<Vec<_>>());
        }
        wr(&p, &v)?;

        let p = dir.join("worldgen/blocks/survival-rules.json");
        let mut v = rd(&p)?;
        for b in &blocks {
            let cls = b.oracle["class"].to_string();
            let rule = if cls.contains("CropBlock") {
                "Crop"
            } else if cls.contains("BushBlock") {
                "TagBelow:minecraft:supports_vegetation"
            } else if cls.contains("PieBlock") || cls.contains("FeastBlock") {
                "SolidBelow"
            } else {
                "Always"
            };
            v[&b.name] = json!(rule);
        }
        wr(&p, &v)?;
    }

    // -- registries the mod adds: its menu type (`ModMenuTypes.COOKING_POT`)
    // and its damage type (`ModDamageTypes.STOVE_BURN`, the mod's generated
    // `damage_type/stove_burn.json` and the damage type tags it joins).
    let p = dir.join("registries.json");
    let mut v = rd(&p)?;
    if let Some(entries) = v["minecraft:menu"]["entries"].as_object_mut()
        && !entries.contains_key(MENU)
    {
        let next = entries
            .values()
            .filter_map(|e| e["protocol_id"].as_i64())
            .max()
            .map_or(0, |m| m + 1);
        entries.insert(MENU.into(), json!({ "protocol_id": next }));
    }
    wr(&p, &v)?;
    let p = dir.join("damage-types.json");
    let mut v = rd(&p)?;
    v[STOVE_BURN] = json!(serde_json::to_string_pretty(&json!({
        "effects": "burning",
        "exhaustion": 0.1,
        "message_id": "farmersdelight.stove",
        "scaling": "when_caused_by_living_non_player",
    }))
    .unwrap());
    wr(&p, &v)?;
    let p = dir.join("damage-catalog.json");
    let mut v = rd(&p)?;
    if let Some(types) = v["types"].as_array_mut() {
        types.retain(|t| t["id"] != STOVE_BURN);
        types.push(json!({
            "id": STOVE_BURN,
            "message_id": "farmersdelight.stove",
            "exhaustion": format!("{:08x}", 0.1f32.to_bits()),
            "scaling": "when_caused_by_living_non_player",
            "effects": "burning",
            "death_message_type": "default",
        }));
    }
    wr(&p, &v)?;
    let p = dir.join("damage-tags.json");
    let mut v = rd(&p)?;
    for tag in [
        "minecraft:burn_from_stepping",
        "minecraft:is_fire",
        "minecraft:no_knockback",
        "minecraft:panic_environmental_causes",
    ] {
        let mut t: Value = v[tag]
            .as_str()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_else(|| json!({ "values": [] }));
        if let Some(values) = t["values"].as_array_mut()
            && !values.iter().any(|x| x == STOVE_BURN)
        {
            values.push(json!(STOVE_BURN));
        }
        v[tag] = json!(serde_json::to_string_pretty(&t).unwrap());
    }
    wr(&p, &v)?;

    // -- the skillet's item (`SkilletItem`, `ModItems.SKILLET`): one to a
    // stack, `Tiers.IRON`'s 250 uses, enchantability and repair, the
    // attributes of `createAttributes(IRON, 5, -3.1)` (5 + the tier's 2,
    // the attack speed, and one more knockback), a use per hit
    // (`postHurtEnemy`) and per block mined with any hardness (`mineBlock`),
    // no breaking blocks in creative (`canAttackBlock`), and its tooltip line
    // (`TextUtils.PLACEABLE_SNEAKING`, dark grey and italic).
    let p = dir.join("items.json");
    let mut v = rd(&p)?;
    if let Some(skillet) = v.get_mut("farmersdelight:skillet").and_then(Value::as_object_mut) {
        skillet.insert("minecraft:max_stack_size".into(), json!(1));
        skillet.insert("minecraft:max_damage".into(), json!(250));
        skillet.insert("minecraft:damage".into(), json!(0));
        skillet.insert("minecraft:enchantable".into(), json!({ "value": 14 }));
        skillet.insert(
            "minecraft:repairable".into(),
            json!({ "items": "#minecraft:iron_tool_materials" }),
        );
        skillet.insert(
            "minecraft:attribute_modifiers".into(),
            json!([
                { "type": "minecraft:attack_damage", "amount": 7.0,
                  "id": "minecraft:base_attack_damage", "operation": "add_value", "slot": "mainhand" },
                { "type": "minecraft:attack_speed", "amount": -3.0999999046325684,
                  "id": "minecraft:base_attack_speed", "operation": "add_value", "slot": "mainhand" },
                { "type": "minecraft:attack_knockback", "amount": 1.0,
                  "id": "farmersdelight:base_attack_knockback", "operation": "add_value", "slot": "mainhand" },
            ]),
        );
        skillet.insert("minecraft:weapon".into(), json!({}));
        skillet.insert(
            "minecraft:tool".into(),
            json!({ "can_destroy_blocks_in_creative": false, "damage_per_block": 1, "rules": [] }),
        );
    }
    // -- the foods' effects (`FoodValues`) and the lines
    // `ConsumableItem.appendHoverText` adds under the name
    // (`TextUtils.addFoodEffectTooltip`, a drink's own line when
    // `hasCustomTooltip`, `DogFoodItem`'s list of what a wolf gets).
    let effect_name = |id: &str| {
        let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
        json!({ "translate": format!("effect.{ns}.{path}") })
    };
    // MobEffectUtil.formatDuration (StringUtil.formatTickDuration): mm:ss.
    let duration = |ticks: i32| {
        let s = ticks / 20;
        format!("{:02}:{:02}", s / 60, s % 60)
    };
    let mut effects: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    // Item ids (full) to their lines, read by the tooltip hook point.
    let mut lines: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    // SkilletItem.appendHoverText: `TextUtils.PLACEABLE_SNEAKING`.
    lines.entry(format!("{NS}skillet")).or_default().push(json!({
        "translate": "tooltip.farmersdelight.placeable_sneaking", "color": "dark_gray", "italic": true,
    }));
    for &(item, effect, ticks, amplifier, probability, visible, tooltip) in FOOD_EFFECTS {
        if effect != NOURISHMENT {
            effects.entry(item).or_default().push(json!({
                "type": "minecraft:apply_effects",
                "effects": [{ "id": effect, "duration": ticks, "amplifier": amplifier,
                              "show_particles": visible, "show_icon": true }],
                "probability": probability,
            }));
        }
        if !tooltip {
            continue;
        }
        let mut text = effect_name(effect);
        if amplifier > 0 {
            text = json!({ "translate": "potion.withAmplifier",
                           "with": [text, { "translate": format!("potion.potency.{amplifier}") }] });
        }
        if ticks > 20 {
            text = json!({ "translate": "potion.withDuration", "with": [text, duration(ticks)] });
        }
        text["color"] = json!(if HARMFUL.contains(&effect) { "red" } else { "blue" });
        text["italic"] = json!(false);
        let list = lines.entry(format!("{NS}{item}")).or_default();
        list.push(text);
        if let Some(&(_, op, amount, attribute)) = EFFECT_MODIFIERS.iter().find(|m| m.0 == effect) {
            list.push(json!({ "text": "", "italic": false }));
            list.push(json!({ "translate": "potion.whenDrank", "color": "dark_purple", "italic": false }));
            list.push(json!({ "translate": format!("attribute.modifier.plus.{op}"),
                              "with": [amount, { "translate": attribute }],
                              "color": "blue", "italic": false }));
        }
    }
    for item in ["milk_bottle", "hot_cocoa", "melon_juice"] {
        lines.entry(format!("{NS}{item}")).or_default().push(json!({
            "translate": format!("tooltip.farmersdelight.{item}"), "color": "blue", "italic": false,
        }));
    }
    let dog = lines.entry(format!("{NS}dog_food")).or_default();
    dog.push(json!({ "translate": "tooltip.farmersdelight.dog_food.when_feeding", "color": "gray", "italic": false }));
    for effect in DOG_FOOD_EFFECTS {
        dog.push(json!({
            "text": " ",
            "extra": [{ "translate": "potion.withDuration", "with": [effect_name(effect), duration(6000)] }],
            "color": "blue", "italic": false,
        }));
    }
    let horse = lines.entry(format!("{NS}horse_feed")).or_default();
    horse.push(json!({ "translate": "tooltip.farmersdelight.horse_feed.when_feeding", "color": "gray", "italic": false }));
    for &(effect, amplifier) in HORSE_FEED_EFFECTS {
        let mut extra = vec![effect_name(effect)];
        if amplifier > 0 {
            extra.push(json!(" "));
            extra.push(json!({ "translate": format!("potion.potency.{amplifier}") }));
        }
        extra.push(json!(format!(" ({})", duration(6000))));
        horse.push(json!({ "text": " ", "extra": extra, "color": "blue", "italic": false }));
    }
    for item in PLACEABLE {
        lines.entry(format!("{NS}{item}")).or_default().push(json!({
            "translate": "tooltip.farmersdelight.placeable", "color": "dark_gray", "italic": true,
        }));
    }
    // TooltipEvents.addTooltipToVanillaSoups: their Nourishment line.
    for &(item, ticks) in VANILLA_SOUPS {
        lines.entry(item.to_string()).or_default().push(json!({
            "translate": "potion.withDuration",
            "with": [effect_name(NOURISHMENT), duration(ticks)],
            "color": "blue", "italic": false,
        }));
    }
    for (item, list) in effects {
        if let Some(c) = v.get_mut(format!("{NS}{item}")).and_then(Value::as_object_mut) {
            let consumable = c.entry("minecraft:consumable").or_insert_with(|| json!({}));
            consumable["on_consume_effects"] = json!(list);
        }
    }
    wr(&p, &v)?;
    let tooltips = dir.join("farmersdelight-tooltips.json");
    let _ = fs::remove_file(&tooltips);
    wr(&tooltips, &json!(lines))?;

    // -- the creative inventory: `ModCreativeTabs.TAB_FARMERS_DELIGHT` (the
    // stove's icon, `itemGroup.farmersdelight`) after vanilla's tabs, first
    // of NeoForge's second page (top, first column); the search tab lists its
    // items after vanilla's, as `CreativeModeTabs.buildAllTabContents` does.
    let items: Value = rd(&dir.join("items.json"))?;
    let lang: Value = rd(&dir.join("assets/farmersdelight/lang/en_us.json")).unwrap_or(Value::Null);
    let known = |id: &str| items.get(id).is_some() || items["items"].get(id).is_some();
    let stacks: Vec<String> = TAB_ITEMS
        .iter()
        .map(|p| format!("{NS}{p}"))
        .filter(|id| known(id))
        .map(|id| format!("{{count:1,id:\"{id}\"}}"))
        .collect();
    let p = dir.join("creative-inventory.json");
    let mut v = rd(&p)?;
    if let Some(tabs) = v["tabs"].as_array_mut() {
        tabs.retain(|t| t["id"] != "farmersdelight:farmersdelight");
        for t in tabs.iter_mut().filter(|t| t["type"] == "SEARCH") {
            let mut list: Vec<Value> = t["items"].as_array().cloned().unwrap_or_default();
            let n = list.len();
            for s in &stacks {
                if !list.iter().any(|x| x == s) {
                    list.push(json!(s));
                }
            }
            let new: Vec<String> = list[n..]
                .iter()
                .filter_map(|s| s.as_str())
                .filter_map(|s| s.split('"').nth(1))
                .map(str::to_owned)
                .collect();
            t["items"] = json!(list);
            // The search tree's lines: the English name (`item.` or `block.`).
            let name = |id: &str| -> Value {
                let path = id.strip_prefix(NS).unwrap_or(id);
                let n = lang[format!("item.farmersdelight.{path}")]
                    .as_str()
                    .or_else(|| lang[format!("block.farmersdelight.{path}")].as_str());
                n.map_or_else(|| json!([]), |n| json!([n.to_lowercase()]))
            };
            if let Some(a) = t["lines"].as_array_mut() {
                a.resize(n, json!([]));
                a.extend(new.iter().map(|id| name(id)));
            }
            if let Some(a) = t["tags"].as_array_mut() {
                a.resize(n, json!([]));
                a.extend(new.iter().map(|_| json!([])));
            }
        }
        tabs.push(json!({
            "id": "farmersdelight:farmersdelight",
            "type": "CATEGORY",
            "row": "TOP",
            "column": 0,
            "title": "Farmer's Delight",
            "icon": "{count:1,id:\"farmersdelight:stove\"}",
            "visible": true,
            "items": stacks,
        }));
    }
    wr(&p, &v)?;

    // -- the hoe (`RichSoilBlock.getToolModifiedState`: HOE_TILL with air
    // above turns rich soil into rich soil farmland), ahead of vanilla's rules.
    let hoe_path = dir.join("data/minecraft/block_transformer/hoe.json");
    if let Ok(mut hoe) = rd(&hoe_path)
        && let Some(rules) = hoe[0]["block_state_provider"]["rules"].as_array_mut()
        && !rules.iter().any(|r| r["then"]["id"] == "farmersdelight:rich_soil_farmland")
    {
        rules.insert(
            0,
            json!({
                "if_true": {
                    "type": "minecraft:all_of",
                    "predicates": [
                        { "type": "minecraft:matching_blocks", "blocks": "farmersdelight:rich_soil" },
                        { "type": "minecraft:matching_block_tag", "offset": [0, 1, 0], "tag": "minecraft:air" }
                    ]
                },
                "then": { "id": "farmersdelight:rich_soil_farmland", "properties": { "moisture": "0" } }
            }),
        );
        let _ = fs::remove_file(&hoe_path);
        wr(&hoe_path, &hoe)?;
    }

    // -- sounds.json: the mod's sounds that are references to other events
    // (`"type": "event"`, the stove's crackle, the cabinet's doors, the rope
    // gate's knot...) are resolved to the referenced event's sounds, which
    // the game's sound loader plays; events vanilla has renamed since 1.21.1
    // (the leash knot's, now the lead's) are followed to their new names.
    let fd_sounds_path = dir.join("assets/farmersdelight/sounds.json");
    if let (Ok(mut fd_sounds), Ok(vanilla)) = (
        rd(&fd_sounds_path),
        rd(&dir.join("assets/minecraft/sounds.json")),
    ) {
        let renamed = |name: &str| -> String {
            let path = name.strip_prefix("minecraft:").unwrap_or(name);
            match path {
                "entity.leash_knot.break" => "item.lead.break".into(),
                "entity.leash_knot.place" => "item.lead.tied".into(),
                p => p.into(),
            }
        };
        let mut changed = false;
        if let Some(events) = fd_sounds.as_object_mut() {
            for event in events.values_mut() {
                let Some(list) = event["sounds"].as_array() else { continue };
                if !list.iter().any(|s| s["type"] == "event") {
                    continue;
                }
                let mut resolved = Vec::new();
                for s in list {
                    if s["type"] == "event" {
                        let target = renamed(s["name"].as_str().unwrap_or(""));
                        if let Some(sounds) = vanilla[&target]["sounds"].as_array() {
                            resolved.extend(sounds.iter().cloned());
                        }
                    } else {
                        resolved.push(s.clone());
                    }
                }
                event["sounds"] = json!(resolved);
                changed = true;
            }
        }
        if changed {
            // The root's file may be a link to the mod's own: replaced, not written through.
            let _ = fs::remove_file(&fd_sounds_path);
            wr(&fd_sounds_path, &fd_sounds)?;
        }
    }

    // -- the screen's textures where the GUI atlas reads (`minecraft`'s
    // `textures/gui`): CookingPotScreen's background and the container slot's
    // empty icon (`CookingPotMenu.EMPTY_CONTAINER_SLOT_BOWL`).
    let gui = dir.join("assets/minecraft/textures/gui/farmersdelight");
    fs::create_dir_all(&gui).map_err(|e| format!("{}: {e}", gui.display()))?;
    for (from, to) in [
        ("assets/farmersdelight/textures/gui/cooking_pot.png", "cooking_pot.png"),
        ("assets/farmersdelight/textures/item/empty_container_slot_bowl.png", "empty_container_slot_bowl.png"),
    ] {
        let from = dir.join(from);
        fs::copy(&from, gui.join(to)).map_err(|e| format!("{}: {e}", from.display()))?;
    }
    // The steam particle's sprites (`particles/steam.json`), where the
    // particle atlas reads them.
    let particle = dir.join("assets/minecraft/textures/particle");
    fs::create_dir_all(&particle).map_err(|e| format!("{}: {e}", particle.display()))?;
    for n in 0..12 {
        let from = dir.join(format!("assets/farmersdelight/textures/particle/steam_{n}.png"));
        fs::copy(&from, particle.join(format!("farmersdelight_steam_{n}.png")))
            .map_err(|e| format!("{}: {e}", from.display()))?;
    }
    canvas_signs(dir, &blocks)?;

    // -- NBT tables
    let mut shapes_added = 0;
    let state_tag = |s: &St| -> Tag {
        let b = &blocks[s.block];
        let mut c = vec![("id".to_string(), Tag::String(b.name.clone()))];
        if !s.props.is_empty() {
            c.push(("properties".into(), Tag::Compound(s.props.iter().map(|(k, v)| (k.clone(), Tag::String(v.clone()))).collect())));
        }
        Tag::Compound(c)
    };
    // Rewrites one table level: states, cached_shapes from `pick`.
    let mut level = |t: &mut Tag, pick: &dyn Fn(&St) -> Vec<Bx>| -> Result<(), String> {
        if let Some(Tag::List(_, l)) = t.get_mut("states") {
            l.truncate(first);
            l.extend(states.iter().map(&state_tag));
        }
        if t.get("cached_shapes").is_some() {
            let mut list = match t.get_mut("shapes") {
                Some(Tag::List(_, l)) => std::mem::take(l),
                _ => return Err("a table without shapes".into()),
            };
            let before = list.len();
            let ids: Vec<i32> = {
                let mut sh = Shapes::new(&mut list)?;
                states.iter().map(|s| sh.id(&pick(s))).collect()
            };
            shapes_added += list.len() - before;
            if let Some(Tag::List(_, l)) = t.get_mut("shapes") {
                *l = list;
            }
            if let Some(Tag::IntArray(a)) = t.get_mut("cached_shapes") {
                a.truncate(first);
                a.extend(ids);
            }
        }
        Ok(())
    };
    let collision = |s: &St| s.collision.clone();
    let occlusion = |s: &St| s.occlusion.clone();
    for (file, pick) in [
        ("cached-collision.nbt", &collision as &dyn Fn(&St) -> Vec<Bx>),
        ("worldgen/blocks/cached-collision.nbt", &collision),
        ("worldgen/blocks/cached-support.nbt", &collision),
        ("occlusion.nbt", &occlusion),
        ("movement-properties.nbt", &collision),
        ("worldgen/blocks/block-properties.nbt", &collision),
    ] {
        let p = dir.join(file);
        let (gz, name, mut t) = nbt::read(&fs::read(&p).map_err(|e| format!("{file}: {e}"))?);
        level(&mut t, pick)?;
        if let Some(inner) = t.get_mut("occlusion") {
            level(inner, &occlusion)?;
        }
        if let Some(Tag::Compound(map)) = t.get_mut("blocks") {
            for b in &blocks {
                let d = &b.oracle["states"][b.oracle["default"].as_str().unwrap_or("")];
                if let Some((_, slot)) = map.iter_mut().find(|(k, _)| *k == b.name) {
                    match slot {
                        Tag::Byte(f) => {
                            let mut flags = 0i8;
                            if d["replaceable"].as_bool() == Some(true) {
                                flags |= 4;
                            }
                            if d["can_occlude"].as_bool() == Some(true) {
                                flags |= 16;
                            }
                            *f = flags;
                        }
                        Tag::Compound(_) => {
                            let f32_of = |k: &str| Tag::Float(b.oracle[k].as_f64().unwrap_or(0.6) as f32);
                            for (k, v) in [("friction", f32_of("friction")), ("speedFactor", f32_of("speed_factor")), ("jumpFactor", f32_of("jump_factor"))] {
                                if let Some(s) = slot.get_mut(k) {
                                    *s = v;
                                }
                            }
                            if let Some(s) = slot.get_mut("fenceGate") {
                                *s = Tag::Byte(0);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        fs::write(&p, nbt::write(gz, &name, &t)).map_err(|e| format!("{file}: {e}"))?;
    }

    Ok(Report { blocks: blocks.len(), states_before, states_after: total, shapes_added })
}

/// A small NBT reader and writer: enough for Horizon's prepared tables.
pub mod nbt {
    use std::io::Read;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Tag {
        End,
        Byte(i8),
        Short(i16),
        Int(i32),
        Long(i64),
        Float(f32),
        Double(f64),
        ByteArray(Vec<i8>),
        String(String),
        List(u8, Vec<Tag>),
        Compound(Vec<(String, Tag)>),
        IntArray(Vec<i32>),
        LongArray(Vec<i64>),
    }

    impl Tag {
        pub fn id(&self) -> u8 {
            match self {
                Tag::End => 0,
                Tag::Byte(_) => 1,
                Tag::Short(_) => 2,
                Tag::Int(_) => 3,
                Tag::Long(_) => 4,
                Tag::Float(_) => 5,
                Tag::Double(_) => 6,
                Tag::ByteArray(_) => 7,
                Tag::String(_) => 8,
                Tag::List(..) => 9,
                Tag::Compound(_) => 10,
                Tag::IntArray(_) => 11,
                Tag::LongArray(_) => 12,
            }
        }
        pub fn get(&self, key: &str) -> Option<&Tag> {
            match self {
                Tag::Compound(c) => c.iter().find(|(k, _)| k == key).map(|(_, v)| v),
                _ => None,
            }
        }
        pub fn get_mut(&mut self, key: &str) -> Option<&mut Tag> {
            match self {
                Tag::Compound(c) => c.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
                _ => None,
            }
        }
    }

    struct R<'a> {
        b: &'a [u8],
        p: usize,
    }
    impl R<'_> {
        fn take(&mut self, n: usize) -> &[u8] {
            let s = &self.b[self.p..self.p + n];
            self.p += n;
            s
        }
        fn u8(&mut self) -> u8 {
            self.take(1)[0]
        }
        fn i16(&mut self) -> i16 {
            i16::from_be_bytes(self.take(2).try_into().unwrap())
        }
        fn i32(&mut self) -> i32 {
            i32::from_be_bytes(self.take(4).try_into().unwrap())
        }
        fn i64(&mut self) -> i64 {
            i64::from_be_bytes(self.take(8).try_into().unwrap())
        }
        fn str(&mut self) -> String {
            let n = self.i16() as u16 as usize;
            // Modified UTF-8; the tables only hold ASCII names.
            String::from_utf8_lossy(self.take(n)).into_owned()
        }
        fn payload(&mut self, id: u8) -> Tag {
            match id {
                1 => Tag::Byte(self.u8() as i8),
                2 => Tag::Short(self.i16()),
                3 => Tag::Int(self.i32()),
                4 => Tag::Long(self.i64()),
                5 => Tag::Float(f32::from_bits(self.i32() as u32)),
                6 => Tag::Double(f64::from_bits(self.i64() as u64)),
                7 => {
                    let n = self.i32() as usize;
                    Tag::ByteArray(self.take(n).iter().map(|&b| b as i8).collect())
                }
                8 => Tag::String(self.str()),
                9 => {
                    let el = self.u8();
                    let n = self.i32().max(0) as usize;
                    Tag::List(el, (0..n).map(|_| self.payload(el)).collect())
                }
                10 => {
                    let mut out = Vec::new();
                    loop {
                        let t = self.u8();
                        if t == 0 {
                            break;
                        }
                        let k = self.str();
                        out.push((k, self.payload(t)));
                    }
                    Tag::Compound(out)
                }
                11 => {
                    let n = self.i32() as usize;
                    Tag::IntArray((0..n).map(|_| self.i32()).collect())
                }
                12 => {
                    let n = self.i32() as usize;
                    Tag::LongArray((0..n).map(|_| self.i64()).collect())
                }
                _ => panic!("bad tag id {id} at {}", self.p),
            }
        }
    }

    /// Reads a (possibly gzipped) named root tag: (gzipped, name, tag).
    pub fn read(bytes: &[u8]) -> (bool, String, Tag) {
        let (gz, raw) = if bytes.starts_with(&[0x1f, 0x8b]) {
            let mut out = Vec::new();
            flate2::read::GzDecoder::new(bytes).read_to_end(&mut out).unwrap();
            (true, out)
        } else {
            (false, bytes.to_vec())
        };
        let mut r = R { b: &raw, p: 0 };
        let id = r.u8();
        let name = r.str();
        (gz, name, r.payload(id))
    }

    fn w_str(o: &mut Vec<u8>, s: &str) {
        o.extend((s.len() as u16).to_be_bytes());
        o.extend(s.as_bytes());
    }
    fn w_payload(o: &mut Vec<u8>, t: &Tag) {
        match t {
            Tag::End => {}
            Tag::Byte(v) => o.push(*v as u8),
            Tag::Short(v) => o.extend(v.to_be_bytes()),
            Tag::Int(v) => o.extend(v.to_be_bytes()),
            Tag::Long(v) => o.extend(v.to_be_bytes()),
            Tag::Float(v) => o.extend(v.to_bits().to_be_bytes()),
            Tag::Double(v) => o.extend(v.to_bits().to_be_bytes()),
            Tag::ByteArray(v) => {
                o.extend((v.len() as i32).to_be_bytes());
                o.extend(v.iter().map(|&b| b as u8));
            }
            Tag::String(s) => w_str(o, s),
            Tag::List(el, v) => {
                o.push(if v.is_empty() { *el } else { v[0].id() });
                o.extend((v.len() as i32).to_be_bytes());
                for x in v {
                    w_payload(o, x);
                }
            }
            Tag::Compound(c) => {
                for (k, v) in c {
                    o.push(v.id());
                    w_str(o, k);
                    w_payload(o, v);
                }
                o.push(0);
            }
            Tag::IntArray(v) => {
                o.extend((v.len() as i32).to_be_bytes());
                for x in v {
                    o.extend(x.to_be_bytes());
                }
            }
            Tag::LongArray(v) => {
                o.extend((v.len() as i32).to_be_bytes());
                for x in v {
                    o.extend(x.to_be_bytes());
                }
            }
        }
    }
    pub fn write(gz: bool, name: &str, t: &Tag) -> Vec<u8> {
        let mut o = vec![t.id()];
        w_str(&mut o, name);
        w_payload(&mut o, t);
        if gz {
            use std::io::Write;
            let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            e.write_all(&o).unwrap();
            e.finish().unwrap()
        } else {
            o
        }
    }

    /// One line per node down to `depth`: key, type, sizes.
    pub fn outline(t: &Tag, key: &str, depth: usize, indent: usize, out: &mut String) {
        let pad = "  ".repeat(indent);
        let desc = match t {
            Tag::List(el, v) => format!("List<{el}>[{}]", v.len()),
            Tag::Compound(c) => format!("Compound{{{}}}", c.len()),
            Tag::IntArray(v) => format!("IntArray[{}]", v.len()),
            Tag::LongArray(v) => format!("LongArray[{}]", v.len()),
            Tag::ByteArray(v) => format!("ByteArray[{}]", v.len()),
            other => format!("{other:?}"),
        };
        out.push_str(&format!("{pad}{key}: {desc}\n"));
        if depth == 0 {
            return;
        }
        match t {
            Tag::Compound(c) => {
                for (k, v) in c.iter().take(40) {
                    outline(v, k, depth - 1, indent + 1, out);
                }
            }
            Tag::List(_, v) => {
                if let Some(f) = v.first() {
                    outline(f, "[0]", depth - 1, indent + 1, out);
                }
            }
            _ => {}
        }
    }
}
