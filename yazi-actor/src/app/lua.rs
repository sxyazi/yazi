use anyhow::Result;
use yazi_dds::Sendable;
use yazi_macro::succ;
use yazi_parser::app::LuaOpt;
use yazi_plugin::LUA;
use yazi_shared::data::Data;

use crate::{Actor, Ctx, lives::Lives};

pub struct Lua;

impl Actor for Lua {
	type Options = LuaOpt;

	const NAME: &str = "lua";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let result = Lives::scope(cx.core, || {
			let chunk = LUA.load(&*opt.code).set_name("anonymous");
			Sendable::value_to_data(&LUA, chunk.eval()?)
		});
		succ!(result?);
	}
}
