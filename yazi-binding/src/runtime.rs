use std::collections::VecDeque;

use hashbrown::HashMap;
use mlua::Function;

#[derive(Debug)]
pub struct Runtime {
	frames:      VecDeque<RuntimeFrame>,
	blocks:      HashMap<String, Vec<Function>>,
	pub initing: bool,
}

#[derive(Debug)]
struct RuntimeFrame {
	id:    String,
	calls: usize,
}

impl Runtime {
	pub fn new() -> Self { Self { frames: <_>::default(), blocks: <_>::default(), initing: true } }

	pub fn new_isolate(id: &str) -> Self {
		Self {
			frames:  VecDeque::from([RuntimeFrame { id: id.to_owned(), calls: 0 }]),
			blocks:  <_>::default(),
			initing: false,
		}
	}

	pub fn push(&mut self, id: &str) {
		self.frames.push_back(RuntimeFrame { id: id.to_owned(), calls: 0 });
	}

	pub fn pop(&mut self) { self.frames.pop_back(); }

	pub fn current(&self) -> Option<&str> { self.frames.back().map(|f| f.id.as_str()) }

	pub fn current_owned(&self) -> Option<String> { self.current().map(ToOwned::to_owned) }

	pub fn next_block(&mut self) -> Option<usize> {
		self.frames.back_mut().map(|f| {
			f.calls += 1;
			f.calls - 1
		})
	}

	pub fn get_block(&self, id: &str, calls: usize) -> Option<Function> {
		self.blocks.get(id).and_then(|v| v.get(calls)).cloned()
	}

	pub fn put_block(&mut self, f: Function) -> bool {
		let Some(cur) = self.frames.back() else { return false };
		if let Some(v) = self.blocks.get_mut(&cur.id) {
			v.push(f);
		} else {
			self.blocks.insert(cur.id.clone(), vec![f]);
		}
		true
	}
}
