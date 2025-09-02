use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use yazi_binding::{FileRef, elements::Renderable};
use yazi_shared::{Id, event::CmdCow};

#[derive(Debug)]
pub struct UpdateSpottedOpt {
	pub lock: SpotLock,
}

impl TryFrom<CmdCow> for UpdateSpottedOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		let Some(lock) = c.take_any("lock") else {
			bail!("Invalid 'lock' argument in UpdateSpottedOpt");
		};

		Ok(Self { lock })
	}
}

impl FromLua for UpdateSpottedOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateSpottedOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Lock
#[derive(Debug)]
pub struct SpotLock {
	pub url:  yazi_shared::url::UrlBuf,
	pub cha:  yazi_fs::cha::Cha,
	pub mime: String,

	pub id:   Id,
	pub skip: usize,
	pub data: Vec<Renderable>,
}

impl TryFrom<Table> for SpotLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("file")?;
		Ok(Self {
			url:  file.url_owned(),
			cha:  file.cha,
			mime: t.raw_get("mime")?,

			id:   *t.raw_get::<yazi_binding::Id>("id")?,
			skip: t.raw_get("skip")?,
			data: Default::default(),
		})
	}
}

impl SpotLock {
	pub fn len(&self) -> Option<usize> { Some(self.table()?.len()) }

	pub fn select(&mut self, idx: Option<usize>) {
		if let Some(t) = self.table_mut() {
			t.select(idx);
		}
	}

	pub fn selected(&self) -> Option<usize> { self.table()?.selected() }

	pub fn table(&self) -> Option<&yazi_binding::elements::Table> {
		self.data.iter().rev().find_map(|r| match r {
			Renderable::Table(t) => Some(t.as_ref()),
			_ => None,
		})
	}

	pub fn table_mut(&mut self) -> Option<&mut yazi_binding::elements::Table> {
		self.data.iter_mut().rev().find_map(|r| match r {
			Renderable::Table(t) => Some(t.as_mut()),
			_ => None,
		})
	}
}
