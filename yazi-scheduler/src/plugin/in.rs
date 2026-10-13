use std::borrow::Cow;

use hashbrown::HashMap;
use mlua::{FromLua, Lua, Table, Value};
use yazi_binding::{Scope, runtime};
use yazi_runner::entry::EntryJob;
use yazi_shared::{data::{Data, DataKey}, id::Id, sendable::Sendable};
use yazi_shim::SStr;

use crate::{TaskIn, custom::CustomIn, plugin::PluginProgEntry};

#[derive(Debug)]
pub(crate) enum PluginIn {
	Entry(PluginInEntry),
	Custom(CustomIn),
}

impl TaskIn for PluginIn {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Entry(r#in) => r#in.id(),
			Self::Custom(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Entry(r#in) => _ = r#in.set_id(id),
			Self::Custom(r#in) => _ = r#in.set_id(id),
		}
		self
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Entry(r#in) => r#in.title(),
			Self::Custom(r#in) => r#in.title(),
		}
	}
}

impl_from_in!(Entry(PluginInEntry), Custom(CustomIn));

// --- Entry
#[derive(Clone, Debug, Default)]
pub struct PluginInEntry {
	pub id:     Id,
	pub tab:    Id,
	pub plugin: SStr,
	pub args:   HashMap<DataKey, Data>,
	pub title:  SStr,
	pub track:  bool,
	pub scope:  Scope,
}

impl TaskIn for PluginInEntry {
	type Prog = PluginProgEntry;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		if self.title.is_empty() {
			format!("Run plugin '{}'", self.plugin).into()
		} else {
			Cow::Borrowed(&self.title)
		}
	}

	fn set_title(&mut self, title: impl Into<SStr>) -> &mut Self {
		self.title = title.into();
		self
	}
}

impl From<PluginInEntry> for EntryJob {
	fn from(value: PluginInEntry) -> Self {
		Self {
			id: value.id,
			tab: value.tab,
			args: value.args,
			plugin: value.plugin,
			scope: value.scope,
		}
	}
}

impl PluginInEntry {
	/// `Scope::from_lua(nil)` silently yields an *empty*, uncancellable scope, so
	/// a plain `t.raw_get("scope")` would drop the caller's scope whenever the
	/// Lua table omits it -- exactly the case for `ya.task("plugin", ...)` and
	/// the built-in `extract` opener. Only trust a value that is actually
	/// present; otherwise inherit the live scope of the calling plugin, which is
	/// what makes the spawned task cancelable (see sxyazi/yazi#4404).
	fn scope_from(lua: &Lua, t: &Table) -> mlua::Result<Scope> {
		let value: Value = t.raw_get("scope")?;
		if value.is_nil() {
			Ok(runtime!(lua).map(|r| r.scope()).unwrap_or_default())
		} else {
			Scope::from_lua(value, lua)
		}
	}
}

impl FromLua for PluginInEntry {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self {
			tab: t.raw_get::<Id>("tab").unwrap_or_default().nz().unwrap_or(runtime!(lua)?.tab()),
			plugin: t.raw_get::<String>(1)?.into(),
			args: Sendable::table_to_args(lua, t.raw_get("args")?)?,
			track: t.raw_get("track")?,
			scope: Self::scope_from(lua, &t)?,
			..Default::default()
		})
	}
}

#[cfg(test)]
mod tests {
	use mlua::{FromLua, Lua};
	use yazi_binding::{Scope, runtime::{Runtime, RuntimeSeed}};

	use super::PluginInEntry;

	/// Regression test for sxyazi/yazi#4404, at the exact place the bug lived.
	///
	/// The built-in `extract` opener does:
	///
	///     ya.task("plugin", { args = ..., track = true }):name("Extract ..."):spawn()
	///
	/// It passes no `scope`. Before the fix that task had no cancelable scope, so
	/// pressing `x` in the `w` task manager never reached the running plugin, and
	/// the `7zz` child it had spawned ran to completion -- the files still
	/// appeared.
	///
	/// This drives the real `FromLua` path with a live scope installed on the
	/// calling runtime (as an entry plugin's runtime now has) and asserts the
	/// parsed task inherits that scope and stays cancelable.
	#[test]
	fn plugin_task_without_explicit_scope_inherits_the_callers_live_scope() {
		let lua = Lua::new();
		let caller = Scope::new();
		lua.set_app_data(Runtime::new(RuntimeSeed::new(Default::default(), "extract", caller.clone())));

		let t = lua.load(r#"return { "extract", args = {}, track = true }"#).eval::<mlua::Table>().unwrap();
		let job = PluginInEntry::from_lua(mlua::Value::Table(t), &lua).unwrap();

		// A `Scope::from_lua(nil)` would have handed back an empty, permanently
		// uncancellable scope instead of the caller's live one.
		assert!(!job.scope.is_cancelled());
		caller.cancel();
		assert!(
			job.scope.is_cancelled(),
			"a plugin task spawned without an explicit scope must inherit the caller's cancelable scope"
		);

		// And it must survive the round-trip into the runner as the same scope.
		let entry_job: yazi_runner::entry::EntryJob = job.into();
		assert!(entry_job.scope.is_cancelled());
	}

	/// A task that *does* pass an explicit scope keeps that scope.
	#[test]
	fn plugin_task_honours_an_explicit_scope() {
		let lua = Lua::new();
		let caller = Scope::new();
		let explicit = Scope::new();
		lua.set_app_data(Runtime::new(RuntimeSeed::new(Default::default(), "extract", caller.clone())));
		let explicit_ud = lua.create_userdata(explicit.clone()).unwrap();

		let globals = lua.globals();
		globals.set("EXPLICIT", explicit_ud).unwrap();
		let t = lua
			.load(r#"return { "extract", args = {}, track = true, scope = EXPLICIT }"#)
			.eval::<mlua::Table>()
			.unwrap();
		let job = PluginInEntry::from_lua(mlua::Value::Table(t), &lua).unwrap();

		caller.cancel();
		assert!(!job.scope.is_cancelled(), "an explicit scope must not follow the caller");
		explicit.cancel();
		assert!(job.scope.is_cancelled());
	}
}
