use yazi_shared::RoCell;

mod args;
mod boot;

pub use args::*;
pub use boot::*;

pub static ARGS: RoCell<Args> = RoCell::new();
pub static BOOT: RoCell<Boot> = RoCell::new();

#[cfg(unix)]
pub static USERS_CACHE: yazi_shared::RoCell<uzers::UsersCache> = yazi_shared::RoCell::new();

pub fn init() {
	ARGS.with(Default::default);
	BOOT.with(Default::default);

	#[cfg(unix)]
	USERS_CACHE.with(Default::default);
}
