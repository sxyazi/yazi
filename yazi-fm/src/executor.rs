use yazi_config::{keymap::{Control, Exec, Key, KeymapLayer}, KEYMAP};
use yazi_core::{input::InputMode, Ctx};

pub(super) struct Executor<'a> {
	cx: &'a mut Ctx,
}

impl<'a> Executor<'a> {
	#[inline]
	pub(super) fn new(cx: &'a mut Ctx) -> Self { Self { cx } }

	pub(super) fn handle(&mut self, key: Key) -> bool {
		if self.cx.which.visible {
			return self.cx.which.press(key);
		}
		if self.cx.input.visible && self.cx.input.type_(&key) {
			return true;
		}
		if self.cx.help.visible && self.cx.help.type_(&key) {
			return true;
		}

		let b = if self.cx.completion.visible {
			self.matches(KeymapLayer::Completion, key).or_else(|| self.matches(KeymapLayer::Input, key))
		} else if self.cx.help.visible {
			self.matches(KeymapLayer::Help, key)
		} else if self.cx.input.visible {
			self.matches(KeymapLayer::Input, key)
		} else if self.cx.select.visible {
			self.matches(KeymapLayer::Select, key)
		} else if self.cx.tasks.visible {
			self.matches(KeymapLayer::Tasks, key)
		} else {
			self.matches(KeymapLayer::Manager, key)
		};
		b == Some(true)
	}

	#[inline]
	fn matches(&mut self, layer: KeymapLayer, key: Key) -> Option<bool> {
		for Control { on, exec, .. } in KEYMAP.get(layer) {
			if on.is_empty() || on[0] != key {
				continue;
			}

			return Some(if on.len() > 1 {
				self.cx.which.show(&key, layer)
			} else {
				self.dispatch(exec, layer)
			});
		}
		None
	}

	#[inline]
	pub(super) fn dispatch(&mut self, exec: &[Exec], layer: KeymapLayer) -> bool {
		let mut render = false;
		for e in exec {
			render |= match layer {
				KeymapLayer::Manager => self.manager(e),
				KeymapLayer::Tasks => self.tasks(e),
				KeymapLayer::Select => self.select(e),
				KeymapLayer::Input => self.input(e),
				KeymapLayer::Help => self.help(e),
				KeymapLayer::Completion => self.completion(e),
				KeymapLayer::Which => unreachable!(),
			};
		}
		render
	}

	fn manager(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			(MANAGER, $name:ident $(,$args:expr)*) => {
				if exec.cmd == stringify!($name) {
					return self.cx.manager.$name(exec, $($args),*);
				}
			};
			(ACTIVE, $name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.cx.manager.active_mut().$name(exec);
				}
			};
			(TABS, $name:ident) => {
				if exec.cmd == concat!("tab_", stringify!($name)) {
					return self.cx.manager.tabs.$name(exec);
				}
			};
		}

		on!(ACTIVE, escape);
		on!(MANAGER, quit, &self.cx.tasks);
		on!(MANAGER, close, &self.cx.tasks);
		on!(MANAGER, suspend);

		// Navigation
		on!(ACTIVE, arrow);
		// on!(T, peek);
		on!(ACTIVE, leave);
		on!(ACTIVE, enter);
		on!(ACTIVE, back);
		on!(ACTIVE, forward);
		on!(ACTIVE, cd);
		on!(ACTIVE, reveal);

		// Selection
		on!(ACTIVE, select);
		on!(ACTIVE, select_all);
		on!(ACTIVE, visual_mode);

		// Operation
		on!(MANAGER, open);
		on!(MANAGER, yank);
		on!(MANAGER, paste, &self.cx.tasks);
		on!(MANAGER, link, &self.cx.tasks);
		on!(MANAGER, remove, &self.cx.tasks);
		on!(MANAGER, create);
		on!(MANAGER, rename);
		on!(ACTIVE, copy);
		on!(ACTIVE, shell);
		on!(ACTIVE, hidden);
		on!(ACTIVE, linemode);
		on!(ACTIVE, search);
		on!(ACTIVE, jump);

		// Find
		on!(ACTIVE, find);
		on!(ACTIVE, find_arrow);

		// Sorting
		on!(ACTIVE, sort);

		// Tabs
		on!(TABS, create);
		on!(TABS, close);
		on!(TABS, switch);
		on!(TABS, swap);

		match exec.cmd.as_bytes() {
			// Tasks
			b"tasks_show" => self.cx.tasks.toggle(),
			// Help
			b"help" => self.cx.help.toggle(KeymapLayer::Manager),
			_ => false,
		}
	}

	fn tasks(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.cx.tasks.$name(exec);
				}
			};
		}

		on!(arrow);
		on!(inspect);
		on!(cancel);

		match exec.cmd.as_str() {
			"close" => self.cx.tasks.toggle(),
			"help" => self.cx.help.toggle(KeymapLayer::Tasks),
			_ => false,
		}
	}

	fn select(&mut self, exec: &Exec) -> bool {
		macro_rules! on {
			($name:ident) => {
				if exec.cmd == stringify!($name) {
					return self.cx.select.$name(exec);
				}
			};
		}

		on!(close);
		on!(arrow);

		match exec.cmd.as_str() {
			"help" => self.cx.help.toggle(KeymapLayer::Select),
			_ => false,
		}
	}

	fn input(&mut self, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"close" => return self.cx.input.close(exec.named.contains_key("submit")),
			"escape" => return self.cx.input.escape(),

			"move" => {
				let step = exec.args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
				let in_operating = exec.named.contains_key("in-operating");
				return if in_operating {
					self.cx.input.move_in_operating(step)
				} else {
					self.cx.input.move_(step)
				};
			}

			"complete" => {
				return if exec.args.is_empty() {
					self.cx.completion.trigger(exec)
				} else {
					self.cx.input.complete(exec)
				};
			}
			_ => {}
		}

		match self.cx.input.mode() {
			InputMode::Normal => match exec.cmd.as_str() {
				"insert" => self.cx.input.insert(exec.named.contains_key("append")),
				"visual" => self.cx.input.visual(),

				"backward" => self.cx.input.backward(),
				"forward" => self.cx.input.forward(exec.named.contains_key("end-of-word")),
				"delete" => {
					self.cx.input.delete(exec.named.contains_key("cut"), exec.named.contains_key("insert"))
				}

				"yank" => self.cx.input.yank(),
				"paste" => self.cx.input.paste(exec.named.contains_key("before")),

				"undo" => self.cx.input.undo(),
				"redo" => self.cx.input.redo(),

				"help" => self.cx.help.toggle(KeymapLayer::Input),
				_ => false,
			},
			InputMode::Insert => false,
		}
	}

	fn help(&mut self, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"close" => self.cx.help.toggle(KeymapLayer::Help),
			"escape" => self.cx.help.escape(),

			"arrow" => {
				let step = exec.args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
				self.cx.help.arrow(step)
			}

			"filter" => self.cx.help.filter(),

			_ => false,
		}
	}

	fn completion(&mut self, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"trigger" => self.cx.completion.trigger(exec),
			"show" => self.cx.completion.show(exec),
			"close" => self.cx.completion.close(exec),

			"arrow" => self.cx.completion.arrow(exec),

			"help" => self.cx.help.toggle(KeymapLayer::Completion),
			_ => false,
		}
	}
}
