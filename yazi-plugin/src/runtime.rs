use std::collections::HashMap;

use mlua::{Function, UserData};

#[derive(Default)]
pub struct Runtime {
	pub current: Option<String>,
	pub calls:   usize,
	pub blocks:  HashMap<String, Vec<Function<'static>>>,
}

pub type RtRef<'lua> = mlua::UserDataRefMut<'lua, Runtime>;

impl Runtime {
	pub fn new(current: &str) -> Self {
		Self { current: Some(current.to_owned()), ..Default::default() }
	}

	pub fn swap(&mut self, name: &str) {
		self.current = Some(name.to_owned());
		self.calls = 0;
	}

	pub fn reset(&mut self) {
		self.current = None;
		self.calls = 0;
	}

	pub fn next_block(&mut self) -> usize {
		self.calls += 1;
		self.calls - 1
	}

	pub fn get_block(&self, name: &str, calls: usize) -> Option<Function<'static>> {
		self.blocks.get(name).and_then(|v| v.get(calls)).cloned()
	}

	pub fn push_block(&mut self, f: Function<'static>) -> bool {
		let Some(ref cur) = self.current else {
			return false;
		};

		if let Some(vec) = self.blocks.get_mut(cur) {
			vec.push(f);
		} else {
			self.blocks.insert(cur.clone(), vec![f]);
		}
		true
	}
}

impl UserData for Runtime {}
