use anyhow::{Context, Result, ensure};
use mlua::{FromLua, Lua, Table, Value};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Delta {
	pub total:     u64,
	pub succeeded: u64,
	pub failed:    u64,
	pub workload:  Option<u64>,
	pub processed: u64,
}

impl From<u64> for Delta {
	fn from(n: u64) -> Self { Self { total: n, succeeded: n, ..Self::default() } }
}

impl Delta {
	pub fn try_add(mut self, rhs: Self) -> Result<Self> {
		let add = |a: u64, b: u64| a.checked_add(b).context("Removal progress overflow");

		self.total = add(self.total, rhs.total)?;
		self.succeeded = add(self.succeeded, rhs.succeeded)?;
		self.failed = add(self.failed, rhs.failed)?;
		self.processed = add(self.processed, rhs.processed)?;

		if let Some(workload) = rhs.workload {
			self.workload = Some(add(self.workload.unwrap_or_default(), workload)?);
		}

		ensure!(self.workload.is_some() || self.processed == 0, "processed requires workload");
		ensure!(self.processed <= self.workload.unwrap_or_default(), "processed exceeds workload");
		ensure!(add(self.succeeded, self.failed)? <= self.total, "succeeded + failed exceeds total");
		Ok(self)
	}
}

impl FromLua for Delta {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;

		Ok(Self {
			total:     t.raw_get("total").unwrap_or_default(),
			succeeded: t.raw_get("succeeded").unwrap_or_default(),
			failed:    t.raw_get("failed").unwrap_or_default(),
			workload:  t.raw_get("workload")?,
			processed: t.raw_get("processed").unwrap_or_default(),
		})
	}
}
