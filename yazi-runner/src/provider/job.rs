use std::sync::Arc;

use mlua::{IntoLua, Lua, Value};
use yazi_config::vfs::ServiceLua;
use yazi_shared::{id::Id, sendable::Sendable};
use yazi_shim::SStr;

use super::ProvideOp;
use crate::PluginJob;

pub struct ProvideJob {
	pub service: Arc<ServiceLua>,
	pub op:      ProvideOp,
}

impl ProvideJob {
	pub fn new(service: &Arc<ServiceLua>, op: ProvideOp) -> Self {
		Self { service: service.clone(), op }
	}
}

impl PluginJob for ProvideJob {
	fn tab(&self) -> Id { Id::ZERO }

	fn name(&self) -> &SStr { &self.service.name }
}

impl IntoLua for ProvideJob {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let job = self.op.into_table(lua)?;

		job.raw_set("args", Sendable::args_to_table_ref(lua, &self.service.args)?)?;
		job.raw_set("opts", Sendable::args_to_table_ref(lua, &self.service.opts)?)?;
		job.into_lua(lua)
	}
}
