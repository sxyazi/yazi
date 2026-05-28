use std::io::Write;

use mlua::{BorrowedBytes, ExternalError, IntoLuaMulti, Lua, MultiValue, Table, UserData, UserDataMethods};
use yazi_shim::mlua::{ByteString, LuaTableExt};
use yazi_term::sequence::{ConfirmDrag, ConfirmDrop, FinishDrop, PresentDrag, PresentDragIcon, StartDrag, StartDrop};
use yazi_tty::TTY;

use crate::Error;

pub struct Tty;

impl Tty {
	fn queue(lua: &Lua, kind: &[u8], t: &Table) -> mlua::Result<MultiValue> {
		let mut w = TTY.writer();

		let result = match kind {
			b"ConfirmDrag" => {
				let it = t.raw_get::<Table>("mimes")?.sequence_iter::<ByteString>(lua).flatten();
				write!(w, "{}", match &*t.raw_get::<BorrowedBytes>("type")? {
					b"copy" => ConfirmDrag::Copy(it),
					b"move" => ConfirmDrag::Move(it),
					b"either" => ConfirmDrag::Either(it),
					_ => return Err("invalid ConfirmDrag type".into_lua_err()),
				})
			}
			b"ConfirmDrop" => {
				let it = t.raw_get::<Table>("mimes")?.sequence_iter::<ByteString>(lua).flatten();
				write!(w, "{}", match &*t.raw_get::<BorrowedBytes>("type")? {
					b"reject" => ConfirmDrop::Reject,
					b"copy" => ConfirmDrop::Copy(it),
					b"move" => ConfirmDrop::Move(it),
					_ => return Err("invalid ConfirmDrop type".into_lua_err()),
				})
			}
			b"StartDrag" => write!(w, "{StartDrag}"),
			b"StartDrop" => write!(w, "{}", StartDrop(t.raw_get("idx")?)),
			b"PresentDrag" => {
				write!(w, "{}", PresentDrag(t.raw_get("idx")?, &t.raw_get::<BorrowedBytes>("data")?))
			}
			b"PresentDragIcon" => {
				write!(w, "{}", PresentDragIcon {
					format:  t.raw_get("format")?,
					opacity: t.raw_get("opacity")?,
					width:   t.raw_get("width")?,
					height:  t.raw_get("height")?,
					data:    &t.raw_get::<BorrowedBytes>("data")?,
				})
			}
			b"FinishDrop" => match &*t.raw_get::<BorrowedBytes>("type")? {
				b"copy" => write!(w, "{}", FinishDrop::Copy),
				b"move" => write!(w, "{}", FinishDrop::Move),
				_ => return Err("invalid FinishDrop type".into_lua_err()),
			},
			_ => return Err("invalid sequence kind".into_lua_err()),
		};

		match result {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	}
}

impl UserData for Tty {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods
			.add_method("queue", |lua, _, (kind, t): (BorrowedBytes, Table)| Self::queue(lua, &kind, &t));

		methods.add_method("flush", |lua, _, ()| match TTY.writer().flush() {
			Ok(()) => true.into_lua_multi(lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(lua),
		});
	}
}
