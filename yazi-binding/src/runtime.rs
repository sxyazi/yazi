use anyhow::{Context, Result};
use compact_str::CompactString;
use hashbrown::HashMap;
use mlua::Function;

use crate::Scope;

#[derive(Debug, Default)]
pub struct Runtime {
	frames: Vec<RuntimeFrame>,
	blocks: HashMap<CompactString, Vec<Function>>,
}

#[derive(Clone, Debug, Default)]
struct RuntimeFrame {
	name:     CompactString,
	blocking: bool,
	scope:    Scope,
}

impl Runtime {
	pub fn new(name: &str, scope: Scope) -> Self {
		Self {
			frames: vec![RuntimeFrame { name: name.into(), scope, ..Default::default() }],
			..Default::default()
		}
	}

	pub fn enter(&mut self, name: &str, blocking: bool, scope: Scope) {
		self.frames.push(RuntimeFrame { name: name.into(), blocking, scope });
	}

	pub fn enter_nested(&mut self, name: &str) {
		let frame =
			RuntimeFrame { name: name.into(), ..self.frames.last().cloned().unwrap_or_default() };
		self.frames.push(frame);
	}

	pub fn enter_inherited(&mut self, name: &str, blocking: bool) {
		self.enter(name, blocking, self.scope());
	}

	pub fn leave(&mut self) -> Result<()> {
		self.frames.pop().map(|_| ()).context("Runtime stack underflow")
	}

	pub fn is_blocking(&self) -> bool { self.frames.last().is_some_and(|f| f.blocking) }

	pub fn scope(&self) -> Scope { self.frames.last().map(|f| f.scope.clone()).unwrap_or_default() }

	pub fn name(&self) -> Result<&str> {
		self.frames.last().map(|f| f.name.as_str()).context("No current runtime frame")
	}

	pub fn name_child_scope(&self) -> Result<(CompactString, Scope)> {
		self
			.frames
			.last()
			.map(|f| (f.name.clone(), f.scope.child()))
			.context("No current runtime frame")
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
