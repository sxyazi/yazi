yazi_macro::mod_flat!(args boot);

use clap::Parser;
use yazi_shared::id::Id;
use yazi_shim::cell::RoCell;

pub static ID: RoCell<Id> = RoCell::new();
pub static ARGS: RoCell<Args> = RoCell::new();
pub static BOOT: RoCell<Boot> = RoCell::new();

pub fn init() {
	ARGS.init(Args::parse());
	ID.init(ARGS.client_id.unwrap_or_else(Id::unique));
	BOOT.init(Default::default());
}

pub fn init_default() {
	ARGS.with(<_>::default);
	ID.init(ARGS.client_id.unwrap_or_else(Id::unique));
	BOOT.with(<_>::default);
}
