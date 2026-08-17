use yazi_term::TERM;

use crate::EMULATOR;

#[derive(Clone, Copy, Debug, Default)]
pub struct Dimension;

impl Dimension {
	pub fn cell_size() -> Option<(f64, f64)> {
		let csi_16t = EMULATOR.csi_16t.get();
		Some(if EMULATOR.force_16t.get() {
			(csi_16t.0 as f64, csi_16t.1 as f64)
		} else if let Some(r) = TERM.dimension().ratio() {
			r
		} else if csi_16t.0 != 0 && csi_16t.1 != 0 {
			(csi_16t.0 as f64, csi_16t.1 as f64)
		} else {
			None?
		})
	}
}
