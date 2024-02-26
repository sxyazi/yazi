use yazi_shared::RoCell;

mod args;
mod boot;

pub use args::*;
pub use boot::*;

pub static ARGS: RoCell<Args> = RoCell::new();
pub static BOOT: RoCell<Boot> = RoCell::new();

pub fn init() {
	ARGS.with(Default::default);
	BOOT.with(Default::default);
}
