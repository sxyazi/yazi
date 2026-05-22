use yazi_term::TERM;

use crate::EMULATOR;

#[derive(Clone, Copy, Debug, Default)]
pub struct Dimension;

impl Dimension {
	pub fn cell_size() -> Option<(f64, f64)> {
		let emu = &*EMULATOR;
		Some(if emu.force_16t {
			(emu.csi_16t.0 as f64, emu.csi_16t.1 as f64)
		} else if let Some(r) = TERM.dimension().ratio() {
			r
		} else if emu.csi_16t.0 != 0 && emu.csi_16t.1 != 0 {
			(emu.csi_16t.0 as f64, emu.csi_16t.1 as f64)
		} else {
			None?
		})
	}
}
