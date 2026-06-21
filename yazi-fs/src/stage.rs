use mlua::{IntoLuaMulti, MetaMethod, UserData, UserDataMethods};
use serde::{Deserialize, Serialize};
use yazi_binding::Error;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum FolderStage {
	#[default]
	Loading,
	Loaded,
	Failed(yazi_shim::fs::Error),
}

impl FolderStage {
	pub fn is_loading(&self) -> bool { *self == Self::Loading }
}

impl UserData for FolderStage {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Call, |lua, me, ()| {
			use FolderStage::*;

			match me {
				Loading => false.into_lua_multi(lua),
				Loaded => true.into_lua_multi(lua),
				Failed(e) => (true, Error::Fs(e.clone())).into_lua_multi(lua),
			}
		});
	}
}
