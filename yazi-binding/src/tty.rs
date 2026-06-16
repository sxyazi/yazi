use std::io::Write;

use mlua::{BorrowedBytes, ExternalError, IntoLuaMulti, Lua, MultiValue, Table, UserData, UserDataMethods};
use yazi_shim::mlua::{ByteString, LuaTableExt};
use yazi_term::sequence::{AgreeDrag, AgreeDrop, FinishDrop, PresentDrag, PresentDragIcon, ReadClipboard, StartDrag, StartDrop, WriteClipboard, WriteClipboardData};
use yazi_tty::TTY;

use crate::Error;

pub struct Tty;

impl Tty {
	fn queue(lua: &Lua, kind: &[u8], t: &Table) -> mlua::Result<MultiValue> {
		let mut w = TTY.writer();

		let result = match kind {
			b"AgreeDrag" => {
				let it = t.raw_get::<Table>("mimes")?.sequence_iter::<ByteString>(lua).flatten();
				write!(w, "{}", match &*t.raw_get::<BorrowedBytes>("type")? {
					b"copy" => AgreeDrag::Copy(it),
					b"move" => AgreeDrag::Move(it),
					b"either" => AgreeDrag::Either(it),
					_ => return Err("invalid AgreeDrag type".into_lua_err()),
				})
			}
			b"AgreeDrop" => {
				let it = t.raw_get::<Table>("mimes")?.sequence_iter::<ByteString>(lua).flatten();
				write!(w, "{}", match &*t.raw_get::<BorrowedBytes>("type")? {
					b"reject" => AgreeDrop::Reject,
					b"copy" => AgreeDrop::Copy(it),
					b"move" => AgreeDrop::Move(it),
					_ => return Err("invalid AgreeDrop type".into_lua_err()),
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
			b"ReadClipboard" => {
				write!(w, "{}", ReadClipboard {
					mime:    &t.raw_get::<BorrowedBytes>("mimes")?,
					pw:      &t.raw_get::<BorrowedBytes>("pw")?,
					name:    &t.raw_get::<BorrowedBytes>("name")?,
					primary: t.raw_get("primary")?,
				})
			}
			b"WriteClipboard" => {
				let mut data = Vec::new();
				for v in &t.sequence_values::<Table>().collect::<Result<Vec<_>, mlua::Error>>()? {
					data.push((
						v.raw_get::<BorrowedBytes>("mime")?,
						v.raw_get::<BorrowedBytes>("data")?,
						v.raw_get::<BorrowedBytes>("alias")?,
					));
				}
				write!(w, "{}", WriteClipboard {
					data: data
						.iter()
						.map(|(m, p, a)| { WriteClipboardData { mime: &m, payload: &p, alias: &a } })
						.collect(),
				})
			}
			_ => return Err("invalid sequence kind".into_lua_err()),
		};

		match result {
			Ok(()) => true.into_lua_multi(lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(lua),
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
