use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mlua::{IntoLua, Lua, Value};

use crate::cha::{Cha, ChaMode};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Attrs {
	pub mode:  Option<ChaMode>,
	pub atime: Option<SystemTime>,
	pub btime: Option<SystemTime>,
	pub mtime: Option<SystemTime>,
}

impl From<Cha> for Attrs {
	fn from(value: Cha) -> Self {
		Self { mode: Some(value.mode), atime: value.atime, btime: value.btime, mtime: value.mtime }
	}
}

impl TryFrom<Attrs> for std::fs::FileTimes {
	type Error = ();

	fn try_from(value: Attrs) -> Result<Self, Self::Error> {
		if !value.has_times() {
			return Err(());
		}

		let mut t = Self::new();
		if let Some(atime) = value.atime {
			t = t.set_accessed(atime);
		}

		#[cfg(target_os = "macos")]
		if let Some(btime) = value.btime {
			use std::os::macos::fs::FileTimesExt;
			t = t.set_created(btime);
		}

		#[cfg(windows)]
		if let Some(btime) = value.btime {
			use std::os::windows::fs::FileTimesExt;
			t = t.set_created(btime);
		}

		if let Some(mtime) = value.mtime {
			t = t.set_modified(mtime);
		}

		Ok(t)
	}
}

impl TryFrom<Attrs> for std::fs::Permissions {
	type Error = ();

	fn try_from(_value: Attrs) -> Result<Self, Self::Error> {
		#[cfg(unix)]
		if let Some(mode) = _value.mode {
			return Ok(mode.into());
		}

		Err(())
	}
}

impl Attrs {
	pub fn mode(mode: ChaMode) -> Self { Self { mode: Some(mode), ..Default::default() } }

	pub fn has_times(self) -> bool {
		self.atime.is_some() || self.btime.is_some() || self.mtime.is_some()
	}

	pub fn atime_dur(self) -> Option<Duration> { self.atime?.duration_since(UNIX_EPOCH).ok() }

	pub fn btime_dur(self) -> Option<Duration> { self.btime?.duration_since(UNIX_EPOCH).ok() }

	pub fn mtime_dur(self) -> Option<Duration> { self.mtime?.duration_since(UNIX_EPOCH).ok() }
}

impl IntoLua for Attrs {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("mode", self.mode.map(|m| m.bits()).into_lua(lua)?),
				("atime", self.atime_dur().map(|d| d.as_secs_f64()).into_lua(lua)?),
				("btime", self.btime_dur().map(|d| d.as_secs_f64()).into_lua(lua)?),
				("mtime", self.mtime_dur().map(|d| d.as_secs_f64()).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
