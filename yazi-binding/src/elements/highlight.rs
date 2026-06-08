use mlua::{
	ExternalError, FromLua, IntoLua, Lua, Result as LuaResult, Table, UserData, UserDataMethods,
	Value,
};

const EXPECTED: &str = "expected a table containing a line and a length";

#[derive(Clone, Debug)]
pub struct HighlightPosition {
	pub line: usize,
	pub length: usize,
}

impl HighlightPosition {
	pub fn compose(lua: &Lua) -> LuaResult<Value> {
		let new = lua.create_function(|_, (line, length): (usize, usize)| Ok(Self { line, length }))?;

		let tbl = lua.create_table_from([("new", new)])?;
		tbl.into_lua(lua)
	}
}

impl FromLua for HighlightPosition {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(tbl) => Ok(Self { line: tbl.raw_get("line")?, length: tbl.raw_get("length")? }),
			_ => Err(EXPECTED.into_lua_err()),
		}
	}
}

impl IntoLua for HighlightPosition {
	fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
		let tbl = lua.create_table()?;
		tbl.set("line", self.line)?;
		tbl.set("length", self.length)?;
		Ok(Value::Table(tbl))
	}
}
