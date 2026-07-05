use mlua::UserDataMethods;
use yazi_fs::file::FileInventory;

use crate::THEME;

inventory::submit! {
	FileInventory {
		register: |registry| {
			registry.add_method("icon", |lua, me, ()| {
				yazi_binding::deprecate!(
					lua,
					"{}: `File:icon()` is deprecated, use `th.icon:match(file)` instead"
				);
				// TODO: use a cache
				Ok(THEME.icon.matches(me, false))
			});
		},
		borrow: |_, _| Err(mlua::Error::UserDataTypeMismatch),
	}
}
