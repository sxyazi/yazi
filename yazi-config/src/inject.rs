use mlua::UserDataMethods;
use yazi_fs::file::FileInventory;

use crate::THEME;

inventory::submit! {
	FileInventory {
		register: |registry| {
			registry.add_method("icon", |_, me, ()| {
				// TODO: use a cache
				Ok(THEME.icon.matches(me, false))
			});
		},
		from_lua: |_| Err(mlua::Error::UserDataTypeMismatch),
	}
}
