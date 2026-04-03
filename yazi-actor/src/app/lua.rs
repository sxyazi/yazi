use anyhow::Result;
use yazi_binding::runtime_scope;
use yazi_dds::Sendable;
use yazi_macro::succ;
use yazi_parser::app::LuaForm;
use yazi_plugin::LUA;
use yazi_shared::data::Data;

use crate::{Actor, Ctx, lives::Lives};

pub struct Lua;

impl Actor for Lua {
	type Form = LuaForm;

	const NAME: &str = "lua";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let chunk = LUA.load(&*form.code).set_name("anonymous");
		let result = Lives::scope(cx.core, || {
			runtime_scope!(LUA, "inline", Sendable::value_to_data(&LUA, chunk.eval()?))
		});
		succ!(result?);
	}
}
