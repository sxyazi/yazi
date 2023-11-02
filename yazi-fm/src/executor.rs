use yazi_config::{keymap::{Control, Exec, Key, KeymapLayer}, KEYMAP};
use yazi_core::{emit, input::InputMode, tab::FinderCase, Ctx};
use yazi_shared::{optional_bool, Url};

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
		match exec.cmd.as_str() {
			"escape" => self.cx.manager.active_mut().escape(exec),
			"quit" => self.cx.manager.quit(&self.cx.tasks, exec.named.contains_key("no-cwd-file")),
			"close" => self.cx.manager.close(&self.cx.tasks),
			"suspend" => self.cx.manager.suspend(),

			// Navigation
			"arrow" => {
				let step = exec.args.first().and_then(|s| s.parse().ok()).unwrap_or_default();
				self.cx.manager.active_mut().arrow(step)
			}
			"peek" => {
				let step = exec.args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
				self.cx.manager.active_mut().preview.arrow(step);
				self.cx.manager.peek(true, self.cx.image_layer())
			}
			"leave" => self.cx.manager.active_mut().leave(),
			"enter" => self.cx.manager.active_mut().enter(),
			"back" => self.cx.manager.active_mut().back(),
			"forward" => self.cx.manager.active_mut().forward(),
			"cd" => {
				let url = exec.args.first().map(Url::from).unwrap_or_default();
				if exec.named.contains_key("interactive") {
					self.cx.manager.active_mut().cd_interactive(url)
				} else {
					emit!(Cd(url));
					false
				}
			}

			// Selection
			"select" => {
				let state = exec.named.get("state").cloned().unwrap_or("none".to_string());
				self.cx.manager.active_mut().select(optional_bool(&state))
			}
			"select_all" => {
				let state = exec.named.get("state").cloned().unwrap_or("none".to_string());
				self.cx.manager.active_mut().select_all(optional_bool(&state))
			}
			"visual_mode" => self.cx.manager.active_mut().visual_mode(exec.named.contains_key("unset")),

			// Operation
			"open" => self.cx.manager.open(exec.named.contains_key("interactive")),
			"yank" => self.cx.manager.yank(exec.named.contains_key("cut")),
			"paste" => {
				let dest = self.cx.manager.cwd();
				let (cut, ref src) = self.cx.manager.yanked;

				let force = exec.named.contains_key("force");
				if cut {
					self.cx.tasks.file_cut(src, dest, force)
				} else {
					self.cx.tasks.file_copy(src, dest, force)
				}
			}
			"link" => {
				let (cut, ref src) = self.cx.manager.yanked;
				!cut
					&& self.cx.tasks.file_link(
						src,
						self.cx.manager.cwd(),
						exec.named.contains_key("relative"),
						exec.named.contains_key("force"),
					)
			}
			"remove" => {
				let targets = self.cx.manager.selected().into_iter().map(|f| f.url()).collect();
				let force = exec.named.contains_key("force");
				let permanently = exec.named.contains_key("permanently");
				self.cx.tasks.file_remove(targets, force, permanently)
			}
			"create" => self.cx.manager.create(exec.named.contains_key("force")),
			"rename" => self.cx.manager.rename(exec.named.contains_key("force")),
			"copy" => self.cx.manager.active().copy(exec.args.first().map(|s| s.as_str()).unwrap_or("")),
			"shell" => self.cx.manager.active().shell(
				exec.args.first().map(|e| e.as_str()).unwrap_or(""),
				exec.named.contains_key("block"),
				exec.named.contains_key("confirm"),
			),
			"hidden" => self.cx.manager.active_mut().hidden(exec),
			"linemode" => self.cx.manager.active_mut().linemode(exec),
			"search" => match exec.args.first().map(|s| s.as_str()).unwrap_or("") {
				"rg" => self.cx.manager.active_mut().search(true),
				"fd" => self.cx.manager.active_mut().search(false),
				_ => self.cx.manager.active_mut().search_stop(),
			},
			"jump" => match exec.args.first().map(|s| s.as_str()).unwrap_or("") {
				"fzf" => self.cx.manager.active_mut().jump(true),
				"zoxide" => self.cx.manager.active_mut().jump(false),
				_ => false,
			},

			// Find
			"find" => {
				let query = exec.args.first().map(|s| s.as_str());
				let prev = exec.named.contains_key("previous");
				let case = match (exec.named.contains_key("smart"), exec.named.contains_key("insensitive"))
				{
					(true, _) => FinderCase::Smart,
					(_, false) => FinderCase::Sensitive,
					(_, true) => FinderCase::Insensitive,
				};
				self.cx.manager.active_mut().find(query, prev, case)
			}
			"find_arrow" => self.cx.manager.active_mut().find_arrow(exec.named.contains_key("previous")),

			// Sorting
			"sort" => {
				let b = self.cx.manager.active_mut().sort(exec);
				self.cx.tasks.precache_size(&self.cx.manager.current().files);
				b
			}

			// Tabs
			"tab_create" => {
				let path = if exec.named.contains_key("current") {
					self.cx.manager.cwd().to_owned()
				} else {
					exec.args.first().map(Url::from).unwrap_or_else(|| Url::from("/"))
				};
				self.cx.manager.tabs.create(&path)
			}
			"tab_close" => {
				let idx = exec.args.first().and_then(|i| i.parse().ok()).unwrap_or(0);
				self.cx.manager.tabs.close(idx)
			}
			"tab_switch" => {
				let step = exec.args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
				let rel = exec.named.contains_key("relative");
				self.cx.manager.tabs.switch(step, rel)
			}
			"tab_swap" => {
				let step = exec.args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
				self.cx.manager.tabs.swap(step)
			}

			// Tasks
			"tasks_show" => self.cx.tasks.toggle(),

			// Help
			"help" => self.cx.help.toggle(KeymapLayer::Manager),

			_ => false,
		}
	}

	fn tasks(&mut self, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"close" => self.cx.tasks.toggle(),

			"arrow" => {
				let step = exec.args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
				if step > 0 { self.cx.tasks.next() } else { self.cx.tasks.prev() }
			}

			"inspect" => self.cx.tasks.inspect(),
			"cancel" => self.cx.tasks.cancel(),

			"help" => self.cx.help.toggle(KeymapLayer::Tasks),
			_ => false,
		}
	}

	fn select(&mut self, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"close" => self.cx.select.close(exec.named.contains_key("submit")),

			"arrow" => {
				let step: isize = exec.args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
				if step > 0 {
					self.cx.select.next(step as usize)
				} else {
					self.cx.select.prev(step.unsigned_abs())
				}
			}

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

			"complete" => return self.cx.completion.trigger(exec),
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
			"close" => self.cx.completion.close(exec.named.contains_key("submit")),

			"arrow" => {
				let step: isize = exec.args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
				if step > 0 {
					self.cx.completion.next(step as usize)
				} else {
					self.cx.completion.prev(step.unsigned_abs())
				}
			}

			"help" => self.cx.help.toggle(KeymapLayer::Completion),
			_ => false,
		}
	}
}
