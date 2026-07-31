yazi_macro::mod_pub!(drivers);

yazi_macro::mod_flat!(adapter icc image);

use yazi_shim::cell::{RoCell, SyncCell};

pub static ADAPTOR: RoCell<Adapter> = RoCell::new();
pub static WSL: SyncCell<bool> = SyncCell::new(false);

pub fn init() {
	ADAPTOR.with(Adapter::default);
	WSL.set(yazi_shared::in_wsl());
}
