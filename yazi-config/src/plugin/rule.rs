use std::sync::atomic::Ordering;

use serde::{Deserialize, Deserializer};
use yazi_shared::{event::Cmd, Condition};

use crate::{Pattern, Priority, DEPRECATED_EXEC};

#[derive(Debug)]
pub struct PluginRule {
	pub id:    u8,
	pub cond:  Option<Condition>,
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	pub cmd:   Cmd,
	pub sync:  bool,
	pub multi: bool,
	pub prio:  Priority,
}

impl PluginRule {
	#[inline]
	pub fn any_file(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_file()) }

	#[inline]
	pub fn any_dir(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_dir()) }
}

// TODO: remove this once Yazi 0.3 is released
impl<'de> Deserialize<'de> for PluginRule {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		pub struct Shadow {
			#[serde(default)]
			pub id:    u8,
			pub cond:  Option<Condition>,
			pub name:  Option<Pattern>,
			pub mime:  Option<Pattern>,
			pub run:   Option<WrappedCmd>,
			pub exec:  Option<WrappedCmd>,
			#[serde(default)]
			pub sync:  bool,
			#[serde(default)]
			pub multi: bool,
			#[serde(default)]
			pub prio:  Priority,
		}

		let shadow = Shadow::deserialize(deserializer)?;

		#[derive(Deserialize)]
		struct WrappedCmd(#[serde(deserialize_with = "super::run_deserialize")] Cmd);

		if shadow.exec.is_some() {
			DEPRECATED_EXEC.store(true, Ordering::Relaxed);
		}
		let Some(run) = shadow.run.or(shadow.exec) else {
			return Err(serde::de::Error::custom("missing field `run` within `[plugin]`"));
		};

		Ok(Self {
			id:    shadow.id,
			cond:  shadow.cond,
			name:  shadow.name,
			mime:  shadow.mime,
			cmd:   run.0,
			sync:  shadow.sync,
			multi: shadow.multi,
			prio:  shadow.prio,
		})
	}
}
