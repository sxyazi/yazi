use clap::Parser;
use yazi_shared::RoCell;

mod actions;
mod args;
mod boot;

pub use args::*;
pub use boot::*;

pub static ARGS: RoCell<Args> = RoCell::new();
pub static BOOT: RoCell<Boot> = RoCell::new();

pub fn init() {
	ARGS.with(<_>::parse);
	BOOT.init(From::from(&*ARGS));

	actions::Actions::act(&ARGS);
}

pub fn init_default() {
	ARGS.with(<_>::default);
	BOOT.with(<_>::default);
}
