use crossterm::event::KeyCode;

use super::Ctx;
use crate::{config::{keymap::{Exec, Key, Single}, KEYMAP}, core::input::InputMode, misc::optinal_bool};

pub struct Executor;

impl Executor {
	pub fn handle(cx: &mut Ctx, key: Key) -> bool {
		let layer = if cx.input.visible {
			2
		} else if cx.tasks.visible {
			1
		} else {
			0
		};

		let mut render = false;
		let mut matched = false;
		let keymap = [&KEYMAP.manager, &KEYMAP.tasks, &KEYMAP.input][layer];

		for Single { on, exec } in keymap {
			if on.len() < 1 || on[0] != key {
				continue;
			}

			matched = true;
			for e in exec {
				if layer == 0 {
					render = Self::manager(cx, e) || render;
				} else if layer == 1 {
					render = Self::tasks(cx, e) || render;
				} else if layer == 2 {
					render = Self::input(cx, Some(e), key.code) || render;
				}
			}
		}

		if layer == 2 && !matched {
			render = Self::input(cx, None, key.code);
		}
		render
	}

	fn manager(cx: &mut Ctx, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"escape" => cx.manager.active_mut().escape(),
			"quit" => cx.manager.quit(&cx.tasks),
			"close" => cx.manager.close(&cx.tasks),

			// Navigation
			"arrow" => {
				let step = exec.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
				cx.manager.active_mut().arrow(step)
			}
			"leave" => cx.manager.active_mut().leave(),
			"enter" => cx.manager.active_mut().enter(),
			"back" => cx.manager.active_mut().back(),
			"forward" => cx.manager.active_mut().forward(),

			// Selection
			"select" => {
				let state = exec.named.get("state").cloned().unwrap_or("none".to_string());
				cx.manager.active_mut().select(optinal_bool(&state))
			}
			"visual_mode" => cx.manager.active_mut().visual_mode(exec.named.contains_key("unselect")),
			"select_all" => {
				let state = exec.named.get("state").cloned().unwrap_or("none".to_string());
				cx.manager.active_mut().select_all(optinal_bool(&state))
			}

			// Operation
			"yank" => cx.manager.yank(exec.named.contains_key("cut")),
			"paste" => {
				let dest = cx.manager.current().cwd.clone();
				let (cut, src) = cx.manager.yanked();

				let force = exec.named.contains_key("force");
				if *cut {
					cx.tasks.file_cut(src, dest, force)
				} else {
					cx.tasks.file_copy(src, dest, force, exec.named.contains_key("follow"))
				}
			}
			"remove" => {
				cx.tasks.file_remove(cx.manager.selected(), exec.named.contains_key("permanently"))
			}
			"create" => cx.manager.create(),
			"rename" => cx.manager.rename(),
			"hidden" => cx.manager.current_mut().hidden(match exec.args.get(0).map(|s| s.as_str()) {
				Some("show") => Some(true),
				Some("hide") => Some(false),
				_ => None,
			}),
			"search" => match exec.args.get(0).map(|s| s.as_str()).unwrap_or("") {
				"rg" => cx.manager.active_mut().search(true),
				"fd" => cx.manager.active_mut().search(false),
				_ => cx.manager.active_mut().search_stop(),
			},
			"jump" => match exec.args.get(0).map(|s| s.as_str()).unwrap_or("") {
				"fzf" => cx.manager.active_mut().jump(true),
				"zoxide" => cx.manager.active_mut().jump(false),
				_ => false,
			},

			// Tabs
			"tab_create" => {
				let path = if exec.named.contains_key("current") {
					cx.manager.current().cwd.clone()
				} else {
					exec.args.get(0).map(|p| p.into()).unwrap_or("/".into())
				};
				cx.manager.tabs_mut().create(&path)
			}
			"tab_close" => {
				let idx = exec.args.get(0).and_then(|i| i.parse().ok()).unwrap_or(0);
				cx.manager.tabs_mut().close(idx)
			}
			"tab_switch" => {
				let step = exec.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
				let rel = exec.named.contains_key("relative");
				cx.manager.tabs_mut().switch(step, rel)
			}
			"tab_swap" => {
				let step = exec.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
				cx.manager.tabs_mut().swap(step)
			}

			// Tasks
			"tasks_show" => cx.tasks.toggle(),

			_ => false,
		}
	}

	fn tasks(cx: &mut Ctx, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"close" => cx.tasks.toggle(),

			"arrow" => {
				let step = exec.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
				if step > 0 { cx.tasks.next() } else { cx.tasks.prev() }
			}

			"cancel" => cx.tasks.cancel(),
			_ => false,
		}
	}

	fn input(cx: &mut Ctx, exec: Option<&Exec>, code: KeyCode) -> bool {
		let exec = if let Some(e) = exec {
			e
		} else {
			if cx.input.mode() == InputMode::Insert {
				if let KeyCode::Char(c) = code {
					return cx.input.type_(c);
				}
			}
			return false;
		};

		match cx.input.mode() {
			InputMode::Normal => match exec.cmd.as_str() {
				"close" => cx.input.close(exec.named.contains_key("submit")),
				"escape" => cx.input.escape(),

				"insert" => cx.input.insert(exec.named.contains_key("append")),
				"visual" => cx.input.visual(),

				"move" => {
					let step = exec.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
					cx.input.move_(step)
				}

				"backward" => cx.input.backward(),
				"forward" => cx.input.forward(exec.named.contains_key("end-of-word")),
				"delete" => cx.input.delete(exec.named.contains_key("insert")),
				_ => false,
			},
			InputMode::Insert => match exec.cmd.as_str() {
				"close" => cx.input.close(exec.named.contains_key("submit")),
				"escape" => cx.input.escape(),
				"backspace" => cx.input.backspace(),
				_ => {
					if let KeyCode::Char(c) = code {
						return cx.input.type_(c);
					}
					false
				}
			},
		}
	}
}
