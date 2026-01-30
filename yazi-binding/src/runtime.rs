use std::{collections::VecDeque, mem};

use anyhow::{Context, Result};
use hashbrown::{HashMap, hash_map::EntryRef};
use mlua::Function;

#[derive(Debug)]
pub struct Runtime {
	frames:       VecDeque<RuntimeFrame>,
	blocks:       HashMap<String, Vec<Function>>,
	pub blocking: bool,
}

#[derive(Debug)]
pub struct RuntimeFrame {
	id: String,
}

impl Runtime {
	pub fn new() -> Self { Self { frames: <_>::default(), blocks: <_>::default(), blocking: false } }

	pub fn new_isolate(id: &str) -> Self {
		Self {
			frames:   VecDeque::from([RuntimeFrame { id: id.to_owned() }]),
			blocks:   <_>::default(),
			blocking: false,
		}
	}

	pub fn push(&mut self, id: &str) { self.frames.push_back(RuntimeFrame { id: id.to_owned() }); }

	pub fn pop(&mut self) -> Result<RuntimeFrame> {
		self.frames.pop_back().context("Runtime stack underflow")
	}

	pub fn critical_push(&mut self, id: &str, blocking: bool) -> bool {
		self.push(id);
		mem::replace(&mut self.blocking, blocking)
	}

	pub fn critical_pop(&mut self, blocking: bool) -> Result<RuntimeFrame> {
		self.blocking = blocking;
		self.pop()
	}

	pub fn current(&self) -> Result<&str> {
		self.frames.back().map(|f| f.id.as_str()).context("No current runtime frame")
	}

	pub fn current_owned(&self) -> Result<String> { self.current().map(ToOwned::to_owned) }

	pub fn get_block(&self, id: &str, calls: usize) -> Option<Function> {
		self.blocks.get(id).and_then(|v| v.get(calls)).cloned()
	}

	pub fn put_block(&mut self, f: &Function) -> Option<usize> {
		let Some(cur) = self.frames.back().filter(|f| f.id != "init") else { return None };
		Some(match self.blocks.entry_ref(&cur.id) {
			EntryRef::Occupied(mut oe) => {
				oe.get_mut().push(f.clone());
				oe.get().len() - 1
			}
			EntryRef::Vacant(ve) => {
				ve.insert(vec![f.clone()]);
				0
			}
		})
	}
}
