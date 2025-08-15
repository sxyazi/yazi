use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use yazi_binding::{FileRef, elements::{Rect, Renderable}};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct UpdatePeekedOpt {
	pub lock: PreviewLock,
}

impl TryFrom<CmdCow> for UpdatePeekedOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		let Some(lock) = c.take_any("lock") else {
			bail!("Invalid 'lock' argument in UpdatePeekedOpt");
		};

		Ok(Self { lock })
	}
}

impl FromLua for UpdatePeekedOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdatePeekedOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Lock
#[derive(Debug, Default)]
pub struct PreviewLock {
	pub url:  yazi_shared::url::UrlBuf,
	pub cha:  yazi_fs::cha::Cha,
	pub mime: String,

	pub skip: usize,
	pub area: Rect,
	pub data: Vec<Renderable>,
}

impl TryFrom<Table> for PreviewLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("file")?;
		Ok(Self {
			url:  file.url_owned(),
			cha:  file.cha,
			mime: t.raw_get("mime")?,

			skip: t.raw_get("skip")?,
			area: t.raw_get("area")?,
			data: Default::default(),
		})
	}
}
