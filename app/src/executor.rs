use core::{emit, files::FilesSorter, input::InputMode};
use std::path::PathBuf;

use config::{keymap::{Control, Exec, Key, KeymapLayer}, manager::SortBy, KEYMAP};
use shared::optional_bool;

use super::Ctx;

pub(super) struct Executor;

impl Executor {
	pub(super) fn handle(cx: &mut Ctx, key: Key) -> bool {
		let layer = cx.layer();
		if layer == KeymapLayer::Which {
			return cx.which.press(key);
		}

		if layer == KeymapLayer::Input && cx.input.type_(&key) {
			return true;
		}

		if layer == KeymapLayer::Help && cx.help.type_(&key) {
			return true;
		}

		for Control { on, exec, .. } in KEYMAP.get(layer) {
			if on.is_empty() || on[0] != key {
				continue;
			}

			return if on.len() > 1 {
				cx.which.show(&key, layer)
			} else {
				Self::dispatch(cx, exec, layer)
			};
		}
		false
	}

	#[inline]
	pub(super) fn dispatch(cx: &mut Ctx, exec: &Vec<Exec>, layer: KeymapLayer) -> bool {
		let mut render = false;
		for e in exec {
			render |= match layer {
				KeymapLayer::Manager => Self::manager(cx, e),
				KeymapLayer::Tasks => Self::tasks(cx, e),
				KeymapLayer::Select => Self::select(cx, e),
				KeymapLayer::Input => Self::input(cx, e),
				KeymapLayer::Help => Self::help(cx, e),
				KeymapLayer::Which => unreachable!(),
			};
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
			"peek" => {
				let step = exec.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
				cx.manager.active_mut().update_peek(step, None);
				cx.manager.peek(true, cx.image_layer())
			}
			"leave" => cx.manager.active_mut().leave(),
			"enter" => cx.manager.active_mut().enter(),
			"back" => cx.manager.active_mut().back(),
			"forward" => cx.manager.active_mut().forward(),
			"cd" => {
				let path = exec.args.get(0).map(PathBuf::from).unwrap_or_default();
				if exec.named.contains_key("interactive") {
					cx.manager.active_mut().cd_interactive(path)
				} else {
					emit!(Cd(path));
					false
				}
			}

			// Selection
			"select" => {
				let state = exec.named.get("state").cloned().unwrap_or("none".to_string());
				cx.manager.active_mut().select(optional_bool(&state))
			}
			"select_all" => {
				let state = exec.named.get("state").cloned().unwrap_or("none".to_string());
				cx.manager.active_mut().select_all(optional_bool(&state))
			}
			"visual_mode" => cx.manager.active_mut().visual_mode(exec.named.contains_key("unset")),

			// Operation
			"open" => cx.manager.open(exec.named.contains_key("interactive")),
			"yank" => cx.manager.yank(exec.named.contains_key("cut")),
			"paste" => {
				let dest = cx.manager.cwd().to_owned();
				let (cut, src) = cx.manager.yanked();

				let force = exec.named.contains_key("force");
				if *cut {
					cx.tasks.file_cut(src, dest, force)
				} else {
					cx.tasks.file_copy(src, dest, force, exec.named.contains_key("follow"))
				}
			}
			"remove" => {
				let targets = cx.manager.selected().into_iter().map(|f| f.path_owned()).collect();
				cx.tasks.file_remove(targets, exec.named.contains_key("permanently"))
			}
			"create" => cx.manager.create(),
			"rename" => cx.manager.rename(),
			"copy" => cx.manager.active().copy(exec.args.get(0).map(|s| s.as_str()).unwrap_or("")),
			"shell" => cx.manager.active().shell(
				exec.args.get(0).map(|e| e.as_str()).unwrap_or(""),
				exec.named.contains_key("block"),
				exec.named.contains_key("confirm"),
			),
			"hidden" => {
				cx.manager.active_mut().set_show_hidden(match exec.args.get(0).map(|s| s.as_str()) {
					Some("show") => Some(true),
					Some("hide") => Some(false),
					_ => None,
				})
			}
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

			// Sorting
			"sort" => {
				let b = cx.manager.active_mut().set_sorter(FilesSorter {
					by:        SortBy::try_from(exec.args.get(0).cloned().unwrap_or_default())
						.unwrap_or_default(),
					reverse:   exec.named.contains_key("reverse"),
					dir_first: exec.named.contains_key("dir_first"),
				});
				cx.tasks.precache_size(&cx.manager.current().files);
				b
			}

			// Tabs
			"tab_create" => {
				let path = if exec.named.contains_key("current") {
					cx.manager.cwd().to_owned()
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

			// Help
			"help" => cx.help.toggle(cx.layer()),

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

			"inspect" => cx.tasks.inspect(),
			"cancel" => cx.tasks.cancel(),

			"help" => cx.help.toggle(cx.layer()),
			_ => false,
		}
	}

	fn select(cx: &mut Ctx, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"close" => cx.select.close(exec.named.contains_key("submit")),

			"arrow" => {
				let step: isize = exec.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
				if step > 0 { cx.select.next(step as usize) } else { cx.select.prev(step.unsigned_abs()) }
			}

			"help" => cx.help.toggle(cx.layer()),
			_ => false,
		}
	}

	fn input(cx: &mut Ctx, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"close" => return cx.input.close(exec.named.contains_key("submit")),
			"escape" => return cx.input.escape(),

			"move" => {
				let step = exec.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
				let in_operating = exec.named.contains_key("in-operating");
				return if in_operating { cx.input.move_in_operating(step) } else { cx.input.move_(step) };
			}
			_ => {}
		}

		match cx.input.mode() {
			InputMode::Normal => match exec.cmd.as_str() {
				"insert" => cx.input.insert(exec.named.contains_key("append")),
				"visual" => cx.input.visual(),

				"backward" => cx.input.backward(),
				"forward" => cx.input.forward(exec.named.contains_key("end-of-word")),
				"delete" => {
					cx.input.delete(exec.named.contains_key("cut"), exec.named.contains_key("insert"))
				}

				"yank" => cx.input.yank(),
				"paste" => cx.input.paste(exec.named.contains_key("before")),

				"undo" => cx.input.undo(),
				"redo" => cx.input.redo(),

				"help" => cx.help.toggle(cx.layer()),
				_ => false,
			},
			InputMode::Insert => false,
		}
	}

	fn help(cx: &mut Ctx, exec: &Exec) -> bool {
		match exec.cmd.as_str() {
			"close" => cx.help.toggle(cx.layer()),
			"escape" => cx.help.escape(),

			"arrow" => {
				let step = exec.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
				cx.help.arrow(step)
			}

			"filter" => cx.help.filter(),

			_ => false,
		}
	}
}
