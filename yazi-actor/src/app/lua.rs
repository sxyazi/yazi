use anyhow::Result;
use yazi_binding::runtime_scope;
use yazi_core::Ctx;
use yazi_macro::succ;
use yazi_parser::app::LuaForm;
use yazi_plugin::LUA;
use yazi_shared::{data::Data, sendable::Sendable};

use crate::{Actor, lives::Lives};

pub struct Lua;

impl Actor for Lua {
	type Form = LuaForm;

	const NAME: &str = "lua";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let chunk = LUA.load(&*form.code).set_name("anonymous");
		let result = Lives::scope(cx, |cx| {
			runtime_scope!(cx, "inline", Sendable::value_to_data(&LUA, chunk.eval()?))
		});
		succ!(result?);
	}
}
