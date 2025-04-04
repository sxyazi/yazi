use mlua::{IntoLuaMulti, MetaMethod, UserData, UserDataMethods};

pub struct FolderStage(yazi_fs::FolderStage);

impl FolderStage {
	pub fn new(inner: yazi_fs::FolderStage) -> Self { Self(inner) }
}

impl UserData for FolderStage {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Call, |lua, me, ()| {
			use yazi_fs::FolderStage::*;

			match me.0 {
				Loading => false.into_lua_multi(lua),
				Loaded => true.into_lua_multi(lua),
				Failed(kind) => (true, crate::Error::IoKind(kind)).into_lua_multi(lua),
			}
		});
	}
}
