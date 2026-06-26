use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::deprecate;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "[u16; 3]")]
pub struct MgrRatio {
	pub parent:  u16,
	pub current: u16,
	pub preview: u16,
	pub all:     u16,
}

impl TryFrom<[u16; 3]> for MgrRatio {
	type Error = anyhow::Error;

	fn try_from(ratio: [u16; 3]) -> Result<Self, Self::Error> {
		if ratio.len() != 3 {
			bail!("invalid layout ratio: {:?}", ratio);
		}
		if ratio.iter().all(|&r| r == 0) {
			bail!("at least one layout ratio must be non-zero: {:?}", ratio);
		}

		Ok(Self {
			parent:  ratio[0],
			current: ratio[1],
			preview: ratio[2],
			all:     ratio[0] + ratio[1] + ratio[2],
		})
	}
}

// TODO: remove ---
impl MgrRatio {
	fn at(self, idx: usize) -> Option<u16> {
		match idx {
			0 => Some(self.parent),
			1 => Some(self.current),
			2 => Some(self.preview),
			_ => None,
		}
	}
}

impl FromLua for MgrRatio {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			Value::Table(t) => from_table(&t, lua),
			_ => Err("expected a table".into_lua_err()),
		}
	}
}

fn from_table(t: &Table, lua: &Lua) -> mlua::Result<MgrRatio> {
	if t.contains_key("parent")? || t.contains_key("current")? || t.contains_key("preview")? {
		deprecate!(
			lua,
			"{}: the table form of `rt.mgr.ratio` is deprecated, use an array `[parent, current, preview]` instead"
		);
		MgrRatio::try_from([t.get("parent")?, t.get("current")?, t.get("preview")?])
			.map_err(|e| e.into_lua_err())
	} else {
		MgrRatio::try_from([t.get(1)?, t.get(2)?, t.get(3)?]).map_err(|e| e.into_lua_err())
	}
}

impl UserData for MgrRatio {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, _, ()| Ok(3usize));

		methods.add_meta_method(MetaMethod::Index, |lua, me, key: Value| {
			match key {
				Value::Integer(i) if (1..=3).contains(&i) => me.at((i - 1) as usize).into_lua(lua),
				Value::String(s) => match &*s.as_bytes() {
					b"parent" => {
						deprecate!(
							lua,
							"{}: `rt.mgr.ratio.parent` is deprecated, use `rt.mgr.ratio[1]` instead"
						);
						me.parent.into_lua(lua)
					}
					b"current" => {
						deprecate!(
							lua,
							"{}: `rt.mgr.ratio.current` is deprecated, use `rt.mgr.ratio[2]` instead"
						);
						me.current.into_lua(lua)
					}
					b"preview" => {
						deprecate!(
							lua,
							"{}: `rt.mgr.ratio.preview` is deprecated, use `rt.mgr.ratio[3]` instead"
						);
						me.preview.into_lua(lua)
					}
					b"all" => {
						deprecate!(
							lua,
							"{}: `rt.mgr.ratio.all` is deprecated, use `rt.mgr.ratio[1] + rt.mgr.ratio[2] + rt.mgr.ratio[3]` instead"
						);
						me.all.into_lua(lua)
					}
					_ => Ok(Value::Nil),
				},
				_ => Ok(Value::Nil),
			}
		});
	}
}
// --- TODO: remove
