// NeoForge's creative tab pages (`CreativeModeInventoryScreen.pages`,
// `CreativeTabsScreenPage`): the category tabs, ten to a page (five on top,
// five below), vanilla's first; the right-aligned tabs (saved hotbars,
// search, inventory, operator blocks) on every page. Farmer's Delight's tab
// (`ModCreativeTabs.TAB_FARMERS_DELIGHT`) is the first of page 2.
use std::sync::atomic::{AtomicUsize, Ordering};

/// The tab Farmer's Delight registers.
pub const TAB: &str = "farmersdelight:farmersdelight";
/// The pages there are.
pub const PAGES: usize = 2;
/// `currentPage`.
static PAGE: AtomicUsize = AtomicUsize::new(0);

/// The page a tab is on; None for the right-aligned tabs, which every page shows.
pub fn page_of(id: &str) -> Option<usize> {
    match id {
        "minecraft:hotbar" | "minecraft:search" | "minecraft:inventory" | "minecraft:op_blocks" => {
            None
        }
        TAB => Some(1),
        _ => Some(0),
    }
}
/// `currentPage.getVisibleTabs().contains(tab)`.
pub fn visible(id: &str) -> bool {
    page_of(id).is_none_or(|p| p == current())
}
pub fn current() -> usize {
    PAGE.load(Ordering::Relaxed)
}
pub fn set(page: usize) {
    PAGE.store(page.min(PAGES - 1), Ordering::Relaxed);
}
