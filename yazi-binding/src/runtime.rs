use std::{collections::VecDeque, mem};

use hashbrown::{HashMap, hash_map::EntryRef};
use mlua::Function;

#[derive(Debug)]
pub struct Runtime {
	frames:       VecDeque<RuntimeFrame>,
	blocks:       HashMap<String, Vec<Function>>,
	pub blocking: bool,
}

#[derive(Debug)]
struct RuntimeFrame {
	id: String,
}

impl Runtime {
	pub fn new() -> Self { Self { frames: <_>::default(), blocks: <_>::default(), blocking: true } }

	pub fn new_isolate(id: &str) -> Self {
		Self {
			frames:   VecDeque::from([RuntimeFrame { id: id.to_owned() }]),
			blocks:   <_>::default(),
			blocking: false,
		}
	}

	pub fn push(&mut self, id: &str) { self.frames.push_back(RuntimeFrame { id: id.to_owned() }); }

	pub fn pop(&mut self) { self.frames.pop_back(); }

	pub fn critical_push(&mut self, id: &str, blocking: bool) -> bool {
		self.push(id);
		mem::replace(&mut self.blocking, blocking)
	}

	pub fn critical_pop(&mut self, blocking: bool) {
		self.pop();
		self.blocking = blocking;
	}

	pub fn current(&self) -> Option<&str> { self.frames.back().map(|f| f.id.as_str()) }

	pub fn current_owned(&self) -> Option<String> { self.current().map(ToOwned::to_owned) }

	pub fn get_block(&self, id: &str, calls: usize) -> Option<Function> {
		self.blocks.get(id).and_then(|v| v.get(calls)).cloned()
	}

	pub fn put_block(&mut self, f: &Function) -> Option<usize> {
		let Some(cur) = self.frames.back() else { return None };
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
