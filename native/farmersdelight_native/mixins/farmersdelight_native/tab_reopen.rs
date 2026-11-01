// creative::Creative::reopen HEAD: the screen opens on the page of the
// selected tab (a right-aligned tab keeps the page that was shown).
fn farmersdelight_tab_reopen(creative: &Creative) {
    if let Some(page) = farmersdelight_pages::page_of(&creative.tab().id) {
        farmersdelight_pages::set(page);
    }
}
