// mod_content::resource_root HEAD: builds the content root as the game does,
// then lays the Farmer's Delight block states out from the mod's own facts
// (`farmersdelight_tables`). Its stamp names the generator too, so a root
// made by the plain game, or by an older generator, is built again.
fn farmersdelight_root(
    base: &Path,
    mods: &[ContentMod],
    cache: &Path,
) -> core::ops::ControlFlow<Result<PathBuf>> {
    use core::ops::ControlFlow::{Break, Continue};
    if !mods.iter().any(|m| m.id == "farmersdelight" && !m.content.is_empty()) {
        return Continue(());
    }
    let run = || -> Result<PathBuf> {
        let out = cache.join("mod-content");
        let stamp = format!("{}\n{}", stamp(base, mods)?, farmersdelight_tables::VERSION);
        if fs::read(out.join(STAMP)).is_ok_and(|s| s == stamp.as_bytes()) {
            let warnings = fs::read_to_string(out.join(WARNINGS)).unwrap_or_default();
            warnings.lines().for_each(|w| log::warn!("Horizon: {w}"));
            return Ok(out);
        }
        for warning in build(base, mods, &out)? {
            log::warn!("Horizon: {warning}");
        }
        let oracle = out.join("data/farmersdelight/horizon_oracle/blocks.json");
        let oracle: serde_json::Value = serde_json::from_slice(&fs::read(&oracle)?)?;
        let report = farmersdelight_tables::apply(&out, &oracle)
            .map_err(|e| format!("Farmer's Delight block tables: {e}"))?;
        log::info!(
            "Horizon: Farmer's Delight: {} blocks laid out, {} block states in all",
            report.blocks,
            report.states_after
        );
        fs::write(out.join(STAMP), stamp)?;
        Ok(out)
    };
    Break(run())
}
