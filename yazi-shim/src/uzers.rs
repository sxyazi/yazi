#[cfg(unix)]
static USERS_CACHE: crate::cell::RoCell<::uzers::UsersCache> = crate::cell::RoCell::new();

pub struct Uzers;

impl Uzers {
	#[cfg(unix)]
	pub(crate) fn init() { USERS_CACHE.with(<_>::default); }

	#[cfg(unix)]
	pub fn uid() -> u32 {
		use ::uzers::Users;
		USERS_CACHE.get_current_uid()
	}

	pub fn uid_or_zero() -> u32 { yazi_macro::unix_either!(Self::uid(), 0) }

	#[cfg(unix)]
	pub fn gid() -> u32 {
		use ::uzers::Groups;
		USERS_CACHE.get_current_gid()
	}

	#[cfg(unix)]
	pub fn user_name(uid: Option<u32>) -> Option<std::ffi::OsString> {
		use ::uzers::Users;
		USERS_CACHE.get_user_by_uid(uid.unwrap_or_else(Self::uid)).map(|u| u.name().to_owned())
	}

	#[cfg(unix)]
	pub fn group_name(gid: Option<u32>) -> Option<std::ffi::OsString> {
		use ::uzers::Groups;
		USERS_CACHE.get_group_by_gid(gid.unwrap_or_else(Self::gid)).map(|g| g.name().to_owned())
	}
}
