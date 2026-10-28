use std::mem;

use anyhow::{Context, Result};
use compact_str::CompactString;
use hashbrown::HashMap;
use mlua::Function;
use yazi_shared::scope::{Scope, ScopeLike};

use super::{RuntimeFrame, RuntimeSeed};

#[derive(Debug, Default)]
pub struct Runtime {
	frames: Vec<RuntimeFrame>,
	blocks: HashMap<CompactString, Vec<Function>>,
}

impl From<&Runtime> for Runtime {
	fn from(value: &Runtime) -> Self { Self { frames: value.frames.clone(), ..Self::default() } }
}

impl From<&RuntimeSeed> for Runtime {
	fn from(value: &RuntimeSeed) -> Self { Self::new(value.clone()) }
}

impl Runtime {
	pub fn new(seed: RuntimeSeed) -> Self {
		Self { frames: vec![RuntimeFrame { seed, blocking: false }], ..Default::default() }
	}

	pub fn swap(&mut self, other: &mut Self) { mem::swap(&mut self.frames, &mut other.frames); }

	pub fn enter(&mut self, seed: RuntimeSeed, blocking: bool) {
		self.frames.push(RuntimeFrame { seed, blocking });
	}

	pub fn enter_blocking(&mut self, name: impl Into<CompactString>, scope: impl Into<Scope>) {
		self.enter(RuntimeSeed::new(name, scope), true);
	}

	pub fn enter_nested(&mut self, name: &str) {
		self.enter(RuntimeSeed::new(name, &*self), self.is_blocking());
	}

	pub fn leave(&mut self) -> Result<()> {
		self.frames.pop().map(|_| ()).context("Runtime stack underflow")
	}

	pub fn is_blocking(&self) -> bool { self.frames.last().is_some_and(|f| f.blocking) }

	pub fn name(&self) -> Result<&str> {
		self.frames.last().map(|f| f.name.as_str()).context("No current runtime frame")
	}

	pub fn module(&self) -> Result<&str> {
		let s = self.name()?;
		Ok(s.split('.').next().unwrap_or(s))
	}

	pub fn get_block(&self, name: &str, calls: usize) -> Option<Function> {
		self.blocks.get(name).and_then(|v| v.get(calls)).cloned()
	}

	pub fn put_block(&mut self, f: &Function) -> Option<usize> {
		let cur = self.frames.last().filter(|f| f.name != "init")?;
		let blocks = self.blocks.entry_ref(&cur.name).or_default();

		blocks.push(f.clone());
		Some(blocks.len() - 1)
	}
}

impl ScopeLike for Runtime {
	fn scope(&self) -> &Scope { self.frames.last().map_or_else(Default::default, |f| &f.scope) }
}
