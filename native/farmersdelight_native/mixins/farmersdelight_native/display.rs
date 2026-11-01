// A module of text_displays.rs: a display Farmer's Delight makes between two
// ticks takes its first tick at once, so its transformation applies before
// the first frame draws it (otherwise that frame is unposed: upright, full
// size).
impl super::RemoteTextDisplay {
    pub(in crate::game) fn fd_settle(&mut self) {
        self.tick();
    }
}
