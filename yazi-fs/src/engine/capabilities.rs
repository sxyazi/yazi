use mlua::{FromLua, Lua, Table, Value};
use yazi_shared::auth::AuthKind;

bitflags::bitflags! {
	#[derive(Clone, Copy, Debug, Default)]
	pub struct Capabilities: u32 {
		const ABSOLUTE         = 1 << 0;
		const CANONICALIZE     = 1 << 1;
		const CASEFOLD         = 1 << 2;
		const COPY_FROM        = 1 << 3;
		const COPY_TO          = 1 << 4;
		const CREATE_DIR       = 1 << 5;
		const CREATE_DIR_ALL   = 1 << 6;
		const FILE             = 1 << 7;
		const HARD_LINK        = 1 << 8;
		const METADATA         = 1 << 9;
		const OPEN             = 1 << 10;
		const READ_DIR         = 1 << 11;
		const READ_LINK        = 1 << 12;
		const REVALIDATE       = 1 << 13;
		const REMOVE_DIR       = 1 << 14;
		const REMOVE_DIR_ALL   = 1 << 15;
		const REMOVE_DIR_CLEAN = 1 << 16;
		const REMOVE_FILE      = 1 << 17;
		const RENAME           = 1 << 18;
		const SET_ATTRS        = 1 << 19;
		const SYMLINK          = 1 << 20;
		const SYMLINK_DIR      = 1 << 21;
		const SYMLINK_FILE     = 1 << 22;
		const SYMLINK_METADATA = 1 << 23;
		const TRASH            = 1 << 24;
	}
}

impl Capabilities {
	pub fn for_kind(kind: AuthKind) -> Self {
		let all = Self::all();

		match kind {
			AuthKind::Regular => all,
			AuthKind::Sftp => all & !Self::TRASH,
			AuthKind::Mount | AuthKind::Hub | AuthKind::Scope => {
				all
					& !(Self::COPY_FROM
						| Self::SYMLINK
						| Self::SYMLINK_DIR
						| Self::SYMLINK_FILE
						| Self::HARD_LINK
						| Self::TRASH)
			}
			AuthKind::View => Self::empty(),
		}
	}
}

impl FromLua for Capabilities {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		let mut caps = Self::empty();
		for (key, cap) in [
			("absolute", Self::ABSOLUTE),
			("canonicalize", Self::CANONICALIZE),
			("casefold", Self::CASEFOLD),
			("copy_from", Self::COPY_FROM),
			("copy_to", Self::COPY_TO),
			("create_dir", Self::CREATE_DIR),
			("create_dir_all", Self::CREATE_DIR_ALL),
			("file", Self::FILE),
			("hard_link", Self::HARD_LINK),
			("metadata", Self::METADATA),
			("open", Self::OPEN),
			("read_dir", Self::READ_DIR),
			("read_link", Self::READ_LINK),
			("revalidate", Self::REVALIDATE),
			("remove_dir", Self::REMOVE_DIR),
			("remove_dir_all", Self::REMOVE_DIR_ALL),
			("remove_dir_clean", Self::REMOVE_DIR_CLEAN),
			("remove_file", Self::REMOVE_FILE),
			("rename", Self::RENAME),
			("set_attrs", Self::SET_ATTRS),
			("symlink", Self::SYMLINK),
			("symlink_dir", Self::SYMLINK_DIR),
			("symlink_file", Self::SYMLINK_FILE),
			("symlink_metadata", Self::SYMLINK_METADATA),
			("trash", Self::TRASH),
		] {
			caps.set(cap, t.raw_get(key)?);
		}

		Ok(caps)
	}
}
