use std::ops::RangeInclusive;

use mlua::{BorrowedBytes, ExternalError, FromLua, Lua, Table, Value};
use yazi_shared::{BytesExt, auth::AuthKind};

#[derive(Clone, Copy, Debug, Default)]
pub struct Capabilities {
	pub absolute:         bool,
	pub canonicalize:     bool,
	pub casefold:         bool,
	pub copy_from:        bool,
	pub copy_to:          bool,
	pub create_dir:       bool,
	pub create_dir_all:   bool,
	pub file:             bool,
	pub hard_link:        bool,
	pub metadata:         bool,
	pub open:             bool,
	pub read_dir:         bool,
	pub read_link:        bool,
	pub remove_dir:       bool,
	pub remove_dir_all:   bool,
	pub remove_dir_clean: bool,
	pub remove_file:      bool,
	pub rename:           bool,
	pub reroute:          u8,
	pub revalidate:       bool,
	pub set_attrs:        bool,
	pub symlink:          bool,
	pub symlink_dir:      bool,
	pub symlink_file:     bool,
	pub symlink_metadata: bool,
	pub trash:            bool,
}

impl Capabilities {
	fn all() -> Self {
		Self {
			absolute:         true,
			canonicalize:     true,
			casefold:         true,
			copy_from:        true,
			copy_to:          true,
			create_dir:       true,
			create_dir_all:   true,
			file:             true,
			hard_link:        true,
			metadata:         true,
			open:             true,
			read_dir:         true,
			read_link:        true,
			remove_dir:       true,
			remove_dir_all:   true,
			remove_dir_clean: true,
			remove_file:      true,
			rename:           true,
			reroute:          0b11,
			revalidate:       true,
			set_attrs:        true,
			symlink:          true,
			symlink_dir:      true,
			symlink_file:     true,
			symlink_metadata: true,
			trash:            true,
		}
	}

	fn set(&mut self, name: &[u8], value: u8) -> mlua::Result<()> {
		let get = |range: RangeInclusive<u8>| {
			if range.contains(&value) {
				Ok(value)
			} else {
				Err(format!("capability `{}` must be in {range:?}", name.display()).into_lua_err())
			}
		};

		match name {
			b"absolute" => self.absolute = get(0..=1)? != 0,
			b"canonicalize" => self.canonicalize = get(0..=1)? != 0,
			b"casefold" => self.casefold = get(0..=1)? != 0,
			b"copy_from" => self.copy_from = get(0..=1)? != 0,
			b"copy_to" => self.copy_to = get(0..=1)? != 0,
			b"create_dir" => self.create_dir = get(0..=1)? != 0,
			b"create_dir_all" => self.create_dir_all = get(0..=1)? != 0,
			b"file" => self.file = get(0..=1)? != 0,
			b"hard_link" => self.hard_link = get(0..=1)? != 0,
			b"metadata" => self.metadata = get(0..=1)? != 0,
			b"open" => self.open = get(0..=1)? != 0,
			b"read_dir" => self.read_dir = get(0..=1)? != 0,
			b"read_link" => self.read_link = get(0..=1)? != 0,
			b"remove_dir" => self.remove_dir = get(0..=1)? != 0,
			b"remove_dir_all" => self.remove_dir_all = get(0..=1)? != 0,
			b"remove_dir_clean" => self.remove_dir_clean = get(0..=1)? != 0,
			b"remove_file" => self.remove_file = get(0..=1)? != 0,
			b"rename" => self.rename = get(0..=1)? != 0,
			b"reroute" => self.reroute = get(0..=3)?,
			b"revalidate" => self.revalidate = get(0..=1)? != 0,
			b"set_attrs" => self.set_attrs = get(0..=1)? != 0,
			b"symlink" => self.symlink = get(0..=1)? != 0,
			b"symlink_dir" => self.symlink_dir = get(0..=1)? != 0,
			b"symlink_file" => self.symlink_file = get(0..=1)? != 0,
			b"symlink_metadata" => self.symlink_metadata = get(0..=1)? != 0,
			b"trash" => self.trash = get(0..=1)? != 0,
			_ => return Err(format!("unknown capability `{}`", name.display()).into_lua_err()),
		}
		Ok(())
	}

	pub fn for_kind(kind: AuthKind) -> Self {
		let mut caps = Self::all();

		match kind {
			AuthKind::Regular => {
				caps.reroute = 0;
				caps
			}
			AuthKind::Sftp => {
				caps.reroute = 0b01;
				caps.trash = false;
				caps
			}
			AuthKind::Mount | AuthKind::Hub | AuthKind::Scope => {
				caps.copy_from = false;
				caps.hard_link = false;
				caps.symlink = false;
				caps.symlink_dir = false;
				caps.symlink_file = false;
				caps.trash = false;
				caps
			}
			AuthKind::View => Self::default(),
		}
	}
}

impl FromLua for Capabilities {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let mut caps = Self::default();
		for pair in Table::from_lua(value, lua)?.pairs::<BorrowedBytes, u8>() {
			let (name, value) = pair?;
			caps.set(&name, value)?;
		}

		Ok(caps)
	}
}
