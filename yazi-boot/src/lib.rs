yazi_macro::mod_pub!(actions);

yazi_macro::mod_flat!(args boot);

use clap::Parser;
use yazi_shim::cell::RoCell;

pub static ARGS: RoCell<Args> = RoCell::new();
pub static BOOT: RoCell<Boot> = RoCell::new();

pub fn preflight() {
	ARGS.with(<_>::parse);
	actions::Actions::act_early(&ARGS);
}

pub fn init() {
	BOOT.init(<_>::from(&*ARGS));

	actions::Actions::act(&ARGS);
}

pub fn init_default() {
	ARGS.with(<_>::default);
	BOOT.with(<_>::default);
}
