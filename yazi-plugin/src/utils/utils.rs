#[cfg(unix)]
pub(super) static USERS_CACHE: yazi_shared::RoCell<uzers::UsersCache> = yazi_shared::RoCell::new();

pub(super) static HOSTNAME_CACHE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

pub struct Utils;

impl Utils {
	pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
		let ya: mlua::Table = lua.create_table()?;

		Utils::cache(lua, &ya)?;
		Utils::call(lua, &ya)?;
		Utils::image(lua, &ya)?;
		Utils::layer(lua, &ya)?;
		Utils::log(lua, &ya)?;
		Utils::plugin(lua, &ya)?;
		Utils::preview(lua, &ya)?;
		Utils::target(lua, &ya)?;
		Utils::text(lua, &ya)?;
		Utils::time(lua, &ya)?;
		Utils::user(lua, &ya)?;

		lua.globals().set("ya", ya)
	}
}

pub fn init() {
	#[cfg(unix)]
	USERS_CACHE.with(Default::default);
}
